use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

thread_local! {
    static DISPATCHER: RefCell<Option<RuntimeDispatcher>> = RefCell::new(None);
}

pub type RuntimeDispatcher = Rc<dyn Fn(Box<dyn Any>)>;

#[derive(Debug, Clone, Copy, Default)]
pub struct Renderer;

#[derive(Debug, Clone, Default)]
pub struct Theme;

#[derive(Default)]
pub struct Settings;

pub trait Program {
    type State: 'static;
    type Message: Clone + 'static;

    fn title(&self, _state: &Self::State) -> String {
        String::new()
    }

    fn boot(&self) -> (Self::State, Task<Self::Message>);

    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message>;

    type View<'a>
    where
        Self: 'a;

    fn view<'a>(&'a self, state: &'a Self::State) -> Self::View<'a>;

    fn subscription(&self, _state: &Self::State) -> Subscription<Self::Message> {
        Subscription::none()
    }
}

pub struct Application<State, Message, View, Update, Boot>
where
    Message: Clone + 'static,
{
    title: String,
    boot: Boot,
    update: Update,
    view: View,
    _state: PhantomData<State>,
    _message: PhantomData<Message>,
}

pub fn application<State, Message, Update, View>(
    title: impl Into<String>,
    update: Update,
    view: View,
) -> Application<State, Message, View, Update, fn() -> (State, Task<Message>)>
where
    Message: Clone + 'static,
    Update: Fn(&mut State, Message) -> Task<Message> + 'static,
    View: 'static,
{
    Application {
        title: title.into(),
        boot: panic_boot::<State, Message>,
        update,
        view,
        _state: PhantomData,
        _message: PhantomData,
    }
}

fn panic_boot<State, Message>() -> (State, Task<Message>)
where
    Message: Clone + 'static,
{
    panic!("application boot function not configured; call `.run_with(...)` before mounting");
}

impl<State, Message, View, Update, Boot> Application<State, Message, View, Update, Boot>
where
    Message: Clone + 'static,
{
    pub fn run_with<NextBoot>(
        self,
        boot: NextBoot,
    ) -> Application<State, Message, View, Update, NextBoot>
    where
        NextBoot: Fn() -> (State, Task<Message>) + 'static,
    {
        Application {
            title: self.title,
            boot,
            update: self.update,
            view: self.view,
            _state: PhantomData,
            _message: PhantomData,
        }
    }
}

impl<State, Message, View, Update, Boot, Element> Program
    for Application<State, Message, View, Update, Boot>
where
    State: 'static,
    Message: Clone + 'static,
    Boot: Fn() -> (State, Task<Message>) + 'static,
    Update: Fn(&mut State, Message) -> Task<Message> + 'static,
    View: Fn(&State) -> Element + 'static,
    Element: 'static,
{
    type State = State;
    type Message = Message;
    type View<'a>
        = Element
    where
        Self: 'a;

    fn title(&self, _state: &Self::State) -> String {
        self.title.clone()
    }

    fn boot(&self) -> (Self::State, Task<Self::Message>) {
        (self.boot)()
    }

    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
        (self.update)(state, message)
    }

    fn view<'a>(&'a self, state: &'a Self::State) -> Self::View<'a> {
        (self.view)(state)
    }
}

#[derive(Clone)]
pub struct Task<Message> {
    messages: Vec<Message>,
}

impl<Message> Task<Message> {
    pub fn none() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn done(message: Message) -> Self {
        Self {
            messages: vec![message],
        }
    }

    pub fn batch(tasks: impl IntoIterator<Item = Self>) -> Self {
        let mut messages = Vec::new();
        for task in tasks {
            messages.extend(task.messages);
        }
        Self { messages }
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn into_messages(self) -> Vec<Message> {
        self.messages
    }
}

pub struct Subscription<Message> {
    _message: PhantomData<Message>,
}

impl<Message> Subscription<Message> {
    pub fn none() -> Self {
        Self {
            _message: PhantomData,
        }
    }
}

pub fn dispatch<Message>(message: Message)
where
    Message: 'static,
{
    DISPATCHER.with(|dispatcher| {
        let dispatcher = dispatcher.borrow().as_ref().cloned();
        let Some(dispatcher) = dispatcher else {
            panic!("arkit::dispatch called without an active runtime dispatcher");
        };
        dispatcher(Box::new(message));
    });
}

pub fn set_dispatcher(next: Option<RuntimeDispatcher>) {
    DISPATCHER.with(|dispatcher| {
        dispatcher.replace(next);
    });
}

pub fn with_dispatcher<R>(dispatcher: RuntimeDispatcher, f: impl FnOnce() -> R) -> R {
    let previous = DISPATCHER.with(|state| state.replace(Some(dispatcher)));
    let result = f();
    DISPATCHER.with(|state| {
        state.replace(previous);
    });
    result
}
