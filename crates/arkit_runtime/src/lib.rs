use std::any::Any;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, LazyLock, Mutex};

pub use arkit_core::theme;
pub use arkit_core::{window, Theme};
pub use arkit_futures::{Subscription, SubscriptionHandle};

pub type Element<Message, AppTheme = Theme, AppRenderer = ()> =
    arkit_core::Element<'static, Message, AppTheme, AppRenderer>;

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

pub enum BackPressDecision<Message> {
    PassThrough,
    Intercept(Task<Message>),
}

impl<Message: Send + 'static> BackPressDecision<Message> {
    pub fn pass_through() -> Self {
        Self::PassThrough
    }

    pub fn handled() -> Self {
        Self::Intercept(Task::none())
    }

    pub fn task(task: Task<Message>) -> Self {
        Self::Intercept(task)
    }

    pub fn message(message: Message) -> Self {
        Self::Intercept(Task::done(message))
    }

    pub fn is_intercepted(&self) -> bool {
        matches!(self, Self::Intercept(_))
    }
}

pub trait Program: Sized {
    type State: 'static;
    type Message: Send + 'static;
    type Theme: theme::Base + Default;
    type Renderer: Default + 'static;

    fn boot(&self) -> (Self::State, Task<Self::Message>);
    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message>;
    fn view(
        &self,
        state: &Self::State,
        window: window::Id,
    ) -> Element<Self::Message, Self::Theme, Self::Renderer>;

    fn subscription(&self, _state: &Self::State) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn back_press(&self, _state: &Self::State) -> BackPressDecision<Self::Message> {
        BackPressDecision::PassThrough
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
    subscription: Rc<dyn Fn(&State) -> Subscription<Message>>,
    back_press: Rc<dyn Fn(&State) -> BackPressDecision<Message>>,
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
        subscription: Rc::new(|_| Subscription::none()),
        back_press: Rc::new(|_| BackPressDecision::PassThrough),
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

    pub fn subscription(mut self, f: impl Fn(&State) -> Subscription<Message> + 'static) -> Self {
        self.subscription = Rc::new(f);
        self
    }

    pub fn on_back_press(
        mut self,
        f: impl Fn(&State) -> BackPressDecision<Message> + 'static,
    ) -> Self {
        self.back_press = Rc::new(f);
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

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        (self.subscription)(state)
    }

    fn back_press(&self, state: &Self::State) -> BackPressDecision<Self::Message> {
        (self.back_press)(state)
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
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::{application, BackPressDecision, Program, Subscription, Task, TaskAction, Theme};

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

    #[test]
    fn back_press_decision_constructors_match_expected_behavior() {
        assert!(!BackPressDecision::<i32>::pass_through().is_intercepted());
        assert!(BackPressDecision::<i32>::handled().is_intercepted());

        let BackPressDecision::Intercept(task) = BackPressDecision::message(7) else {
            panic!("message decision should intercept");
        };
        assert_eq!(task.into_messages(), vec![7]);

        let task = Task::done(9);
        let BackPressDecision::Intercept(task) = BackPressDecision::task(task) else {
            panic!("task decision should intercept");
        };
        assert_eq!(task.into_messages(), vec![9]);
    }

    #[test]
    fn program_default_back_press_passes_through() {
        struct TestProgram;

        impl Program for TestProgram {
            type State = ();
            type Message = ();
            type Theme = Theme;
            type Renderer = ();

            fn boot(&self) -> (Self::State, Task<Self::Message>) {
                ((), Task::none())
            }

            fn update(
                &self,
                _state: &mut Self::State,
                _message: Self::Message,
            ) -> Task<Self::Message> {
                Task::none()
            }

            fn view(
                &self,
                _state: &Self::State,
                _window: super::window::Id,
            ) -> super::Element<Self::Message, Self::Theme, Self::Renderer> {
                unreachable!("view is not used by this test")
            }
        }

        assert!(!TestProgram.back_press(&()).is_intercepted());
    }

    #[test]
    fn application_back_press_handler_overrides_default() {
        let app = application::<_, _, _, _, _, Theme, ()>(
            || 3,
            |_state, _message: i32| Task::none(),
            |_state| unreachable!("view is not used by this test"),
        )
        .on_back_press(|state| BackPressDecision::message(*state + 4));

        let BackPressDecision::Intercept(task) = Program::back_press(&app, &3) else {
            panic!("custom back press handler should intercept");
        };
        assert_eq!(task.into_messages(), vec![7]);
    }

    #[test]
    fn subscriptions_emit_synchronously_in_batch_order() {
        fn first() -> [i32; 2] {
            [1, 2]
        }

        fn second(seed: &i32) -> [i32; 1] {
            [*seed]
        }

        let subscription =
            Subscription::batch([Subscription::run(first), Subscription::run_with(3, second)])
                .map(|value| format!("message:{value}"));

        assert_eq!(subscription.units(), 2);

        let emitted = Rc::new(RefCell::new(Vec::new()));
        let mut handles = Vec::new();
        for recipe in subscription.into_recipes() {
            let emitted = emitted.clone();
            handles.push((recipe.start)(Rc::new(move |message| {
                emitted.borrow_mut().push(message);
            })));
        }

        assert_eq!(
            emitted.borrow().as_slice(),
            ["message:1", "message:2", "message:3"]
        );
        drop(handles);
    }
}
