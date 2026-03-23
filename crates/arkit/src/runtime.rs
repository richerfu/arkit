use std::cell::{Cell, RefCell};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use napi_ohos::{Error, Result};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::handle::ArkUIHandle;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::component::built_in_component::Column;
use ohos_arkui_binding::component::root::RootNode;
use openharmony_ability::{Event as AbilityEvent, OpenHarmonyApp, OpenHarmonyWaker};

use crate::component::{mount_element, patch_element, MountedElement};
use crate::lifecycle::from_ability_event;
use crate::logging;
use crate::signal::{emit_lifecycle_event, set_scheduler, with_hook_state, HookState};
use crate::view::Element;
use crate::{column, text};

thread_local! {
    static RENDERER: RefCell<Option<Rc<dyn Fn()>>> = RefCell::new(None);
    static AFTER_MOUNT_EFFECTS: RefCell<Vec<Box<dyn FnOnce()>>> = RefCell::new(Vec::new());
    static UI_LOOP_EFFECTS: RefCell<Vec<Box<dyn FnOnce()>>> = RefCell::new(Vec::new());
    static UI_WAKER: RefCell<Option<OpenHarmonyWaker>> = RefCell::new(None);
    static RENDER_REQUESTED: Cell<bool> = const { Cell::new(false) };
    static CURRENT_APP: RefCell<Option<OpenHarmonyApp>> = RefCell::new(None);
}

pub struct Runtime {
    inner: Rc<RuntimeInner>,
}

struct RuntimeInner {
    app: OpenHarmonyApp,
    root: RefCell<RootNode>,
    render: Box<dyn Fn() -> Element>,
    hooks: Rc<RefCell<HookState>>,
    mounted: RefCell<Option<MountedApp>>,
    is_rendering: Cell<bool>,
    pending_render: Cell<bool>,
}

struct MountedApp {
    host_root: ArkUINode,
    app_root: MountedElement,
}

impl Runtime {
    pub fn new<F>(slot: ArkUIHandle, app: OpenHarmonyApp, render: F) -> Result<Self>
    where
        F: Fn() -> Element + 'static,
    {
        let runtime = Self {
            inner: Rc::new(RuntimeInner {
                app,
                root: RefCell::new(RootNode::new(slot)),
                render: Box::new(render),
                hooks: Rc::new(RefCell::new(HookState::new())),
                mounted: RefCell::new(None),
                is_rendering: Cell::new(false),
                pending_render: Cell::new(false),
            }),
        };

        logging::init_hilog();
        logging::info("runtime created");
        set_current_app(Some(runtime.inner.app.clone()));
        runtime.install_renderer();
        runtime.install_scheduler();
        runtime.install_ability_event_loop();
        runtime.inner.request_render()?;

        Ok(runtime)
    }

    pub fn app(&self) -> OpenHarmonyApp {
        self.inner.app.clone()
    }

    pub fn render(&self) -> Result<()> {
        self.inner.request_render()
    }

    pub fn unmount(&self) -> Result<()> {
        self.inner.unmount_root()
    }

    fn install_scheduler(&self) {
        let waker = self.inner.app.create_waker();
        set_ui_waker(Some(waker.clone()));
        set_scheduler(Some(Rc::new(move || {
            request_render_on_ui_loop(&waker);
        })));
    }

    fn install_renderer(&self) {
        let runtime = Rc::downgrade(&self.inner);
        set_renderer(Some(Rc::new(move || {
            if let Some(inner) = runtime.upgrade() {
                let _ = inner.request_render();
            }
        })));
    }

    fn install_ability_event_loop(&self) {
        self.inner.app.run_loop(move |event| match event {
            AbilityEvent::UserEvent => {
                run_ui_loop_effects();
                if take_render_requested() {
                    trigger_renderer();
                }
            }
            _ => {
                emit_lifecycle_event(from_ability_event(event));
            }
        });
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        set_scheduler(None);
        set_renderer(None);
        set_ui_waker(None);
        set_current_app(None);
        clear_ui_loop_effects();
        clear_after_mount_effects();
        clear_render_requested();
        self.inner.hooks.borrow_mut().cleanup_all();
        let _ = self.unmount();
    }
}

impl RuntimeInner {
    fn unmount_root(&self) -> Result<()> {
        let mounted = self.mounted.borrow_mut().take();
        let result = map_arkui_result(self.root.borrow_mut().unmount());
        if let Some(mounted) = mounted {
            mounted.app_root.cleanup_recursive();
        }
        result
    }

    fn request_render(&self) -> Result<()> {
        if self.is_rendering.get() {
            self.pending_render.set(true);
            return Ok(());
        }

        self.is_rendering.set(true);
        let result = self.render_once();
        self.is_rendering.set(false);

        if result.is_ok() && self.pending_render.replace(false) {
            self.request_render()?;
        }

        result
    }

    fn render_once(&self) -> Result<()> {
        clear_after_mount_effects();
        let next_tree = match catch_unwind(AssertUnwindSafe(|| {
            let hooks = self.hooks.clone();
            hooks.borrow_mut().reset_cursor();
            with_hook_state(hooks, || (self.render)())
        })) {
            Ok(tree) => tree,
            Err(_) => build_fallback_element("render panic".to_string()),
        };
        self.hooks.borrow_mut().finalize_render();
        if let Err(error) = self.commit_tree(next_tree) {
            logging::error(format!("commit tree failed: {error}"));
            clear_after_mount_effects();
            self.remount_tree(build_fallback_element(format!("commit failed: {error}")))?;
        }
        schedule_after_mount_effects();
        Ok(())
    }

    fn commit_tree(&self, next_tree: Element) -> Result<()> {
        let mounted_root = self.mounted.borrow().as_ref().map(|mounted| {
            (
                mounted.app_root.kind,
                mounted.app_root.key.as_deref().map(str::to_owned),
            )
        });
        let next_key = next_tree.key().map(str::to_owned);
        match mounted_root {
            None => self.remount_tree(next_tree),
            Some((kind, key)) if kind == next_tree.kind() && key == next_key => {
                self.patch_tree(next_tree)
            }
            Some(_) => self.remount_tree(next_tree),
        }
    }

    fn patch_tree(&self, next_tree: Element) -> Result<()> {
        let mut mounted = self.mounted.borrow_mut();
        let mounted = mounted
            .as_mut()
            .ok_or_else(|| Error::from_reason("missing mounted app".to_string()))?;
        let app_handle = mounted
            .host_root
            .children()
            .first()
            .cloned()
            .ok_or_else(|| Error::from_reason("host root missing app child".to_string()))?;
        let mut app_node = app_handle.borrow_mut();
        map_arkui_result(patch_element(
            next_tree,
            &mut app_node,
            &mut mounted.app_root,
        ))
    }

    fn remount_tree(&self, next_tree: Element) -> Result<()> {
        let (app_node, app_root) = map_arkui_result(mount_element(next_tree))?;
        let host_root = map_arkui_result(build_host_root(app_node))?;

        let previous = self.mounted.borrow_mut().take();
        let mut root = self.root.borrow_mut();
        let unmount_result = map_arkui_result(root.unmount());
        if let Some(previous) = previous {
            previous.app_root.cleanup_recursive();
        }
        let _ = unmount_result;
        map_arkui_result(root.mount(host_root.clone()))?;

        self.mounted.borrow_mut().replace(MountedApp {
            host_root,
            app_root,
        });
        Ok(())
    }
}

fn build_fallback_element(message: String) -> Element {
    column(vec![
        text("arkit runtime fallback").font_size(18.0).into(),
        text(message).font_size(14.0).into(),
    ])
}

fn build_host_root(app_root: ArkUINode) -> ArkUIResult<ArkUINode> {
    let mut host = Column::new()?;
    host.percent_width(1.0)?;
    host.percent_height(1.0)?;
    host.add_child(app_root)?;
    Ok(host.into())
}

fn map_arkui_result<T, E: ToString>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|error| {
        let reason = error.to_string();
        logging::error(format!("arkui runtime error: {reason}"));
        Error::from_reason(reason)
    })
}

fn set_renderer(renderer: Option<Rc<dyn Fn()>>) {
    RENDERER.with(|state| {
        state.replace(renderer);
    });
}

fn set_current_app(app: Option<OpenHarmonyApp>) {
    CURRENT_APP.with(|state| {
        state.replace(app);
    });
}

pub fn current_app() -> Option<OpenHarmonyApp> {
    CURRENT_APP.with(|state| state.borrow().clone())
}

pub fn queue_after_mount(effect: impl FnOnce() + 'static) {
    AFTER_MOUNT_EFFECTS.with(|state| {
        state.borrow_mut().push(Box::new(effect));
    });
}

pub fn queue_ui_loop(effect: impl FnOnce() + 'static) {
    UI_LOOP_EFFECTS.with(|state| {
        state.borrow_mut().push(Box::new(effect));
    });
    wake_ui_loop();
}

fn clear_after_mount_effects() {
    AFTER_MOUNT_EFFECTS.with(|state| {
        state.borrow_mut().clear();
    });
}

fn schedule_after_mount_effects() {
    let effects = AFTER_MOUNT_EFFECTS.with(|state| state.replace(Vec::new()));
    if effects.is_empty() {
        return;
    }

    UI_LOOP_EFFECTS.with(|state| {
        state.borrow_mut().extend(effects);
    });

    wake_ui_loop();
}

fn run_ui_loop_effects() {
    let effects = UI_LOOP_EFFECTS.with(|state| state.replace(Vec::new()));
    for effect in effects {
        effect();
    }
}

fn clear_ui_loop_effects() {
    UI_LOOP_EFFECTS.with(|state| {
        state.borrow_mut().clear();
    });
}

fn set_ui_waker(waker: Option<OpenHarmonyWaker>) {
    UI_WAKER.with(|state| {
        state.replace(waker);
    });
}

fn wake_ui_loop() {
    UI_WAKER.with(|state| {
        if let Some(waker) = state.borrow().as_ref() {
            waker.wake();
        }
    });
}

fn request_render_on_ui_loop(waker: &OpenHarmonyWaker) {
    RENDER_REQUESTED.with(|state| {
        state.set(true);
    });
    waker.wake();
}

fn take_render_requested() -> bool {
    RENDER_REQUESTED.with(|state| state.replace(false))
}

fn clear_render_requested() {
    RENDER_REQUESTED.with(|state| {
        state.set(false);
    });
}

fn trigger_renderer() {
    RENDERER.with(|state| {
        if let Some(renderer) = state.borrow().as_ref() {
            renderer();
        }
    });
}
