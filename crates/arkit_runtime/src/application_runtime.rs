use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use napi_ohos::Result;
use ohos_arkui_binding::common::handle::ArkUIHandle;
use openharmony_ability::OpenHarmonyApp;

use crate::runtime::Runtime;
use crate::Element;
use crate::{set_dispatcher, Program, RuntimeDispatcher};

pub struct ApplicationRuntime<P>
where
    P: Program + 'static,
    for<'a> P::View<'a>: Into<Element>,
{
    runtime: Runtime,
    _dispatcher: RuntimeDispatcher,
    _program: Rc<P>,
}

struct ApplicationState<P>
where
    P: Program + 'static,
{
    program: Rc<P>,
    state: Rc<RefCell<P::State>>,
    handling: Cell<bool>,
    pending: RefCell<Vec<P::Message>>,
}

impl<P> ApplicationState<P>
where
    P: Program + 'static,
    for<'a> P::View<'a>: Into<Element>,
{
    fn new(program: Rc<P>) -> Rc<Self> {
        let (state, boot_task) = program.boot();
        let app = Rc::new(Self {
            program,
            state: Rc::new(RefCell::new(state)),
            handling: Cell::new(false),
            pending: RefCell::new(Vec::new()),
        });
        app.enqueue_many(boot_task.into_messages());
        app
    }

    fn render(&self) -> Element {
        let state = self.state.borrow();
        self.program.view(&state).into()
    }

    fn enqueue(&self, message: P::Message) -> bool {
        self.pending.borrow_mut().push(message);
        self.flush()
    }

    fn enqueue_many(&self, messages: Vec<P::Message>) -> bool {
        if messages.is_empty() {
            return false;
        }
        self.pending.borrow_mut().extend(messages);
        self.flush()
    }

    fn flush(&self) -> bool {
        if self.handling.replace(true) {
            return false;
        }

        let mut handled_any = false;
        loop {
            let Some(message) = ({ self.pending.borrow_mut().pop() }) else {
                break;
            };
            handled_any = true;
            let task = {
                let mut state = self.state.borrow_mut();
                self.program.update(&mut state, message)
            };
            self.pending
                .borrow_mut()
                .extend(task.into_messages().into_iter().rev());
        }

        self.handling.set(false);
        handled_any
    }
}

impl<P> ApplicationRuntime<P>
where
    P: Program + 'static,
    for<'a> P::View<'a>: Into<Element>,
{
    pub fn new(slot: ArkUIHandle, app: OpenHarmonyApp, program: P) -> Result<Self> {
        let program = Rc::new(program);
        let application = ApplicationState::new(program.clone());
        let render_app = application.clone();
        let runtime = Runtime::new(slot, app, move || render_app.render())?;
        let runtime_for_dispatch = runtime.clone();
        let dispatcher_app = application.clone();
        let dispatcher: RuntimeDispatcher = Rc::new(move |message: Box<dyn Any>| {
            let message = message
                .downcast::<P::Message>()
                .expect("dispatch received a message with the wrong concrete type");
            if dispatcher_app.enqueue(*message) {
                let _ = runtime_for_dispatch.rerender({
                    let render_app = dispatcher_app.clone();
                    move || render_app.render()
                });
            }
        });
        set_dispatcher(Some(dispatcher.clone()));

        Ok(Self {
            runtime,
            _dispatcher: dispatcher,
            _program: program,
        })
    }

    pub fn unmount(&self) -> Result<()> {
        self.runtime.unmount()
    }
}

impl<P> Drop for ApplicationRuntime<P>
where
    P: Program + 'static,
    for<'a> P::View<'a>: Into<Element>,
{
    fn drop(&mut self) {
        set_dispatcher(None);
    }
}

pub fn mount_application<P>(
    slot: ArkUIHandle,
    app: OpenHarmonyApp,
    program: P,
) -> Result<ApplicationRuntime<P>>
where
    P: Program + 'static,
    for<'a> P::View<'a>: Into<Element>,
{
    ApplicationRuntime::new(slot, app, program)
}
