use std::cell::RefCell;
use std::rc::Rc;

use napi_ohos::{Error, Result};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::handle::ArkUIHandle;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use ohos_arkui_binding::component::root::RootNode;
use openharmony_ability::{Event as AbilityEvent, OpenHarmonyApp, OpenHarmonyWaker};

use crate::component::{mount_element, MountedElement};
use crate::logging;
use crate::owner::{with_owner, Owner};
use crate::portal::{
    build_portal_host, set_current_portal_host, with_current_portal_host, PortalHostHandle,
};
use crate::view::Element;
use crate::{column, text};

thread_local! {
    static AFTER_MOUNT_EFFECTS: RefCell<Vec<Box<dyn FnOnce()>>> = RefCell::new(Vec::new());
    static UI_LOOP_EFFECTS: RefCell<Vec<Box<dyn FnOnce()>>> = RefCell::new(Vec::new());
    static UI_WAKER: RefCell<Option<OpenHarmonyWaker>> = RefCell::new(None);
    static CURRENT_APP: RefCell<Option<OpenHarmonyApp>> = RefCell::new(None);
}

pub struct Runtime {
    inner: Rc<RuntimeInner>,
}

struct RuntimeInner {
    app: OpenHarmonyApp,
    root: RefCell<RootNode>,
    mounted: RefCell<Option<MountedApp>>,
    /// Root owner for the entire application's reactive tree.
    root_owner: Rc<Owner>,
}

#[allow(dead_code)]
struct MountedApp {
    host_root: ArkUINode,
    app_root: MountedElement,
    portal_host: PortalHostHandle,
}

impl Runtime {
    /// Create a new runtime. The render function `render` is called **once** to build
    /// the initial UI tree. All subsequent updates are driven by reactive signals.
    pub fn new<F>(slot: ArkUIHandle, app: OpenHarmonyApp, render: F) -> Result<Self>
    where
        F: FnOnce() -> Element + 'static,
    {
        let root_owner = Owner::new_root();
        let runtime = Self {
            inner: Rc::new(RuntimeInner {
                app,
                root: RefCell::new(RootNode::new(slot)),
                mounted: RefCell::new(None),
                root_owner: root_owner.clone(),
            }),
        };

        logging::init_hilog();
        logging::info("runtime created");
        set_current_app(Some(runtime.inner.app.clone()));

        let waker = runtime.inner.app.create_waker();
        set_ui_waker(Some(waker));

        runtime.install_ability_event_loop();

        // Render once inside the root owner
        runtime.inner.mount_root(root_owner, render)?;

        Ok(runtime)
    }

    pub fn app(&self) -> OpenHarmonyApp {
        self.inner.app.clone()
    }

    pub fn unmount(&self) -> Result<()> {
        self.inner.unmount_root()
    }

    fn install_ability_event_loop(&self) {
        self.inner.app.run_loop(move |event| match event {
            AbilityEvent::UserEvent => {
                run_ui_loop_effects();
            }
            _ => {}
        });
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        set_ui_waker(None);
        set_current_app(None);
        clear_ui_loop_effects();
        clear_after_mount_effects();
        // Dispose the root owner — this cleans up all reactive computations,
        // effects, memos, and context in the entire tree.
        self.inner.root_owner.dispose();
        let _ = self.unmount();
    }
}

impl RuntimeInner {
    /// Mount the application tree. Called once at startup.
    fn mount_root<F>(&self, root_owner: Rc<Owner>, render: F) -> Result<()>
    where
        F: FnOnce() -> Element + 'static,
    {
        clear_after_mount_effects();

        // Both building the element tree AND mounting it must happen inside
        // the root owner scope. Component `mount()` implementations create
        // reactive computations (effects, signals, child scopes) that require
        // an active owner.
        with_owner(root_owner, || {
            let next_tree = render();

            match self.mount_tree(next_tree) {
                Ok(()) => {
                    schedule_after_mount_effects();
                    Ok(())
                }
                Err(error) => {
                    logging::error(format!("mount failed: {error}"));
                    clear_after_mount_effects();
                    self.mount_tree(build_fallback_element(format!("mount failed: {error}")))
                }
            }
        })
    }

    fn unmount_root(&self) -> Result<()> {
        let mounted = self.mounted.borrow_mut().take();
        if let Some(mounted) = mounted {
            if let Err(error) = mounted.portal_host.clear() {
                logging::error(format!("portal host clear failed during unmount: {error}"));
            }
            let result = map_arkui_result(self.root.borrow_mut().unmount());
            mounted.app_root.cleanup_recursive();
            set_current_portal_host(None);
            return result;
        }
        let result = map_arkui_result(self.root.borrow_mut().unmount());
        set_current_portal_host(None);
        result
    }

    fn mount_tree(&self, tree: Element) -> Result<()> {
        let (portal_host, portal_root) = map_arkui_result(build_portal_host())?;
        let (app_node, app_root) =
            map_arkui_result(with_current_portal_host(portal_host.clone(), || {
                mount_element(tree)
            }))?;
        let host_root = map_arkui_result(build_host_root(app_node, portal_root))?;

        let previous = self.mounted.borrow_mut().take();
        let mut root = self.root.borrow_mut();
        if let Some(previous) = previous.as_ref() {
            if let Err(error) = previous.portal_host.clear() {
                logging::error(format!("portal host clear failed during mount: {error}"));
            }
        }
        let unmount_result = map_arkui_result(root.unmount());
        if let Some(previous) = previous {
            previous.app_root.cleanup_recursive();
        }
        let _ = unmount_result;
        set_current_portal_host(Some(portal_host.clone()));
        if let Err(error) = map_arkui_result(root.mount(host_root.clone())) {
            set_current_portal_host(None);
            if let Err(clear_error) = portal_host.clear() {
                logging::error(format!(
                    "portal host clear failed after mount failure: {clear_error}"
                ));
            }
            app_root.cleanup_recursive();
            return Err(error);
        }

        self.mounted.borrow_mut().replace(MountedApp {
            host_root,
            app_root,
            portal_host,
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

fn build_host_root(app_root: ArkUINode, portal_root: ArkUINode) -> ArkUIResult<ArkUINode> {
    let mut host = crate::ohos_arkui_binding::component::built_in_component::Stack::new()?;
    host.percent_width(1.0)?;
    host.percent_height(1.0)?;
    host.set_alignment(i32::from(
        crate::ohos_arkui_binding::types::alignment::Alignment::TopStart,
    ))?;
    host.set_clip(false)?;
    host.add_child(app_root)?;
    host.add_child(portal_root)?;
    Ok(host.into())
}

fn map_arkui_result<T, E: ToString>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|error| {
        let reason = error.to_string();
        logging::error(format!("arkui runtime error: {reason}"));
        Error::from_reason(reason)
    })
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
    // Capture the current reactive owner so the callback runs in the correct scope.
    let owner = crate::owner::current_owner();
    AFTER_MOUNT_EFFECTS.with(|state| {
        state.borrow_mut().push(Box::new(move || {
            if let Some(owner) = owner {
                crate::owner::with_owner(owner, effect);
            } else {
                effect();
            }
        }));
    });
}

pub fn queue_ui_loop(effect: impl FnOnce() + 'static) {
    // Capture the current reactive owner so the callback runs in the correct scope.
    let owner = crate::owner::current_owner();
    UI_LOOP_EFFECTS.with(|state| {
        state.borrow_mut().push(Box::new(move || {
            if let Some(owner) = owner {
                crate::owner::with_owner(owner, effect);
            } else {
                effect();
            }
        }));
    });
    wake_ui_loop();
}

fn clear_after_mount_effects() {
    AFTER_MOUNT_EFFECTS.with(|state| {
        state.borrow_mut().clear();
    });
}

pub(crate) fn schedule_after_mount_effects() {
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

pub(crate) fn wake_ui_loop() {
    UI_WAKER.with(|state| {
        if let Some(waker) = state.borrow().as_ref() {
            waker.wake();
        }
    });
}
