use std::cell::{Cell, RefCell};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use napi_ohos::{Error, Result};
use ohos_arkui_binding::common::handle::ArkUIHandle;
use ohos_arkui_binding::component::root::RootNode;
use openharmony_ability::{Event as AbilityEvent, OpenHarmonyApp};

use crate::component::build_element;
use crate::lifecycle::from_ability_event;
use crate::signal::{emit_lifecycle_event, set_scheduler, with_hook_state, HookState};
use crate::view::Element;
use crate::{column, text};

thread_local! {
    static RENDERER: RefCell<Option<Rc<dyn Fn()>>> = RefCell::new(None);
}

pub struct Runtime {
    inner: Rc<RuntimeInner>,
}

struct RuntimeInner {
    app: OpenHarmonyApp,
    root: RefCell<RootNode>,
    render: Box<dyn Fn() -> Element>,
    hooks: Rc<RefCell<HookState>>,
    is_rendering: Cell<bool>,
    pending_render: Cell<bool>,
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
                is_rendering: Cell::new(false),
                pending_render: Cell::new(false),
            }),
        };

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
        map_arkui_result(self.inner.root.borrow_mut().unmount())
    }

    fn install_scheduler(&self) {
        let waker = self.inner.app.create_waker();
        set_scheduler(Some(Rc::new(move || {
            waker.wake();
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
                trigger_renderer();
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
        self.inner.hooks.borrow_mut().cleanup_all();
        let _ = self.unmount();
    }
}

impl RuntimeInner {
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
        let next_tree = match catch_unwind(AssertUnwindSafe(|| {
            let hooks = self.hooks.clone();
            hooks.borrow_mut().reset_cursor();
            with_hook_state(hooks, || (self.render)())
        })) {
            Ok(tree) => tree,
            Err(_) => build_fallback_element("render panic".to_string()),
        };
        self.hooks.borrow_mut().finalize_render();
        let next_root = match map_arkui_result(build_element(next_tree)) {
            Ok(node) => node,
            Err(error) => map_arkui_result(build_element(build_fallback_element(format!(
                "build failed: {error}"
            ))))?,
        };
        let mut root = self.root.borrow_mut();
        let _ = map_arkui_result(root.unmount());
        if let Err(error) = map_arkui_result(root.mount(next_root)) {
            let fallback = map_arkui_result(build_element(build_fallback_element(format!(
                "mount failed: {error}"
            ))))?;
            let _ = map_arkui_result(root.unmount());
            map_arkui_result(root.mount(fallback))?;
        }
        Ok(())
    }
}

fn build_fallback_element(message: String) -> Element {
    column(vec![
        text("arkit runtime fallback").font_size(18.0).into(),
        text(message).font_size(14.0).into(),
    ])
}

fn map_arkui_result<T, E: ToString>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|error| Error::from_reason(error.to_string()))
}

fn set_renderer(renderer: Option<Rc<dyn Fn()>>) {
    RENDERER.with(|state| {
        state.replace(renderer);
    });
}

fn trigger_renderer() {
    RENDERER.with(|state| {
        if let Some(renderer) = state.borrow().as_ref() {
            renderer();
        }
    });
}
