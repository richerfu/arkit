use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::component::MountedElement;
use crate::logging;
use crate::ohos_arkui_binding::common::error::ArkUIResult;
use crate::ohos_arkui_binding::common::node::ArkUINode;
use crate::ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUICommonAttribute};
use crate::ohos_arkui_binding::component::built_in_component::Column;
use crate::queue_ui_loop;
use crate::signal::{with_hook_state, HookState};

use super::super::element::{Element, ViewNode};

struct ScopedState {
    render: RefCell<Rc<dyn Fn() -> Element>>,
    hooks: Rc<RefCell<HookState>>,
    container: RefCell<Option<Column>>,
    mounted_child: RefCell<Option<MountedElement>>,
    is_rendering: Cell<bool>,
    pending: Cell<bool>,
    render_scheduled: Cell<bool>,
}

impl ScopedState {
    fn new(render: Rc<dyn Fn() -> Element>) -> Rc<Self> {
        Rc::new_cyclic(|weak: &std::rc::Weak<Self>| {
            let weak = weak.clone();
            let subscriber: Rc<dyn Fn()> = Rc::new(move || {
                if let Some(state) = weak.upgrade() {
                    state.schedule_render();
                }
            });

            Self {
                render: RefCell::new(render),
                hooks: Rc::new(RefCell::new(HookState::with_signal_subscriber(subscriber))),
                container: RefCell::new(None),
                mounted_child: RefCell::new(None),
                is_rendering: Cell::new(false),
                pending: Cell::new(false),
                render_scheduled: Cell::new(false),
            }
        })
    }

    fn schedule_render(self: &Rc<Self>) {
        if self.render_scheduled.replace(true) {
            return;
        }

        let weak = Rc::downgrade(self);
        queue_ui_loop(move || {
            let Some(state) = weak.upgrade() else {
                return;
            };
            state.render_scheduled.set(false);
            if let Err(error) = state.request_render() {
                logging::error(format!(
                    "scope error: deferred scoped render failed: {error}"
                ));
            }
        });
    }

    fn render_element(&self) -> Element {
        let hooks = self.hooks.clone();
        hooks.borrow_mut().reset_cursor();
        let render = self.render.borrow().clone();
        with_hook_state(hooks, || render())
    }

    fn request_render(&self) -> ArkUIResult<()> {
        if self.is_rendering.get() {
            self.pending.set(true);
            return Ok(());
        }

        self.is_rendering.set(true);
        let result = self.render_once();
        self.is_rendering.set(false);

        if result.is_ok() && self.pending.replace(false) {
            self.request_render()?;
        }

        result
    }

    fn render_once(&self) -> ArkUIResult<()> {
        let next = self.render_element();
        let mut container = self.container.borrow_mut();
        let container = container
            .as_mut()
            .expect("scoped container should be initialized before rerender");
        let mut mounted_child = self.mounted_child.borrow_mut();

        match mounted_child.as_mut() {
            Some(existing) => {
                if next.kind() == existing.kind && next.key() == existing.key.as_deref() {
                    let child_handle = container
                        .borrow_mut()
                        .children()
                        .first()
                        .cloned()
                        .expect("scoped container child should exist");
                    let mut child_node = child_handle.borrow_mut();
                    if let Err(error) = next.patch(&mut child_node, existing) {
                        logging::error(format!(
                            "scope error: failed to patch scoped child: {error}"
                        ));
                        return Err(error);
                    }
                } else {
                    if let Some(old_child) = container.remove_child(0)? {
                        old_child.borrow_mut().dispose()?;
                    }
                    let old_meta = mounted_child.take().expect("scoped meta should exist");
                    old_meta.cleanup_recursive();

                    let (child_node, child_meta) = next.mount().map_err(|error| {
                        logging::error(format!(
                            "scope error: failed to mount replacement scoped child: {error}"
                        ));
                        error
                    })?;
                    let mut child_cleanup_node = child_node.clone();
                    if let Err(error) = container.add_child(child_node) {
                        logging::error(format!(
                            "scope error: failed to attach replacement scoped child {}: {error}",
                            child_meta.name
                        ));
                        let _ = child_cleanup_node.dispose();
                        child_meta.cleanup_recursive();
                        return Err(error);
                    }
                    mounted_child.replace(child_meta);
                }
            }
            None => {
                let (child_node, child_meta) = next.mount().map_err(|error| {
                    logging::error(format!(
                        "scope error: failed to mount initial scoped child: {error}"
                    ));
                    error
                })?;
                let mut child_cleanup_node = child_node.clone();
                if let Err(error) = container.add_child(child_node) {
                    logging::error(format!(
                        "scope error: failed to attach initial scoped child {}: {error}",
                        child_meta.name
                    ));
                    let _ = child_cleanup_node.dispose();
                    child_meta.cleanup_recursive();
                    return Err(error);
                }
                mounted_child.replace(child_meta);
            }
        }

        self.hooks.borrow_mut().finalize_render();
        Ok(())
    }

    fn cleanup(&self) {
        self.hooks.borrow_mut().cleanup_all();
        if let Some(mounted_child) = self.mounted_child.borrow_mut().take() {
            mounted_child.cleanup_recursive();
        }
    }
}

pub struct ScopeElement {
    render: Rc<dyn Fn() -> Element>,
    key: Option<String>,
}

pub fn scope(render: impl Fn() -> Element + 'static) -> Element {
    ScopeElement {
        render: Rc::new(render),
        key: None,
    }
    .into()
}

pub fn keyed_scope(key: impl Into<String>, render: impl Fn() -> Element + 'static) -> Element {
    ScopeElement {
        render: Rc::new(render),
        key: Some(key.into()),
    }
    .into()
}

impl ScopeElement {
    pub fn key(mut self, value: impl Into<String>) -> Self {
        self.key = Some(value.into());
        self
    }
}

impl ViewNode for ScopeElement {
    fn kind(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let Self { render, key } = *self;
        let state = ScopedState::new(render);
        let child_element = state.render_element();
        let (child_node, child_meta) = child_element.mount().map_err(|error| {
            logging::error(format!(
                "scope error: failed to mount scoped root child: {error}"
            ));
            error
        })?;

        let mut container = Column::new()?;
        container.add_child(child_node).map_err(|error| {
            logging::error(format!(
                "scope error: failed to attach scoped root child {}: {error}",
                child_meta.name
            ));
            error
        })?;
        let container_node = container.borrow_mut().clone();
        state.container.replace(Some(container));
        state.mounted_child.replace(Some(child_meta));
        state.hooks.borrow_mut().finalize_render();

        let cleanup_state = state.clone();
        let mut mounted = MountedElement::new(
            TypeId::of::<Self>(),
            std::any::type_name::<Self>(),
            key,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );
        mounted.set_state(Box::new(state));
        Ok((container_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        _node: &mut ArkUINode,
        mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        let Self { render, key } = *self;
        let state = mounted
            .state_mut::<Rc<ScopedState>>()
            .expect("scoped state should exist")
            .clone();
        state.render.replace(render);
        state.request_render()?;
        mounted.key = key;
        Ok(())
    }
}
