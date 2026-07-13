use std::cell::Cell;
use std::rc::Rc;

use arkit::prelude::*;

use crate::{color, cubic_out, target, ActionButton, Metric, Section};

const DRAG_TARGET: &str = "lab-drag-target";
const SCROLL_TARGET: &str = "lab-scroll-target";

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
    let callbacks = DraggableCallbacks {
        grab: Some(Rc::new(move |update| {
            let mut status = status;
            status.set(format_drag("Grab", update))
        })),
        drag: Some(Rc::new(move |update| {
            let mut status = status;
            status.set(format_drag("Drag", update))
        })),
        release: Some(Rc::new(move |update| {
            let mut status = status;
            status.set(format_drag("Release", update))
        })),
        snap: Some(Rc::new(move |update| {
            let mut status = status;
            status.set(format_drag("Snap", update))
        })),
        ..DraggableCallbacks::default()
    };
    let drag = use_draggable(
        controls.clone(),
        TargetName::owned(DRAG_TARGET),
        DraggableConfig {
            axis: DragAxis::Both,
            constraints: Some(DragConstraints::new(
                Vec2::new(0.0, 0.0),
                Vec2::new(220.0, 110.0),
            )),
            snap: DragSnap::Grid(Vec2::new(55.0, 55.0)),
            release_duration: TimeSpan::from_millis(520),
            map_duration: TimeSpan::from_millis(1_000),
            auto_scroll: Some(AutoScroll {
                viewport: DragConstraints::new(Vec2::new(0.0, 0.0), Vec2::new(220.0, 110.0)),
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
            description: "Touch samples feed deterministic velocity, constrained seek mapping, auto-scroll hints and a spring release timeline owned by the root Engine.",
            column {
                percent_width: 1.0,
                height: 210.0,
                padding: 12.0,
                background_color: 0xffe2e8f0u32,
                border_radius: 14.0,
                clip: false,
                DragTarget {
                    on_pointer: move |pointer: dioxus_elements::event::PointerPayload| {
                        let at = next_time(&touch_clock);
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
                percent_width: 1.0,
                flex_wrap: "wrap",
                ActionButton {
                    label: "Simulate drag",
                    on_press: {
                        let drag = drag.clone();
                        move |_| {
                            drag.grab(point_ms(0), Vec2::new(0.0, 0.0));
                            drag.drag(point_ms(90), Vec2::new(132.0, 68.0));
                            drag.drag(point_ms(170), Vec2::new(184.0, 96.0));
                            drag.release();
                        }
                    }
                }
                ActionButton { label: "Reset", on_press: { let drag = drag.clone(); move |_| drag.reset() } }
                ActionButton { label: "Stop", on_press: { let drag = drag.clone(); move |_| drag.stop() } }
                ActionButton { label: "Refresh bounds", on_press: { let drag = drag.clone(); move |_| drag.refresh() } }
                Metric { label: "Drag update", value: status() }
            }
        }
    }
}

#[component]
fn DragTarget(on_pointer: EventHandler<dioxus_elements::event::PointerPayload>) -> Element {
    let _target = use_animation_target(DRAG_TARGET);
    rsx! {
        column {
            width: 96.0,
            height: 72.0,
            align_items: "center",
            justify_content: "center",
            background_color: 0xff4f46e5u32,
            border_radius: 20.0,
            hit_test_behavior: 0,
            ontouch: move |event: dioxus_core::Event<dioxus_elements::event::PointerData>| {
                if let Some(pointer) = event.data().pointer {
                    on_pointer.call(pointer);
                }
            },
            text { font_size: 13.0, font_weight: 700, font_color: 0xffffffffu32, "Drag me" }
        }
    }
}

fn drag_mapping_timeline() -> Timeline {
    Timeline::new().add(
        Animation::new(target(DRAG_TARGET))
            .tween(
                &TRANSLATE_X,
                Length::vp(0.0),
                Length::vp(220.0),
                TimeSpan::from_millis(1_000),
            )
            .tween(
                &TRANSLATE_Y,
                Length::vp(0.0),
                Length::vp(110.0),
                TimeSpan::from_millis(1_000),
            )
            .tween(
                &ROTATION,
                Angle::degrees(0.0),
                Angle::degrees(16.0),
                TimeSpan::from_millis(1_000),
            ),
        TimelinePosition::START,
    )
}

#[component]
fn ScrollDemo() -> Element {
    let offset = use_signal(|| 0.0_f32);
    let status = use_signal(|| "outside · stationary".to_string());
    let clock = use_signal(|| 0_u64);
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
                percent_width: 1.0,
                height: 150.0,
                justify_content: "center",
                padding: 12.0,
                background_color: 0xffe2e8f0u32,
                border_radius: 14.0,
                clip: false,
                ScrollTarget {}
            }
            slider {
                margin_top: 12.0,
                percent_width: 1.0,
                slider_value: offset(),
                slider_min: 0.0,
                slider_max: 100.0,
                slider_step: 1.0,
                selected_color: 0xff4f46e5u32,
                track_color: 0xffcbd5e1u32,
                block_color: 0xff312e81u32,
                on_change: {
                    let observer = observer.clone();
                    move |event| {
                        drive_scroll(&observer, offset, clock, event.data().float_value);
                    }
                }
            }
            flex {
                margin_top: 8.0,
                percent_width: 1.0,
                flex_wrap: "wrap",
                for (value, label) in [(0.0, "0%"), (25.0, "25%"), (50.0, "50%"), (75.0, "75%"), (100.0, "100%")] {
                    ActionButton {
                        label,
                        on_press: {
                            let observer = observer.clone();
                            move |_| drive_scroll(&observer, offset, clock, value)
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
                        }
                    }
                }
            }
            flex {
                percent_width: 1.0,
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
    let _target = use_animation_target(SCROLL_TARGET);
    rsx! {
        column {
            width: 112.0,
            height: 82.0,
            align_items: "center",
            justify_content: "center",
            background_color: 0xff0891b2u32,
            border_radius: 18.0,
            text { font_size: 13.0, font_weight: 700, font_color: 0xffffffffu32, "Scroll sync" }
        }
    }
}

fn scroll_timeline() -> Timeline {
    let duration = TimeSpan::from_millis(1_000);
    Timeline::new().add(
        Animation::new(target(SCROLL_TARGET))
            .tween(&TRANSLATE_X, Length::vp(0.0), Length::vp(200.0), duration)
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

fn drive_scroll(
    observer: &Rc<ScrollObserver>,
    mut offset: Signal<f32>,
    mut clock: Signal<u64>,
    value: f32,
) {
    let at = clock().saturating_add(16_000_000);
    clock.set(at);
    observer.update_at(TimePoint::from_nanos(at), value);
    observer.flush_frame();
    offset.set(value);
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
    let snapshot = use_animation_snapshot(value.controls());
    let _ = snapshot();
    let current = value.get().clamp(0.0, 1.0);
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
            column {
                percent_width: 1.0,
                height: 28.0,
                background_color: 0xffe2e8f0u32,
                border_radius: 14.0,
                clip: true,
                column {
                    percent_width: current,
                    percent_height: 1.0,
                    background_color: 0xff4f46e5u32,
                    border_radius: 14.0,
                }
            }
            flex {
                margin_top: 12.0,
                percent_width: 1.0,
                flex_wrap: "wrap",
                ActionButton { label: "To 1.0", on_press: move |_| to_one.to(1.0) }
                ActionButton { label: "Retarget 0.35", on_press: move |_| retarget.retarget(0.35, TimeSpan::from_millis(360)) }
                ActionButton { label: "Repeat", on_press: move |_| repeat.animate_repeating(0.0, 1.0, TimeSpan::from_millis(1_200), Easing::Linear) }
                ActionButton { label: "Pause", on_press: move |_| pause.controls().pause() }
                ActionButton { label: "Set 0.65", on_press: move |_| set.set(0.65) }
                ActionButton { label: "Revert", on_press: move |_| revert.revert() }
                Metric { label: "Value", value: format!("{current:.3}") }
            }
        }
    }
}

fn next_time(clock: &Rc<Cell<u64>>) -> TimePoint {
    let next = clock.get().saturating_add(16_000_000);
    clock.set(next);
    TimePoint::from_nanos(next)
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
