use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arkit_router::{Route, RouteTransitionDirection, RouteTransitionEvent};
use ohos_arkui_binding::animate::options::Animation;
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUICommonAttribute};
use ohos_arkui_binding::component::built_in_component::Stack;
use ohos_arkui_binding::r#type::animation_finish_type::AnimationFinishCallbackType;
use ohos_arkui_binding::r#type::attribute::ArkUINodeAttributeType;
use ohos_arkui_binding::r#type::curve::Curve;

use crate::component::{mount_element, MountedElement};
use crate::component::dispose_node_handle;
use crate::logging;
use crate::owner::{on_cleanup, with_child_owner, Owner};
use crate::runtime::{queue_after_mount, queue_ui_loop, schedule_after_mount_effects};
use crate::view::Element;
use crate::view::ViewNode;

const ALIGN_CENTER: i32 = 4;

/// Configuration for animated route transitions.
#[derive(Debug, Clone)]
pub struct RouteTransitionConfig {
    pub duration_ms: i32,
    pub replace_duration_ms: i32,
    pub enter_scale: f32,
    pub exit_scale: f32,
    pub enter_opacity: f32,
    pub exit_opacity: f32,
}

impl Default for RouteTransitionConfig {
    fn default() -> Self {
        Self {
            duration_ms: 150,
            replace_duration_ms: 130,
            enter_scale: 0.94,
            exit_scale: 0.94,
            enter_opacity: 0.0,
            exit_opacity: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Pose {
    scale: f32,
    opacity: f32,
}

const REST: Pose = Pose {
    scale: 1.0,
    opacity: 1.0,
};

fn apply_pose(node: &mut ArkUINode, pose: Pose) -> ArkUIResult<()> {
    node.set_transform_center(vec![0.0, 0.0, 0.0, 0.5, 0.5, 0.0])?;
    node.set_scale(vec![pose.scale, pose.scale])?;
    node.opacity(pose.opacity)
}

fn motion_for(
    direction: RouteTransitionDirection,
    config: &RouteTransitionConfig,
) -> Option<(i32, Curve)> {
    match direction {
        RouteTransitionDirection::Forward => Some((config.duration_ms, Curve::FastOutSlowIn)),
        RouteTransitionDirection::Backward => Some((config.duration_ms, Curve::EaseInOut)),
        RouteTransitionDirection::Replace => {
            Some((config.replace_duration_ms, Curve::FastOutSlowIn))
        }
        RouteTransitionDirection::None => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OverlayData {
    id: u64,
    route: Route,
    direction: RouteTransitionDirection,
    is_enter: bool,
    settles_to: Route,
}

type RenderFn = Rc<dyn Fn(&Route) -> Element>;

// ── State management ──────────────────────────────────────────────────────

struct Layer {
    mounted: MountedElement,
    owner: Rc<Owner>,
    cancel: Option<Rc<Cell<bool>>>,
}

struct AnimatedRouterState {
    container: RefCell<Stack>,
    base: RefCell<Option<Layer>>,
    overlay: RefCell<Option<Layer>>,
}

impl AnimatedRouterState {
    fn cleanup(&self) {
        self.remove_overlay();
        self.remove_base();
    }

    fn remove_base(&self) {
        let layer = self.base.borrow_mut().take();
        if let Some(layer) = layer {
            if let Some(cancel) = &layer.cancel {
                cancel.set(false);
            }
            if let Ok(Some(removed)) = self.container.borrow_mut().remove_child(0) {
                let _ = dispose_node_handle(removed);
            }
            layer.mounted.cleanup_recursive();
            layer.owner.dispose();
        }
    }

    fn remove_overlay(&self) {
        let layer = self.overlay.borrow_mut().take();
        if let Some(layer) = layer {
            if let Some(cancel) = &layer.cancel {
                cancel.set(false);
            }
            let last_idx = self
                .container
                .borrow()
                .raw()
                .children()
                .len()
                .saturating_sub(1);
            if let Ok(Some(removed)) = self.container.borrow_mut().remove_child(last_idx) {
                let _ = dispose_node_handle(removed);
            }
            layer.mounted.cleanup_recursive();
            layer.owner.dispose();
        }
    }

    fn set_base(&self, route: &Route, enabled: bool, render_fn: &RenderFn) -> ArkUIResult<()> {
        self.remove_base();
        let key = format!("route-base:{}", route.raw());
        let surface_key = format!("route-base-surface:{}", route.raw());
        let route = route.clone();
        let render_fn = render_fn.clone();

        // Both element creation AND mounting must happen inside with_child_owner
        // so that all reactive effects are properly scoped under child_owner.
        // keyed_scope's mount() calls with_child_owner internally, which creates
        // nested child owners — all will be cleaned up when we dispose this owner.
        let (mount_result, child_owner) = with_child_owner(|| {
            let el = crate::view::keyed_scope(key, move || {
                crate::view::stack_component()
                    .percent_width(1.0)
                    .percent_height(1.0)
                    .style(ArkUINodeAttributeType::Alignment, ALIGN_CENTER)
                    .style(ArkUINodeAttributeType::ZIndex, 0_i32)
                    .style(ArkUINodeAttributeType::Enabled, enabled)
                    .children(vec![route_surface(&route, surface_key.clone(), &render_fn)])
                    .into()
            });
            mount_element(el)
        });
        let (child_node, child_meta) = match mount_result {
            Ok(mounted) => mounted,
            Err(error) => {
                child_owner.dispose();
                logging::error(format!("animated_router: base mount failed: {error}"));
                return Err(error);
            }
        };

        // Insert as first child
        let is_empty = self.container.borrow().raw().children().is_empty();
        if is_empty {
            let _ = self.container.borrow_mut().add_child(child_node.clone());
        } else {
            let _ = self
                .container
                .borrow_mut()
                .insert_child(child_node.clone(), 0);
        }

        *self.base.borrow_mut() = Some(Layer {
            mounted: child_meta,
            owner: child_owner,
            cancel: None,
        });
        Ok(())
    }

    fn set_overlay(
        &self,
        data: OverlayData,
        config: &RouteTransitionConfig,
        state_ref: Rc<AnimatedRouterState>,
        transition_id: Rc<Cell<u64>>,
        render_fn: &RenderFn,
    ) -> ArkUIResult<()> {
        self.remove_overlay();

        let surface_key = format!("route-overlay-surface:{}:{}", data.id, data.route.raw());

        let key = format!("route-overlay:{}:{}", data.id, data.route.raw());
        let surface_el = route_surface(&data.route, surface_key, render_fn);

        let mut layer_el = crate::view::stack_component()
            .key(key)
            .percent_width(1.0)
            .percent_height(1.0)
            .style(ArkUINodeAttributeType::Alignment, ALIGN_CENTER)
            .style(ArkUINodeAttributeType::ZIndex, 1_i32)
            .style(ArkUINodeAttributeType::Enabled, false)
            .children(vec![surface_el]);

        let is_active = Rc::new(Cell::new(true));

        if let Some((duration_ms, curve)) = motion_for(data.direction, config) {
            let initial = if data.is_enter {
                Pose {
                    scale: config.enter_scale,
                    opacity: config.enter_opacity,
                }
            } else {
                REST
            };
            let target = if data.is_enter {
                REST
            } else {
                Pose {
                    scale: config.exit_scale,
                    opacity: config.exit_opacity,
                }
            };

            let expected_id = data.id;
            let settles_to = data.settles_to.clone();
            let overlay_render_fn = render_fn.clone();

            let active_ref = is_active.clone();
            layer_el = layer_el.native_with_cleanup(move |node| {
                apply_pose(node.borrow_mut(), initial)?;

                let animated_node = Rc::new(RefCell::new(node.borrow_mut().clone()));
                let node_active = active_ref.clone();
                let cleanup_active = node_active.clone();

                // Slot keeps the Animation alive until the finish callback fires.
                // Without this, the Animation (and its native option + callback contexts)
                // is dropped at the end of queue_after_mount, but the native animation
                // still holds raw pointers to the freed contexts → use-after-free → SIGBUS.
                let animation_slot = Rc::new(RefCell::new(None::<Animation>));

                queue_after_mount(move || {
                    if !node_active.get() {
                        return;
                    }

                    let animation = Animation::new();
                    animation.duration(duration_ms);
                    animation.curve(curve);

                    let update_node = animated_node.clone();
                    let update_active = node_active.clone();
                    animation.update(move || {
                        if !update_active.get() {
                            return;
                        }
                        let mut n = update_node.borrow_mut();
                        let _ = apply_pose(&mut n, target);
                    });

                    let finish_slot = animation_slot.clone();
                    let finish_active = node_active.clone();
                    animation.finish(AnimationFinishCallbackType::Logically, move || {
                        // Release the Animation from the slot so it can be dropped.
                        let release_slot = finish_slot.clone();
                        queue_ui_loop(move || {
                            let _ = release_slot.borrow_mut().take();
                        });
                        if !finish_active.get() {
                            return;
                        }
                        let finish_tid = transition_id.clone();
                        let finish_state = state_ref.clone();
                        let finish_render_fn = overlay_render_fn.clone();
                        let finish_settles = settles_to.clone();
                        queue_ui_loop(move || {
                            if finish_tid.get() != expected_id {
                                return;
                            }
                            finish_state.remove_overlay();
                            if let Err(error) =
                                finish_state.set_base(&finish_settles, true, &finish_render_fn)
                            {
                                logging::error(format!(
                                    "animated_router: failed to settle base layer: {error}"
                                ));
                                return;
                            }
                            schedule_after_mount_effects();
                        });
                    });

                    if !node_active.get() {
                        return;
                    }

                    let a_node = animated_node.borrow().clone();
                    if let Err(e) = a_node.animate_to(&animation) {
                        logging::error(format!("animated_router: animate_to failed: {e}"));
                        return;
                    }

                    // Store in slot to keep Animation alive until finish callback releases it.
                    *animation_slot.borrow_mut() = Some(animation);
                });

                Ok(move || {
                    cleanup_active.set(false);
                })
            });
        }

        let (mount_result, child_owner) = with_child_owner(|| mount_element(layer_el.into()));
        let (child_node, child_meta) = match mount_result {
            Ok(mounted) => mounted,
            Err(error) => {
                child_owner.dispose();
                logging::error(format!("animated_router: overlay mount failed: {error}"));
                return Err(error);
            }
        };

        let _ = self.container.borrow_mut().add_child(child_node.clone());

        *self.overlay.borrow_mut() = Some(Layer {
            mounted: child_meta,
            owner: child_owner,
            cancel: Some(is_active),
        });
        Ok(())
    }
}

fn route_surface(route: &Route, key: String, render_fn: &RenderFn) -> Element {
    let render_fn = render_fn.clone();
    let route = route.clone();
    crate::view::column_component()
        .key(key)
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![{
            let key = format!("route-scene:{}", route.raw());
            let route = route.clone();
            let render_fn = render_fn.clone();
            crate::view::keyed_scope(key, move || render_fn(&route))
        }])
        .into()
}

// ── ViewNode implementation ──────────────────────────────────────────────

struct AnimatedRouterView {
    config: RouteTransitionConfig,
    render_fn: RenderFn,
}

impl ViewNode for AnimatedRouterView {
    fn kind(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn key(&self) -> Option<&str> {
        None
    }

    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let Self { config, render_fn } = *self;
        let active_router = crate::route::router();
        let initial = active_router.current_route();

        let mut container = Stack::new()?;
        let container_node = container.borrow_mut().clone();

        let state = Rc::new(AnimatedRouterState {
            container: RefCell::new(container),
            base: RefCell::new(None),
            overlay: RefCell::new(None),
        });

        let transition_id = Rc::new(Cell::new(0u64));
        let animations_ready = Rc::new(Cell::new(false));
        let subscription_id = Rc::new(RefCell::new(None::<usize>));

        // Mount initial base
        state.set_base(&initial, true, &render_fn)?;
        schedule_after_mount_effects();

        // Subscribe to router transitions
        {
            let sub_state = state.clone();
            let sub_render_fn = render_fn.clone();
            let sub_config = config.clone();
            let sub_tid = transition_id.clone();
            let sub_ready = animations_ready.clone();
            let mount_ready = animations_ready.clone();

            let id = active_router.subscribe_transition(move |event: RouteTransitionEvent| {
                if !sub_ready.get() || event.direction() == RouteTransitionDirection::None {
                    let ui_state = sub_state.clone();
                    let ui_render_fn = sub_render_fn.clone();
                    let base = event.to().clone();
                    queue_ui_loop(move || {
                        ui_state.remove_overlay();
                        if let Err(error) = ui_state.set_base(&base, true, &ui_render_fn) {
                            logging::error(format!(
                                "animated_router: failed to refresh settled base layer: {error}"
                            ));
                            return;
                        }
                        schedule_after_mount_effects();
                    });
                    return;
                }

                let direction = event.direction();
                let next_id = sub_tid.get().wrapping_add(1);
                sub_tid.set(next_id);

                let (base, data) = match direction {
                    RouteTransitionDirection::Forward | RouteTransitionDirection::Replace => (
                        event.from().clone(),
                        OverlayData {
                            id: next_id,
                            route: event.to().clone(),
                            direction,
                            is_enter: true,
                            settles_to: event.to().clone(),
                        },
                    ),
                    RouteTransitionDirection::Backward => (
                        event.to().clone(),
                        OverlayData {
                            id: next_id,
                            route: event.from().clone(),
                            direction,
                            is_enter: false,
                            settles_to: event.to().clone(),
                        },
                    ),
                    RouteTransitionDirection::None => unreachable!(),
                };

                let ui_state = sub_state.clone();
                let ui_render_fn = sub_render_fn.clone();
                let ui_config = sub_config.clone();
                let ui_tid = sub_tid.clone();
                queue_ui_loop(move || {
                    if let Err(error) = ui_state.set_base(&base, false, &ui_render_fn) {
                        logging::error(format!(
                            "animated_router: failed to refresh base layer: {error}"
                        ));
                        return;
                    }
                    if let Err(error) = ui_state.set_overlay(
                        data,
                        &ui_config,
                        ui_state.clone(),
                        ui_tid,
                        &ui_render_fn,
                    ) {
                        logging::error(format!(
                            "animated_router: failed to mount overlay layer: {error}"
                        ));
                        ui_state.remove_overlay();
                        let _ = ui_state.set_base(&base, true, &ui_render_fn);
                        return;
                    }
                    schedule_after_mount_effects();
                });
            });
            *subscription_id.borrow_mut() = Some(id);

            queue_after_mount(move || {
                mount_ready.set(true);
            });
        }

        on_cleanup({
            let router = active_router.clone();
            let sub_id = subscription_id.clone();
            let cleanup_state = state.clone();
            move || {
                if let Some(id) = sub_id.borrow_mut().take() {
                    router.unsubscribe_transition(id);
                }
                cleanup_state.cleanup();
            }
        });

        let cleanup_state = state.clone();
        let mounted = MountedElement::new(
            TypeId::of::<Self>(),
            std::any::type_name::<Self>(),
            None,
            vec![Box::new(move || cleanup_state.cleanup())],
            vec![],
        );
        Ok((container_node, mounted))
    }

    fn patch(
        self: Box<Self>,
        _node: &mut ArkUINode,
        _mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        // Self-managed via reactive effects — patch is a no-op.
        Ok(())
    }
}

/// Renders the current route with animated transitions between navigation events.
///
/// Uses a two-layer stack (base + overlay) to animate Forward, Backward, and
/// Replace transitions. The first mount skips animation.
pub fn animated_router_view(
    config: RouteTransitionConfig,
    render_fn: impl Fn(&Route) -> Element + 'static,
) -> Element {
    AnimatedRouterView {
        config,
        render_fn: Rc::new(render_fn),
    }
    .into()
}
