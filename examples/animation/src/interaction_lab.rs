use std::cell::Cell;
use std::rc::Rc;

use arkit::prelude::*;

use crate::{color, cubic_out, target, ActionButton, Metric, Section};

const DRAG_TARGET: &str = "lab-drag-target";
const SCROLL_TARGET: &str = "lab-scroll-target";
const DRAG_MAX_X_VP: f32 = 184.0;
const DRAG_MAX_Y_VP: f32 = 110.0;
const SCROLL_TRAVEL_VP: f32 = 168.0;

#[component]
pub(crate) fn InteractionLab() -> Element {
    rsx! {
        DragDemo {}
        ScrollDemo {}
        AnimatableDemo {}
    }
}

#[component]
fn DragDemo() -> Element {
    let status = use_signal(|| "Idle".to_string());
    let controls = use_animation(drag_mapping_timeline());
    let drag_samples = use_hook(|| Rc::new(Cell::new(0_u32)));
    let grab_samples = drag_samples.clone();
    let move_samples = drag_samples.clone();
    let release_samples = drag_samples.clone();
    let callbacks = DraggableCallbacks {
        grab: Some(Rc::new(move |update| {
            grab_samples.set(0);
            let mut status = status;
            status.set(format_drag("Grab", update))
        })),
        drag: Some(Rc::new(move |_| {
            move_samples.set(move_samples.get().saturating_add(1));
        })),
        release: Some(Rc::new(move |update| {
            let mut status = status;
            status.set(format!(
                "{} · {} frame samples",
                format_drag("Release", update),
                release_samples.get()
            ))
        })),
        snap: Some(Rc::new(move |update| {
            let mut status = status;
            status.set(format_drag("Snap queued", update))
        })),
        settle: Some(Rc::new(move |update| {
            let mut status = status;
            status.set(format_drag("Settled", update))
        })),
        ..DraggableCallbacks::default()
    };
    let drag = use_draggable(
        controls.clone(),
        TargetName::owned(DRAG_TARGET),
        DraggableConfig {
            axis: DragAxis::Both,
            mapping: DragMapping::DirectPosition,
            constraints: Some(DragConstraints::new(
                Vec2::new(0.0, 0.0),
                Vec2::new(DRAG_MAX_X_VP, DRAG_MAX_Y_VP),
            )),
            snap: DragSnap::Grid(Vec2::new(46.0, 55.0)),
            release_duration: TimeSpan::from_millis(520),
            map_duration: TimeSpan::from_millis(1_000),
            auto_scroll: Some(AutoScroll {
                viewport: DragConstraints::new(
                    Vec2::new(0.0, 0.0),
                    Vec2::new(DRAG_MAX_X_VP, DRAG_MAX_Y_VP),
                ),
                threshold: 24.0,
                max_speed: 320.0,
            }),
            ..DraggableConfig::default()
        },
        callbacks,
    );
    let clock = use_hook(|| Rc::new(Cell::new(0_u64)));
    let touch_drag = drag.clone();
    let touch_clock = clock.clone();

    rsx! {
        Section {
            title: "Draggable + inertia + snap",
            description: "Touch input is coalesced once per root frame. X/Y outputs follow the pointer independently, then deterministic velocity, bounds, grid snap and spring release stay on the same Engine.",
            column {
                width: "100%",
                height: 210.0,
                align_items: "start",
                justify_content: "start",
                padding: 12.0,
                background_color: "#FFE2E8F0",
                border_radius: 14.0,
                clip: false,
                DragTarget {
                    on_pointer: move |pointer: dioxus_elements::event::PointerPayload| {
                        let at = pointer_time(pointer, &touch_clock);
                        let point = if pointer.has_window_position() {
                            Vec2::new(pointer.window_x, pointer.window_y)
                        } else {
                            Vec2::new(pointer.x, pointer.y)
                        };
                        match pointer.action {
                            dioxus_elements::event::PointerAction::Down => { touch_drag.grab(at, point); }
                            dioxus_elements::event::PointerAction::Move => { touch_drag.drag(at, point); }
                            dioxus_elements::event::PointerAction::Up => { touch_drag.release(); }
                            dioxus_elements::event::PointerAction::Cancel => touch_drag.stop(),
                            dioxus_elements::event::PointerAction::Unknown => {}
                        }
                    }
                }
            }
            flex {
                margin_top: 12.0,
                width: "100%",
                flex_wrap: "wrap",
                ActionButton {
                    label: "Simulate drag",
                    on_press: {
                        let drag = drag.clone();
                        move |_| {
                            drag.grab(point_ms(0), Vec2::new(0.0, 0.0));
                            drag.drag(point_ms(90), Vec2::new(132.0, 68.0));
                            drag.drag(point_ms(170), Vec2::new(176.0, 96.0));
                            drag.release();
                        }
                    }
                }
                ActionButton {
                    label: "Reset",
                    on_press: {
                        let drag = drag.clone();
                        move |_| {
                            drag.reset();
                            let mut status = status;
                            status.set("Reset to origin".to_string());
                        }
                    }
                }
                ActionButton {
                    label: "Stop",
                    on_press: {
                        let drag = drag.clone();
                        move |_| {
                            drag.stop();
                            let mut status = status;
                            status.set("Stopped".to_string());
                        }
                    }
                }
                ActionButton {
                    label: "Refresh bounds",
                    on_press: {
                        let drag = drag.clone();
                        move |_| {
                            drag.refresh();
                            let mut status = status;
                            status.set("Bounds refreshed".to_string());
                        }
                    }
                }
                Metric { label: "Drag update", value: status() }
            }
        }
    }
}

#[component]
fn DragTarget(on_pointer: EventHandler<dioxus_elements::event::PointerPayload>) -> Element {
    let target = use_animation_target(DRAG_TARGET);
    rsx! {
        column {
            native_ref: target.native_ref(),
            width: 96.0,
            height: 72.0,
            align_items: "center",
            justify_content: "center",
            background_color: "#FF4F46E5",
            border_radius: 20.0,
            hit_test_behavior: "default",
            ontouch: move |event: dioxus_core::Event<dioxus_elements::event::PointerData>| {
                if let Some(pointer) = event.data().pointer {
                    on_pointer.call(pointer);
                }
            },
            text { font_size: 13.0, font_weight: 700, font_color: "#FFFFFFFF", "Drag me" }
        }
    }
}

fn drag_mapping_timeline() -> Timeline {
    Timeline::new().add(
        Animation::new(target(DRAG_TARGET))
            .tween(
                &TRANSLATE_X,
                Length::vp(0.0),
                Length::vp(DRAG_MAX_X_VP),
                TimeSpan::from_millis(1_000),
            )
            .tween(
                &TRANSLATE_Y,
                Length::vp(0.0),
                Length::vp(DRAG_MAX_Y_VP),
                TimeSpan::from_millis(1_000),
            ),
        TimelinePosition::START,
    )
}

#[component]
fn ScrollDemo() -> Element {
    let offset = use_signal(|| 0.0_f32);
    let status = use_signal(|| "outside · stationary".to_string());
    let clock = use_hook(|| Rc::new(Cell::new(0_u64)));
    let controls = use_animation(scroll_timeline());
    let observer = use_scroll_observer(
        controls.clone(),
        ScrollRange {
            start: 10.0,
            end: 90.0,
        },
        TimeSpan::from_millis(1_000),
        ScrollSync::Eased(Easing::Builtin(BuiltinEase::Cubic(EaseDirection::InOut))),
        ScrollCallbacks {
            enter: Some(Rc::new(move |_| {
                let mut status = status;
                status.set("enter".to_string());
            })),
            leave: Some(Rc::new(move |_| {
                let mut status = status;
                status.set("leave".to_string());
            })),
            update: Some(Rc::new(move |sample| {
                let mut offset = offset;
                offset.set(sample.offset);
                let mut status = status;
                status.set(format!(
                    "{:?} · {:.0}% · v={:.0}",
                    sample.direction,
                    sample.progress * 100.0,
                    sample.velocity
                ));
            })),
            ..ScrollCallbacks::default()
        },
    );

    rsx! {
        Section {
            title: "Scroll-linked scrubbing",
            description: "The slider acts as a platform scroll source: events are coalesced at a frame boundary, typed range/easing maps offset to seek, and direction/velocity callbacks stay observable.",
            column {
                width: "100%",
                height: 150.0,
                align_items: "start",
                justify_content: "center",
                padding: 12.0,
                background_color: "#FFE2E8F0",
                border_radius: 14.0,
                clip: false,
                ScrollTarget {}
            }
            slider {
                margin_top: 12.0,
                width: "100%",
                slider_value: offset(),
                slider_min: 0.0,
                slider_max: 100.0,
                slider_step: 1.0,
                selected_color: "#FF4F46E5",
                track_color: "#FFCBD5E1",
                block_color: "#FF312E81",
                on_change: {
                    let observer = observer.clone();
                    let clock = clock.clone();
                    move |event| {
                        drive_scroll(&observer, &clock, event.data().float_value);
                    }
                }
            }
            flex {
                margin_top: 8.0,
                width: "100%",
                flex_wrap: "wrap",
                for (value, label) in [(0.0, "0%"), (25.0, "25%"), (50.0, "50%"), (75.0, "75%"), (100.0, "100%")] {
                    ActionButton {
                        label,
                        on_press: {
                            let observer = observer.clone();
                            let clock = clock.clone();
                            move |_| drive_scroll(&observer, &clock, value)
                        },
                    }
                }
                ActionButton {
                    label: "Revert",
                    on_press: {
                        let observer = observer.clone();
                        move |_| {
                            observer.revert();
                            let mut offset = offset;
                            offset.set(0.0);
                            let mut status = status;
                            status.set("reverted · outside".to_string());
                        }
                    }
                }
            }
            flex {
                width: "100%",
                flex_wrap: "wrap",
                Metric { label: "Offset", value: format!("{:.0}", offset()) }
                Metric { label: "Sample", value: status() }
                Metric { label: "In range", value: observer.is_in_view().to_string() }
            }
        }
    }
}

#[component]
fn ScrollTarget() -> Element {
    let target = use_animation_target(SCROLL_TARGET);
    rsx! {
        column {
            native_ref: target.native_ref(),
            width: 112.0,
            height: 82.0,
            align_items: "center",
            justify_content: "center",
            background_color: "#FF0891B2",
            border_radius: 18.0,
            text { font_size: 13.0, font_weight: 700, font_color: "#FFFFFFFF", "Scroll sync" }
        }
    }
}

fn scroll_timeline() -> Timeline {
    let duration = TimeSpan::from_millis(1_000);
    Timeline::new().add(
        Animation::new(target(SCROLL_TARGET))
            .tween(
                &TRANSLATE_X,
                Length::vp(0.0),
                Length::vp(SCROLL_TRAVEL_VP),
                duration,
            )
            .tween(
                &ROTATION,
                Angle::degrees(-12.0),
                Angle::degrees(18.0),
                duration,
            )
            .tween(&SCALE_X, 0.72, 1.08, duration)
            .tween(&SCALE_Y, 0.72, 1.08, duration)
            .tween(
                &BACKGROUND_COLOR,
                color(0xff0891b2u32),
                color(0xff7c3aedu32),
                duration,
            ),
        TimelinePosition::START,
    )
}

fn drive_scroll(observer: &Rc<ScrollObserver>, clock: &Rc<Cell<u64>>, value: f32) {
    let at = clock.get().saturating_add(16_000_000);
    clock.set(at);
    observer.update_at(TimePoint::from_nanos(at), value);
}

#[component]
fn AnimatableDemo() -> Element {
    let value = use_animatable_with_defaults(
        0.0_f32,
        AnimatableDefaults {
            duration: TimeSpan::from_millis(720),
            easing: cubic_out(),
            ..AnimatableDefaults::default()
        },
    );
    let to_one = value.clone();
    let retarget = value.clone();
    let repeat = value.clone();
    let pause = value.clone();
    let set = value.clone();
    let revert = value.clone();

    rsx! {
        Section {
            title: "Animatable<T> drawing value",
            description: "A typed f32 value uses DrawingAdapter and the root clock for retargeting and repeating invalidation; no ArkUI property or async timer is required.",
            AnimatableBar { value: value.clone() }
            flex {
                margin_top: 12.0,
                width: "100%",
                flex_wrap: "wrap",
                ActionButton { label: "To 1.0", on_press: move |_| to_one.to(1.0) }
                ActionButton { label: "Retarget 0.35", on_press: move |_| retarget.retarget(0.35, TimeSpan::from_millis(360)) }
                ActionButton { label: "Repeat", on_press: move |_| repeat.animate_repeating(0.0, 1.0, TimeSpan::from_millis(1_200), Easing::Linear) }
                ActionButton { label: "Pause", on_press: move |_| pause.controls().pause() }
                ActionButton { label: "Set 0.65", on_press: move |_| set.set(0.65) }
                ActionButton { label: "Revert", on_press: move |_| revert.revert() }
            }
        }
    }
}

#[component]
fn AnimatableBar(value: Animatable<f32>) -> Element {
    let snapshot = use_animation_snapshot(value.controls());
    let _ = snapshot();
    let current = value.get().clamp(0.0, 1.0);
    rsx! {
        column {
            width: "100%",
            height: 28.0,
            background_color: "#FFE2E8F0",
            border_radius: 14.0,
            clip: true,
            column {
                width: format!("{}%", (current * 100.0).round().clamp(0.0, 100.0) as i32),
                height: "100%",
                background_color: "#FF4F46E5",
                border_radius: 14.0,
            }
        }
        Metric { label: "Value", value: format!("{current:.3}") }
    }
}

fn next_time(clock: &Rc<Cell<u64>>) -> TimePoint {
    let next = clock.get().saturating_add(16_000_000);
    clock.set(next);
    TimePoint::from_nanos(next)
}

fn pointer_time(
    pointer: dioxus_elements::event::PointerPayload,
    clock: &Rc<Cell<u64>>,
) -> TimePoint {
    if pointer.timestamp_nanos > clock.get() {
        clock.set(pointer.timestamp_nanos);
        TimePoint::from_nanos(pointer.timestamp_nanos)
    } else {
        next_time(clock)
    }
}

fn point_ms(milliseconds: u64) -> TimePoint {
    TimePoint::from_nanos(milliseconds * 1_000_000)
}

fn format_drag(label: &str, update: DragUpdate) -> String {
    format!(
        "{label} ({:.0},{:.0}) v=({:.0},{:.0}) auto=({:.0},{:.0})",
        update.position.x,
        update.position.y,
        update.velocity.x,
        update.velocity.y,
        update.auto_scroll_velocity.x,
        update.auto_scroll_velocity.y
    )
}
