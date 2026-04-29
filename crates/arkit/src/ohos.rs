use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::sync::mpsc;
use std::thread;

use arkit_core::{theme, window};
use arkit_runtime::{
    internal::{
        clear_ui_loop_effects, run_ui_loop_effects, set_current_runtime, set_dispatcher,
        set_global_dispatcher, set_ui_waker, GlobalRuntimeDispatcher, RuntimeDispatcher,
        RuntimeHandle,
    },
    BackPressDecision, Program, SubscriptionHandle, Task, TaskAction,
};
use arkit_widget::{mount, patch, realize_attached_mount, Element, MountedNode, Renderer};
use napi_ohos::{Error, Result};
use ohos_arkui_binding::common::handle::ArkUIHandle;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::root::RootNode;
use openharmony_ability::{Event as AbilityEvent, OpenHarmonyApp, OpenHarmonyWaker};

pub use napi_derive_ohos;
pub use napi_ohos;
pub use ohos_arkui_binding;
pub use openharmony_ability;

struct MountedRoot {
    node: ArkUINode,
    mounted: MountedNode,
}

struct RootRuntime<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: theme::Base + Default + 'static,
{
    app: OpenHarmonyApp,
    root: RefCell<RootNode>,
    mounted: RefCell<Option<MountedRoot>>,
    render: Option<Rc<dyn Fn() -> Element<Message, AppTheme>>>,
}

impl<Message, AppTheme> RootRuntime<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: theme::Base + Default + 'static,
{
    fn new<F>(slot: ArkUIHandle, app: OpenHarmonyApp, render: F) -> Result<Self>
    where
        F: Fn() -> Element<Message, AppTheme> + 'static,
    {
        let runtime = Self {
            app,
            root: RefCell::new(RootNode::new(slot)),
            mounted: RefCell::new(None),
            render: Some(Rc::new(render)),
        };
        runtime.mount_root((runtime.render.as_ref().expect("render closure"))())?;
        Ok(runtime)
    }

    fn new_static(
        slot: ArkUIHandle,
        app: OpenHarmonyApp,
        tree: Element<Message, AppTheme>,
    ) -> Result<Self> {
        let runtime = Self {
            app,
            root: RefCell::new(RootNode::new(slot)),
            mounted: RefCell::new(None),
            render: None,
        };
        runtime.mount_root(tree)?;
        Ok(runtime)
    }

    fn rerender<F>(&self, render: F) -> Result<()>
    where
        F: FnOnce() -> Element<Message, AppTheme>,
    {
        self.patch_root(render())
    }

    fn request_rerender(&self) -> Result<()> {
        let Some(render) = self.render.clone() else {
            return Ok(());
        };
        self.rerender(move || render())
    }

    fn mount_root(&self, tree: Element<Message, AppTheme>) -> Result<()> {
        let (mut node, mut mounted) = map_arkui_result(mount(tree))?;
        if let Some(previous) = self.mounted.borrow_mut().take() {
            let _ = map_arkui_result(self.root.borrow_mut().unmount());
            previous.mounted.cleanup_recursive();
        }
        map_arkui_result(self.root.borrow_mut().mount(node.clone()))?;
        map_arkui_result(realize_attached_mount(&mut node, &mut mounted))?;
        self.mounted
            .borrow_mut()
            .replace(MountedRoot { node, mounted });
        Ok(())
    }

    fn patch_root(&self, tree: Element<Message, AppTheme>) -> Result<()> {
        let mut mounted = self.mounted.borrow_mut();
        let Some(current) = mounted.as_mut() else {
            drop(mounted);
            return self.mount_root(tree);
        };
        let patch_result = patch(tree, &mut current.node, &mut current.mounted);
        if patch_result.is_err() {
            drop(mounted);
            return self.mount_root((self.render.as_ref().expect("render closure"))());
        }
        map_arkui_result(patch_result)
    }

    fn unmount(&self) -> Result<()> {
        let mounted = self.mounted.borrow_mut().take();
        if let Some(mounted) = mounted {
            let _ = map_arkui_result(self.root.borrow_mut().unmount());
            mounted.mounted.cleanup_recursive();
        } else {
            let _ = map_arkui_result(self.root.borrow_mut().unmount());
        }
        Ok(())
    }
}

fn map_arkui_result<T, E: ToString>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|error| Error::from_reason(error.to_string()))
}

pub trait MountedEntryHandle {
    fn unmount(&self) -> Result<()>;
}

pub trait EntryPoint {
    fn mount(self, slot: ArkUIHandle, app: OpenHarmonyApp) -> Result<Box<dyn MountedEntryHandle>>;
}

struct StaticRuntimeHandle<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: theme::Base + Default + 'static,
{
    runtime: RootRuntime<Message, AppTheme>,
}

impl<Message, AppTheme> MountedEntryHandle for StaticRuntimeHandle<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: theme::Base + Default + 'static,
{
    fn unmount(&self) -> Result<()> {
        self.runtime.unmount()
    }
}

pub struct ApplicationRuntime<P>
where
    P: Program<Renderer = Renderer> + 'static,
    P::Theme: theme::Base + Default + 'static,
{
    runtime: Rc<RootRuntime<P::Message, P::Theme>>,
    _application: Rc<ApplicationState<P>>,
    _dispatcher: RuntimeDispatcher,
    _program: Rc<P>,
}

struct TaskRunner<Message>
where
    Message: Send + 'static,
{
    task_sender: tokio::sync::mpsc::UnboundedSender<BoxedTaskFuture<Message>>,
    _worker: thread::JoinHandle<()>,
    sender: mpsc::Sender<Message>,
    waker: OpenHarmonyWaker,
}

type BoxedTaskFuture<Message> = Pin<Box<dyn Future<Output = Message> + Send + 'static>>;

impl<Message> TaskRunner<Message>
where
    Message: Send + 'static,
{
    fn new(sender: mpsc::Sender<Message>, waker: OpenHarmonyWaker) -> Result<Self> {
        let (task_sender, mut task_receiver) =
            tokio::sync::mpsc::unbounded_channel::<BoxedTaskFuture<Message>>();
        let background_sender = sender.clone();
        let background_waker = waker.clone();
        let worker = thread::Builder::new()
            .name(String::from("arkit-task"))
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("arkit task runtime");

                runtime.block_on(async move {
                    while let Some(future) = task_receiver.recv().await {
                        let sender = background_sender.clone();
                        let waker = background_waker.clone();
                        tokio::spawn(async move {
                            let message = future.await;
                            if sender.send(message).is_ok() {
                                waker.wake();
                            }
                        });
                    }
                });
            })
            .map_err(|error| Error::from_reason(error.to_string()))?;

        Ok(Self {
            task_sender,
            _worker: worker,
            sender,
            waker,
        })
    }

    fn run(&self, task: Task<Message>) -> Vec<Message> {
        let mut ready = Vec::new();

        for action in task.into_actions() {
            match action {
                TaskAction::Ready(action) => action(&mut ready),
                TaskAction::Future(future) => {
                    if let Err(error) = self.task_sender.send(future) {
                        let sender = self.sender.clone();
                        let waker = self.waker.clone();
                        thread::spawn(move || {
                            let runtime = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .expect("arkit fallback task runtime");
                            let message = runtime.block_on(error.0);
                            if sender.send(message).is_ok() {
                                waker.wake();
                            }
                        });
                    }
                }
            }
        }

        ready
    }
}

struct ApplicationState<P>
where
    P: Program<Renderer = Renderer> + 'static,
{
    program: Rc<P>,
    state: Rc<RefCell<P::State>>,
    handling: Cell<bool>,
    redraw_pending: Cell<bool>,
    pending: RefCell<Vec<P::Message>>,
    background_sender: mpsc::Sender<P::Message>,
    background_receiver: RefCell<mpsc::Receiver<P::Message>>,
    task_runner: RefCell<Option<Rc<TaskRunner<P::Message>>>>,
    emitter: RefCell<Option<Rc<dyn Fn(P::Message)>>>,
    subscriptions: RefCell<BTreeMap<String, SubscriptionHandle>>,
}

impl<P> ApplicationState<P>
where
    P: Program<Renderer = Renderer> + 'static,
    P::Theme: theme::Base + Default + 'static,
{
    fn new(program: Rc<P>) -> (Rc<Self>, Task<P::Message>) {
        let (state, boot_task) = program.boot();
        let (background_sender, background_receiver) = mpsc::channel();
        let app = Rc::new(Self {
            program,
            state: Rc::new(RefCell::new(state)),
            handling: Cell::new(false),
            redraw_pending: Cell::new(false),
            pending: RefCell::new(Vec::new()),
            background_sender,
            background_receiver: RefCell::new(background_receiver),
            task_runner: RefCell::new(None),
            emitter: RefCell::new(None),
            subscriptions: RefCell::new(BTreeMap::new()),
        });
        (app, boot_task)
    }

    fn render(&self) -> Element<P::Message, P::Theme> {
        {
            let state = self.state.borrow();
            self.program.view(&state, window::Id::MAIN)
        }
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

    fn set_task_runner(&self, waker: OpenHarmonyWaker) -> Result<()> {
        let runner = TaskRunner::new(self.background_sender.clone(), waker)?;
        self.task_runner.replace(Some(Rc::new(runner)));
        Ok(())
    }

    fn run_task(&self, task: Task<P::Message>) -> Vec<P::Message> {
        let runner = self
            .task_runner
            .borrow()
            .as_ref()
            .cloned()
            .expect("arkit runtime task executed before task runner was installed");
        runner.run(task)
    }

    fn drain_background(&self) -> bool {
        let mut messages = Vec::new();
        {
            let receiver = self.background_receiver.borrow_mut();
            while let Ok(message) = receiver.try_recv() {
                messages.push(message);
            }
        }
        self.enqueue_many(messages)
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
            self.sync_subscription();
            let messages = self.run_task(task);
            self.pending.borrow_mut().extend(messages.into_iter().rev());
        }

        self.handling.set(false);
        handled_any
    }

    fn set_emitter(&self, emitter: Rc<dyn Fn(P::Message)>) {
        self.emitter.replace(Some(emitter));
        self.sync_subscription();
    }

    fn sync_subscription(&self) {
        let Some(emitter) = self.emitter.borrow().as_ref().cloned() else {
            return;
        };

        let next = {
            let state = self.state.borrow();
            self.program.subscription(&state)
        };

        let mut next_ids = Vec::new();
        let mut pending = Vec::new();
        {
            let current = self.subscriptions.borrow();
            for recipe in next.into_recipes() {
                let id = recipe.id.clone();
                next_ids.push(id.clone());
                if current.contains_key(&id) {
                    continue;
                }
                pending.push(recipe);
            }
        }

        self.subscriptions
            .borrow_mut()
            .retain(|id, _| next_ids.iter().any(|next_id| next_id == id));

        let mut current = self.subscriptions.borrow_mut();
        for recipe in pending {
            let id = recipe.id.clone();
            let handle = (recipe.start)(emitter.clone());
            current.insert(id, handle);
        }
    }

    fn schedule_redraw(self: &Rc<Self>, runtime: Weak<RootRuntime<P::Message, P::Theme>>) {
        if self.redraw_pending.replace(true) {
            return;
        }

        let application = Rc::downgrade(self);
        arkit_runtime::internal::queue_ui_loop(move || {
            let Some(application) = application.upgrade() else {
                return;
            };
            application.redraw_pending.set(false);

            if let Some(runtime) = runtime.upgrade() {
                let _ = runtime.request_rerender();
            }
        });
    }
}

impl<P> ApplicationRuntime<P>
where
    P: Program<Renderer = Renderer> + 'static,
    P::Theme: theme::Base + Default + 'static,
{
    pub fn new(slot: ArkUIHandle, app: OpenHarmonyApp, program: P) -> Result<Self> {
        let program = Rc::new(program);
        let (application, boot_task) = ApplicationState::new(program.clone());
        let waker = app.create_waker();
        application.set_task_runner(waker.clone())?;
        set_ui_waker(Some(Rc::new({
            let waker = waker.clone();
            move || waker.wake()
        })));
        let runtime_slot = Rc::new(RefCell::new(
            None::<Weak<RootRuntime<P::Message, P::Theme>>>,
        ));
        let event_application = application.clone();
        let event_runtime = runtime_slot.clone();
        app.run_loop(move |event| {
            if matches!(event, AbilityEvent::UserEvent) {
                run_ui_loop_effects();
                if event_application.drain_background() {
                    if let Some(runtime) = event_runtime.borrow().clone() {
                        event_application.schedule_redraw(runtime);
                    }
                }
            }
        });
        let back_application = application.clone();
        let back_runtime = runtime_slot.clone();
        app.on_back_press_intercept(move || {
            let decision = {
                let state = back_application.state.borrow();
                back_application.program.back_press(&state)
            };

            match decision {
                BackPressDecision::PassThrough => false,
                BackPressDecision::Intercept(task) => {
                    let messages = back_application.run_task(task);
                    if back_application.enqueue_many(messages) {
                        if let Some(runtime) = back_runtime.borrow().clone() {
                            back_application.schedule_redraw(runtime);
                        }
                    }
                    true
                }
            }
        });
        let runtime_for_dispatch = runtime_slot.clone();
        let app_for_dispatch = Rc::downgrade(&application);
        let emitter: Rc<dyn Fn(P::Message)> = Rc::new(move |message| {
            let Some(application) = Weak::upgrade(&app_for_dispatch) else {
                return;
            };

            if application.enqueue(message) {
                if let Some(runtime) = runtime_for_dispatch.borrow().clone() {
                    application.schedule_redraw(runtime);
                }
            }
        });
        let dispatcher_emitter = emitter.clone();
        let dispatcher: RuntimeDispatcher = Rc::new(move |message: Box<dyn Any + Send>| {
            let message = message
                .downcast::<P::Message>()
                .expect("dispatch received the wrong message type");
            dispatcher_emitter(*message);
        });
        let background_sender = application.background_sender.clone();
        let background_waker = waker.clone();
        let global_dispatcher: GlobalRuntimeDispatcher =
            std::sync::Arc::new(move |message: Box<dyn Any + Send>| {
                let message = message
                    .downcast::<P::Message>()
                    .expect("dispatch received the wrong message type");
                if background_sender.send(*message).is_ok() {
                    background_waker.wake();
                }
            });
        application.set_emitter(emitter);
        set_dispatcher(Some(dispatcher.clone()));
        set_global_dispatcher(Some(global_dispatcher));
        let render_state = application.clone();
        let runtime = match RootRuntime::new(slot, app.clone(), move || render_state.render()) {
            Ok(runtime) => runtime,
            Err(error) => {
                app.on_back_press_intercept(|| false);
                set_dispatcher(None);
                set_global_dispatcher(None);
                set_ui_waker(None);
                return Err(error);
            }
        };
        let runtime = Rc::new(runtime);
        runtime_slot.replace(Some(Rc::downgrade(&runtime)));
        set_current_runtime(Some(RuntimeHandle::new({
            let runtime = runtime.clone();
            move || {
                let _ = runtime.request_rerender();
            }
        })));
        if application.enqueue_many(application.run_task(boot_task)) {
            application.schedule_redraw(Rc::downgrade(&runtime));
        }

        Ok(Self {
            runtime,
            _application: application,
            _dispatcher: dispatcher,
            _program: program,
        })
    }
}

impl<P> MountedEntryHandle for ApplicationRuntime<P>
where
    P: Program<Renderer = Renderer> + 'static,
    P::Theme: theme::Base + Default + 'static,
{
    fn unmount(&self) -> Result<()> {
        self.runtime.unmount()
    }
}

impl<P> Drop for ApplicationRuntime<P>
where
    P: Program<Renderer = Renderer> + 'static,
    P::Theme: theme::Base + Default + 'static,
{
    fn drop(&mut self) {
        self.runtime.app.on_back_press_intercept(|| false);
        set_current_runtime(None);
        set_dispatcher(None);
        set_global_dispatcher(None);
        set_ui_waker(None);
        clear_ui_loop_effects();
    }
}

pub fn mount_application<P>(
    slot: ArkUIHandle,
    app: OpenHarmonyApp,
    program: P,
) -> Result<ApplicationRuntime<P>>
where
    P: Program<Renderer = Renderer> + 'static,
    P::Theme: theme::Base + Default + 'static,
{
    ApplicationRuntime::new(slot, app, program)
}

pub fn mount_entry(
    slot: ArkUIHandle,
    app: OpenHarmonyApp,
    entry: impl EntryPoint,
) -> Result<Box<dyn MountedEntryHandle>> {
    entry.mount(slot, app)
}

impl<P> EntryPoint for P
where
    P: Program<Renderer = Renderer> + 'static,
    P::Theme: theme::Base + Default + 'static,
{
    fn mount(self, slot: ArkUIHandle, app: OpenHarmonyApp) -> Result<Box<dyn MountedEntryHandle>> {
        Ok(Box::new(mount_application(slot, app, self)?))
    }
}
