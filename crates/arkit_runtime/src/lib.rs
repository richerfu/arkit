use std::any::Any;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, LazyLock, Mutex};

pub use arkit_core::theme;
pub use arkit_core::{advanced, window, Settings, Theme};
pub use arkit_futures::{Subscription, SubscriptionHandle};

pub type Element<Message, AppTheme = Theme, AppRenderer = ()> =
    arkit_core::Element<'static, Message, AppTheme, AppRenderer>;

pub trait Executor: 'static {}

#[derive(Debug, Default)]
pub struct DefaultExecutor;

impl Executor for DefaultExecutor {}

thread_local! {
    static DISPATCHER: RefCell<Option<RuntimeDispatcher>> = RefCell::new(None);
    static CURRENT_RUNTIME: RefCell<Option<RuntimeHandle>> = RefCell::new(None);
    static UI_LOOP_EFFECTS: RefCell<Vec<Box<dyn FnOnce()>>> = RefCell::new(Vec::new());
    static UI_WAKER: RefCell<Option<Rc<dyn Fn()>>> = RefCell::new(None);
}

pub type RuntimeDispatcher = Rc<dyn Fn(Box<dyn Any + Send>)>;
pub type GlobalRuntimeDispatcher = Arc<dyn Fn(Box<dyn Any + Send>) + Send + Sync>;

static GLOBAL_DISPATCHER: LazyLock<Mutex<Option<GlobalRuntimeDispatcher>>> =
    LazyLock::new(|| Mutex::new(None));

#[derive(Clone)]
pub struct RuntimeHandle {
    request_rerender: Rc<dyn Fn()>,
}

impl RuntimeHandle {
    pub fn new(request_rerender: impl Fn() + 'static) -> Self {
        Self {
            request_rerender: Rc::new(request_rerender),
        }
    }

    pub fn request_rerender(&self) {
        (self.request_rerender)();
    }
}

pub fn dispatch<Message>(message: Message)
where
    Message: Send + 'static,
{
    if let Some(dispatcher) = DISPATCHER.with(|state| state.borrow().as_ref().cloned()) {
        dispatcher(Box::new(message));
        return;
    }

    if let Some(dispatcher) = GLOBAL_DISPATCHER
        .lock()
        .expect("global dispatcher lock")
        .clone()
    {
        dispatcher(Box::new(message));
        return;
    }

    panic!("arkit runtime dispatch called without an active dispatcher");
}

pub fn set_dispatcher(dispatcher: Option<RuntimeDispatcher>) {
    DISPATCHER.with(|state| {
        state.replace(dispatcher);
    });
}

pub fn set_global_dispatcher(dispatcher: Option<GlobalRuntimeDispatcher>) {
    *GLOBAL_DISPATCHER.lock().expect("global dispatcher lock") = dispatcher;
}

pub fn global_dispatcher() -> Option<GlobalRuntimeDispatcher> {
    GLOBAL_DISPATCHER
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}

pub fn with_global_dispatcher<R>(
    dispatcher: Option<GlobalRuntimeDispatcher>,
    f: impl FnOnce() -> R,
) -> R {
    let Some(dispatcher) = dispatcher else {
        return f();
    };

    let local_dispatcher: RuntimeDispatcher = Rc::new(move |message| dispatcher(message));
    with_dispatcher(local_dispatcher, f)
}

pub fn with_dispatcher<R>(dispatcher: RuntimeDispatcher, f: impl FnOnce() -> R) -> R {
    let previous = DISPATCHER.with(|state| state.replace(Some(dispatcher)));
    let result = f();
    DISPATCHER.with(|state| {
        state.replace(previous);
    });
    result
}

pub fn set_current_runtime(runtime: Option<RuntimeHandle>) {
    CURRENT_RUNTIME.with(|state| {
        state.replace(runtime);
    });
}

pub fn current_runtime() -> Option<RuntimeHandle> {
    CURRENT_RUNTIME.with(|state| state.borrow().clone())
}

pub fn set_ui_waker(waker: Option<Rc<dyn Fn()>>) {
    UI_WAKER.with(|state| {
        state.replace(waker);
    });
}

pub fn queue_ui_loop(effect: impl FnOnce() + 'static) {
    UI_LOOP_EFFECTS.with(|state| {
        state.borrow_mut().push(Box::new(effect));
    });

    UI_WAKER.with(|state| {
        if let Some(waker) = state.borrow().as_ref() {
            waker();
        }
    });
}

pub fn run_ui_loop_effects() {
    let effects = UI_LOOP_EFFECTS.with(|state| state.replace(Vec::new()));
    for effect in effects {
        effect();
    }
}

pub fn clear_ui_loop_effects() {
    UI_LOOP_EFFECTS.with(|state| {
        state.borrow_mut().clear();
    });
}

#[doc(hidden)]
pub type BoxedTaskFuture<Message> = Pin<Box<dyn Future<Output = Message> + Send + 'static>>;

#[doc(hidden)]
pub enum TaskAction<Message> {
    Ready(Box<dyn FnOnce(&mut Vec<Message>)>),
    Future(BoxedTaskFuture<Message>),
}

pub struct Task<Message> {
    actions: Vec<TaskAction<Message>>,
}

impl<Message: Send + 'static> Task<Message> {
    pub fn none() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    pub fn done(value: Message) -> Self {
        Self {
            actions: vec![TaskAction::Ready(Box::new(move |messages| {
                messages.push(value)
            }))],
        }
    }

    pub fn batch(tasks: impl IntoIterator<Item = Self>) -> Self {
        let mut actions = Vec::new();
        for task in tasks {
            actions.extend(task.actions);
        }
        Self { actions }
    }

    pub fn run(action: impl FnOnce() -> Message + 'static) -> Self {
        Self {
            actions: vec![TaskAction::Ready(Box::new(move |messages| {
                messages.push(action())
            }))],
        }
    }

    pub fn perform<T: Send + 'static>(
        operation: impl Future<Output = T> + Send + 'static,
        map: impl FnOnce(T) -> Message + Send + 'static,
    ) -> Self {
        Self {
            actions: vec![TaskAction::Future(Box::pin(
                async move { map(operation.await) },
            ))],
        }
    }

    pub fn map<B: Send + 'static>(
        self,
        map: impl Fn(Message) -> B + Send + Sync + 'static,
    ) -> Task<B> {
        let map = Arc::new(map);
        let mut actions = Vec::with_capacity(self.actions.len());

        for action in self.actions {
            let map = map.clone();
            match action {
                TaskAction::Ready(action) => {
                    actions.push(TaskAction::Ready(Box::new(move |messages: &mut Vec<B>| {
                        let mut source = Vec::new();
                        action(&mut source);
                        messages.extend(source.into_iter().map(|message| map(message)));
                    })));
                }
                TaskAction::Future(future) => {
                    actions.push(TaskAction::Future(Box::pin(
                        async move { map(future.await) },
                    )));
                }
            }
        }

        Task { actions }
    }

    pub fn units(&self) -> usize {
        self.actions.len()
    }

    pub fn into_messages(self) -> Vec<Message> {
        let mut messages = Vec::new();
        for action in self.actions {
            match action {
                TaskAction::Ready(action) => action(&mut messages),
                TaskAction::Future(_) => {
                    panic!("Task::into_messages cannot consume async tasks");
                }
            }
        }
        messages
    }

    #[doc(hidden)]
    pub fn into_actions(self) -> Vec<TaskAction<Message>> {
        self.actions
    }
}

impl<Message: Send + 'static> Default for Task<Message> {
    fn default() -> Self {
        Self::none()
    }
}

#[derive(Clone)]
pub struct Preset<State, Message> {
    pub state: fn() -> State,
    pub boot: fn() -> Task<Message>,
}

pub trait Program: Sized {
    type State: 'static;
    type Message: Send + 'static;
    type Theme: theme::Base + Default;
    type Renderer: Default + 'static;
    type Executor: Executor;

    fn name() -> &'static str;
    fn settings(&self) -> Settings;
    fn window(&self) -> Option<window::Settings>;
    fn boot(&self) -> (Self::State, Task<Self::Message>);
    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message>;
    fn view(
        &self,
        state: &Self::State,
        window: window::Id,
    ) -> Element<Self::Message, Self::Theme, Self::Renderer>;

    fn title(&self, _state: &Self::State, _window: window::Id) -> String {
        Self::name().to_string()
    }

    fn subscription(&self, _state: &Self::State) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn theme(&self, _state: &Self::State, _window: window::Id) -> Option<Self::Theme> {
        None
    }

    fn style(&self, _state: &Self::State, theme: &Self::Theme) -> theme::Style {
        advanced::default_style(theme)
    }

    fn scale_factor(&self, _state: &Self::State, _window: window::Id) -> f32 {
        1.0
    }

    fn presets(&self) -> &[Preset<Self::State, Self::Message>] {
        &[]
    }
}

pub struct Application<State, Message, AppTheme = Theme, AppRenderer = ()>
where
    State: 'static,
    Message: Send + 'static,
    AppTheme: theme::Base + Default,
    AppRenderer: Default + 'static,
{
    boot: Rc<dyn Fn() -> (State, Task<Message>)>,
    update: Rc<dyn Fn(&mut State, Message) -> Task<Message>>,
    view: Rc<dyn Fn(&State) -> Element<Message, AppTheme, AppRenderer>>,
    title: Rc<dyn Fn(&State, window::Id) -> String>,
    subscription: Rc<dyn Fn(&State) -> Subscription<Message>>,
    theme: Rc<dyn Fn(&State, window::Id) -> Option<AppTheme>>,
    style: Rc<dyn Fn(&State, &AppTheme) -> theme::Style>,
    settings: Settings,
    window: Option<window::Settings>,
    scale_factor: Rc<dyn Fn(&State, window::Id) -> f32>,
    presets: Vec<Preset<State, Message>>,
    name: &'static str,
}

pub fn application<State, Message, Boot, Update, View, AppTheme, AppRenderer>(
    boot: Boot,
    update: Update,
    view: View,
) -> Application<State, Message, AppTheme, AppRenderer>
where
    State: 'static,
    Message: Send + 'static,
    Boot: Fn() -> State + 'static,
    Update: Fn(&mut State, Message) -> Task<Message> + 'static,
    View: Fn(&State) -> Element<Message, AppTheme, AppRenderer> + 'static,
    AppTheme: theme::Base + Default,
    AppRenderer: Default + 'static,
{
    Application {
        boot: Rc::new(move || (boot(), Task::none())),
        update: Rc::new(update),
        view: Rc::new(view),
        title: Rc::new(|_, _| String::from("arkit")),
        subscription: Rc::new(|_| Subscription::none()),
        theme: Rc::new(|_, _| None),
        style: Rc::new(|_, theme| advanced::default_style(theme)),
        settings: Settings::default(),
        window: Some(window::Settings::default()),
        scale_factor: Rc::new(|_, _| 1.0),
        presets: Vec::new(),
        name: "arkit",
    }
}

impl<State, Message, AppTheme, AppRenderer> Application<State, Message, AppTheme, AppRenderer>
where
    State: 'static,
    Message: Send + 'static,
    AppTheme: theme::Base + Default,
    AppRenderer: Default + 'static,
{
    pub fn boot(mut self, boot: impl Fn() -> (State, Task<Message>) + 'static) -> Self {
        self.boot = Rc::new(boot);
        self
    }

    pub fn title(mut self, f: impl Fn(&State, window::Id) -> String + 'static) -> Self {
        self.title = Rc::new(f);
        self
    }

    pub fn subscription(mut self, f: impl Fn(&State) -> Subscription<Message> + 'static) -> Self {
        self.subscription = Rc::new(f);
        self
    }

    pub fn theme(mut self, f: impl Fn(&State, window::Id) -> AppTheme + 'static) -> Self {
        self.theme = Rc::new(move |state, window| Some(f(state, window)));
        self
    }

    pub fn style(mut self, f: impl Fn(&State, &AppTheme) -> theme::Style + 'static) -> Self {
        self.style = Rc::new(f);
        self
    }

    pub fn settings(mut self, settings: Settings) -> Self {
        self.settings = settings;
        self
    }

    pub fn window(mut self, settings: window::Settings) -> Self {
        self.window = Some(settings);
        self
    }

    pub fn scale_factor(mut self, f: impl Fn(&State, window::Id) -> f32 + 'static) -> Self {
        self.scale_factor = Rc::new(f);
        self
    }

    pub fn presets(mut self, presets: impl IntoIterator<Item = Preset<State, Message>>) -> Self {
        self.presets = presets.into_iter().collect();
        self
    }
}

impl<State, Message, AppTheme, AppRenderer> Program
    for Application<State, Message, AppTheme, AppRenderer>
where
    State: 'static,
    Message: Send + 'static,
    AppTheme: theme::Base + Default,
    AppRenderer: Default + 'static,
{
    type State = State;
    type Message = Message;
    type Theme = AppTheme;
    type Renderer = AppRenderer;
    type Executor = DefaultExecutor;

    fn name() -> &'static str {
        "arkit"
    }

    fn settings(&self) -> Settings {
        self.settings.clone()
    }

    fn window(&self) -> Option<window::Settings> {
        self.window.clone()
    }

    fn boot(&self) -> (Self::State, Task<Self::Message>) {
        (self.boot)()
    }

    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
        (self.update)(state, message)
    }

    fn view(
        &self,
        state: &Self::State,
        _window: window::Id,
    ) -> Element<Self::Message, Self::Theme, Self::Renderer> {
        (self.view)(state)
    }

    fn title(&self, state: &Self::State, window: window::Id) -> String {
        (self.title)(state, window)
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        (self.subscription)(state)
    }

    fn theme(&self, state: &Self::State, window: window::Id) -> Option<Self::Theme> {
        (self.theme)(state, window)
    }

    fn style(&self, state: &Self::State, theme: &Self::Theme) -> theme::Style {
        (self.style)(state, theme)
    }

    fn scale_factor(&self, state: &Self::State, window: window::Id) -> f32 {
        (self.scale_factor)(state, window)
    }

    fn presets(&self) -> &[Preset<Self::State, Self::Message>] {
        self.presets.as_slice()
    }
}

#[doc(hidden)]
pub mod internal {
    pub use crate::{
        clear_ui_loop_effects, current_runtime, dispatch, global_dispatcher, queue_ui_loop,
        run_ui_loop_effects, set_current_runtime, set_dispatcher, set_global_dispatcher,
        set_ui_waker, with_dispatcher, with_global_dispatcher, GlobalRuntimeDispatcher,
        RuntimeDispatcher, RuntimeHandle,
    };
}

#[cfg(test)]
mod tests {
    use super::{Task, TaskAction};

    fn test_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test tokio runtime")
    }

    #[test]
    fn ready_tasks_produce_messages_in_batch_order() {
        let task = Task::batch([Task::done(1), Task::run(|| 2), Task::done(3)]);

        assert_eq!(task.into_messages(), vec![1, 2, 3]);
    }

    #[test]
    fn perform_maps_async_result_to_message() {
        let mut actions = Task::perform(async { 41 }, |value| value + 1).into_actions();
        assert_eq!(actions.len(), 1);

        let TaskAction::Future(future) = actions.remove(0) else {
            panic!("Task::perform should create a future action");
        };

        assert_eq!(test_runtime().block_on(future), 42);
    }

    #[test]
    fn map_applies_to_ready_and_async_actions() {
        let task = Task::batch([Task::done(1), Task::perform(async { 2 }, |value| value)])
            .map(|value| format!("message:{value}"));

        let mut ready = Vec::new();
        let mut futures = Vec::new();

        for action in task.into_actions() {
            match action {
                TaskAction::Ready(action) => action(&mut ready),
                TaskAction::Future(future) => futures.push(future),
            }
        }

        assert_eq!(ready, vec![String::from("message:1")]);
        assert_eq!(futures.len(), 1);
        assert_eq!(
            test_runtime().block_on(futures.remove(0)),
            String::from("message:2")
        );
    }

    #[test]
    #[should_panic(expected = "Task::into_messages cannot consume async tasks")]
    fn into_messages_rejects_async_tasks() {
        let _ = Task::perform(async { 1 }, |value| value).into_messages();
    }
}
