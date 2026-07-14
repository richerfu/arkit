use arkit::prelude::*;

use crate::{cubic_out, restart_forward, reverse_and_play, target, ActionButton, Metric, Section};

const LAYOUT_TARGET: &str = "lab-layout-card";

#[component]
pub(crate) fn LifecycleLab() -> Element {
    rsx! {
        TransitionPresets {}
        PresenceDemo {}
        LayoutDemo {}
    }
}

#[component]
fn TransitionPresets() -> Element {
    let mut preset = use_signal(|| TransitionPreset::SlideUp);
    let mut replay = use_signal(|| 0_u64);
    let selected = preset();

    rsx! {
        Section {
            title: "Mount transitions",
            description: "All public TransitionPreset variants use the same Timeline/Controls path. Selecting the active preset also replays it.",
            column {
                percent_width: 1.0,
                height: 190.0,
                align_items: "center",
                justify_content: "center",
                background_color: 0xffe2e8f0u32,
                border_radius: 14.0,
                MountTransition {
                    preset: Some(selected),
                    duration_ms: Some(850),
                    replay_id: Some(replay()),
                    column {
                        width: 170.0,
                        height: 104.0,
                        align_items: "center",
                        justify_content: "center",
                        background_color: 0xff4f46e5u32,
                        border_radius: 20.0,
                        text { font_size: 15.0, font_weight: 700, font_color: 0xffffffffu32, "{selected:?}" }
                    }
                }
            }
            flex {
                margin_top: 12.0,
                percent_width: 1.0,
                flex_wrap: "wrap",
                for (candidate, label) in [
                    (TransitionPreset::Fade, "Fade"),
                    (TransitionPreset::SlideUp, "Up"),
                    (TransitionPreset::SlideDown, "Down"),
                    (TransitionPreset::SlideLeft, "Left"),
                    (TransitionPreset::SlideRight, "Right"),
                    (TransitionPreset::ZoomIn, "Zoom in"),
                    (TransitionPreset::ZoomOut, "Zoom out"),
                    (TransitionPreset::RotateClockwise, "Rotate +"),
                    (TransitionPreset::RotateCounterClockwise, "Rotate -"),
                ] {
                    ActionButton {
                        label,
                        active: selected == candidate,
                        on_press: move |_| {
                            preset.set(candidate);
                            replay += 1;
                        }
                    }
                }
                ActionButton { label: "Replay", on_press: move |_| replay += 1 }
            }
        }
    }
}

#[component]
fn PresenceDemo() -> Element {
    let mut mode = use_signal(|| PresenceMode::Wait);
    let selected = mode();
    rsx! {
        Section {
            title: "AnimatePresence lifecycle",
            description: "Sync, Wait and PopLayout retain leaving nodes until the real exit timeline reaches its terminal event; there is no timeout-based cleanup.",
            flex {
                percent_width: 1.0,
                flex_wrap: "wrap",
                for (candidate, label) in [
                    (PresenceMode::Sync, "Sync"),
                    (PresenceMode::Wait, "Wait"),
                    (PresenceMode::PopLayout, "PopLayout"),
                ] {
                    ActionButton {
                        label,
                        active: selected == candidate,
                        on_press: move |_| mode.set(candidate),
                    }
                }
            }
            KeyedPresenceBoard { mode: selected }
        }
    }
}

#[component]
fn KeyedPresenceBoard(mode: PresenceMode) -> Element {
    rsx! {
        column {
            key: "{mode:?}",
            percent_width: 1.0,
            PresenceBoard { mode }
        }
    }
}

#[component]
fn PresenceBoard(mode: PresenceMode) -> Element {
    let mut items = use_signal(|| vec![1_usize, 2, 3]);
    let mut next_id = use_signal(|| 4_usize);
    let presence = use_animate_presence(
        mode,
        items()
            .into_iter()
            .map(|value| (PresenceKey::new(format!("item-{value}")), value)),
    );
    let entries = presence.entries();

    rsx! {
        column {
            margin_top: 8.0,
            percent_width: 1.0,
            flex {
                percent_width: 1.0,
                height: 126.0,
                padding: 10.0,
                flex_wrap: "wrap",
                background_color: 0xfff8fafcu32,
                border_radius: 12.0,
                for entry in entries {
                    PresenceTile {
                        key: "{entry.key.as_str()}",
                        item_key: entry.key,
                        value: entry.value,
                        phase: entry.phase,
                        popped: entry.popped_from_layout,
                        delay_ms: entry.stagger_delay_ms,
                        on_terminal: {
                            let presence = presence.clone();
                            move |(key, phase)| match phase {
                                PresencePhase::Entering => { presence.mark_present(&key); }
                                PresencePhase::Leaving => { presence.settle_exit(&key); }
                                PresencePhase::Present => {}
                            }
                        }
                    }
                }
            }
            flex {
                margin_top: 10.0,
                percent_width: 1.0,
                flex_wrap: "wrap",
                ActionButton {
                    label: "Add",
                    on_press: move |_| {
                        let id = next_id();
                        next_id += 1;
                        let mut next = items();
                        next.push(id);
                        items.set(next);
                    }
                }
                ActionButton {
                    label: "Remove",
                    on_press: move |_| {
                        let mut next = items();
                        next.pop();
                        items.set(next);
                    }
                }
                ActionButton {
                    label: "Swap",
                    on_press: move |_| {
                        let id = next_id();
                        next_id += 1;
                        items.set(vec![id]);
                    }
                }
                ActionButton {
                    label: "Restore 3",
                    on_press: move |_| {
                        let start = next_id();
                        next_id += 3;
                        items.set(vec![start, start + 1, start + 2]);
                    }
                }
            }
        }
    }
}

#[component]
fn PresenceTile(
    item_key: PresenceKey,
    value: usize,
    phase: PresencePhase,
    popped: bool,
    delay_ms: u32,
    on_terminal: EventHandler<(PresenceKey, PresencePhase)>,
) -> Element {
    let name = format!("lab-presence-{}", item_key.as_str());
    let target_ready = use_animation_target(name.clone());
    let controls = use_animation(presence_timeline(&name, phase, delay_ms));
    let mut active = use_signal(|| None::<PresencePhase>);
    controls.on_complete(move || on_terminal.call((item_key.clone(), phase)));
    use_effect(use_reactive((&phase,), move |(phase,)| {
        if !target_ready.is_ready() || !controls.is_ready() || *active.peek() == Some(phase) {
            return;
        }
        active.set(Some(phase));
        if phase != PresencePhase::Present {
            controls.set_timeline(presence_timeline(&name, phase, delay_ms));
            controls.restart();
        }
    }));
    rsx! {
        column {
            margin: 5.0,
            width: 82.0,
            height: 82.0,
            align_items: "center",
            justify_content: "center",
            background_color: if popped { 0xfff97316u32 } else { 0xff0f766eu32 },
            border_radius: 18.0,
            text { font_size: 18.0, font_weight: 700, font_color: 0xffffffffu32, "{value}" }
            text { margin_top: 3.0, font_size: 9.0, font_color: 0xffccfbf1u32, "{phase:?}" }
        }
    }
}

fn presence_timeline(name: &str, phase: PresencePhase, delay_ms: u32) -> Timeline {
    let duration = TimeSpan::from_millis(if phase == PresencePhase::Leaving {
        320
    } else {
        420
    });
    let (from_opacity, to_opacity, from_x, to_x, from_y, to_y, from_scale, to_scale) = match phase {
        PresencePhase::Entering => (0.0, 1.0, 0.0, 0.0, 36.0, 0.0, 0.55, 1.0),
        PresencePhase::Present => (1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0),
        PresencePhase::Leaving => (1.0, 0.0, 0.0, 64.0, 0.0, -18.0, 1.0, 0.72),
    };
    let delay = TimeSpan::from_millis(u64::from(delay_ms));
    let animation = Animation::new(target(name))
        .tween(&OPACITY, from_opacity, to_opacity, duration)
        .configure_last(
            cubic_out(),
            Composition::Replace,
            Modifier::Identity,
            delay,
            0,
        )
        .tween(&TRANSLATE_X, Length::vp(from_x), Length::vp(to_x), duration)
        .configure_last(
            cubic_out(),
            Composition::Replace,
            Modifier::Identity,
            delay,
            0,
        )
        .tween(&TRANSLATE_Y, Length::vp(from_y), Length::vp(to_y), duration)
        .configure_last(
            cubic_out(),
            Composition::Replace,
            Modifier::Identity,
            delay,
            0,
        )
        .tween(&SCALE_X, from_scale, to_scale, duration)
        .configure_last(
            Easing::Spring(SpringSpec::default()),
            Composition::Replace,
            Modifier::Identity,
            delay,
            0,
        )
        .tween(&SCALE_Y, from_scale, to_scale, duration)
        .configure_last(
            Easing::Spring(SpringSpec::default()),
            Composition::Replace,
            Modifier::Identity,
            delay,
            0,
        );
    Timeline::new().add(animation, TimelinePosition::START)
}

#[component]
fn LayoutDemo() -> Element {
    let mut expanded = use_signal(|| false);
    let controls = use_animation(layout_timeline(false));
    let is_expanded = expanded();

    rsx! {
        Section {
            title: "FLIP layout projection",
            description: "LayoutEngine compares typed old/new snapshots, classifies the delta, and compiles inverse position/scale into the same animation Timeline.",
            row {
                percent_width: 1.0,
                height: 150.0,
                align_items: "center",
                justify_content: if is_expanded { "end" } else { "start" },
                padding: 12.0,
                background_color: 0xffe2e8f0u32,
                border_radius: 14.0,
                LayoutTarget { expanded: is_expanded }
            }
            flex {
                margin_top: 12.0,
                percent_width: 1.0,
                flex_wrap: "wrap",
                ActionButton {
                    label: "Toggle layout",
                    on_press: {
                        let controls = controls.clone();
                        move |_| {
                            let next = !expanded();
                            controls.set_timeline(layout_timeline(next));
                            expanded.set(next);
                            restart_forward(&controls);
                        }
                    }
                }
                ActionButton {
                    label: "Reverse + play",
                    on_press: {
                        let controls = controls.clone();
                        move |_| reverse_and_play(&controls, point_ms(620))
                    }
                }
                ActionButton { label: "Revert", on_press: { let controls = controls.clone(); move |_| controls.revert() } }
                LayoutRuntimeMetric { controls: controls.clone() }
                LayoutRegistryReadout {}
            }
        }
    }
}

fn point_ms(milliseconds: u64) -> TimePoint {
    TimePoint::from_nanos(milliseconds * 1_000_000)
}

#[component]
fn LayoutRuntimeMetric(controls: AnimationControls) -> Element {
    let snapshot = use_animation_snapshot(&controls);
    let status = snapshot()
        .map(|value| {
            format!(
                "{:?} · {:.0}ms",
                value.state,
                value.elapsed.as_nanos() as f64 / 1_000_000.0
            )
        })
        .unwrap_or_else(|| {
            if controls.is_ready() {
                "Ready".to_string()
            } else {
                "Resolving".to_string()
            }
        });
    rsx! { Metric { label: "Engine", value: status } }
}

#[component]
fn LayoutTarget(expanded: bool) -> Element {
    let _target = use_animation_target(LAYOUT_TARGET);
    use_animation_layout(LayoutId::owned("lab-layout-node"), None, true, 1);
    rsx! {
        column {
            width: if expanded { 184.0 } else { 118.0 },
            height: if expanded { 112.0 } else { 78.0 },
            align_items: "center",
            justify_content: "center",
            background_color: if expanded { 0xff7c3aedu32 } else { 0xff0891b2u32 },
            border_radius: if expanded { 28.0 } else { 14.0 },
            text { font_size: 13.0, font_weight: 700, font_color: 0xffffffffu32, if expanded { "Expanded" } else { "Compact" } }
        }
    }
}

#[component]
fn LayoutRegistryReadout() -> Element {
    let snapshot = use_layout_snapshot(AnimationWindowMetrics {
        width_vp: 360.0,
        height_vp: 720.0,
        density: 1.0,
    });
    rsx! {
        Metric {
            label: "Registry",
            value: format!("gen {} · {} nodes", snapshot.generation, snapshot.nodes.len())
        }
    }
}

fn layout_timeline(expanded: bool) -> Timeline {
    let mut engine = LayoutEngine::default();
    engine.record_old(layout_snapshot(!expanded, 1));
    let delta = engine
        .record_new(layout_snapshot(expanded, 2))
        .into_iter()
        .find(|delta| delta.id.as_str() == "lab-layout-node")
        .expect("layout node exists in both snapshots");
    delta
        .timeline(
            TargetName::owned(LAYOUT_TARGET),
            LayoutAnimationMode::PositionAndSize,
            TimeSpan::from_millis(620),
            Easing::Spring(SpringSpec::default()),
        )
        .expect("a retained layout node produces a timeline")
}

fn layout_snapshot(expanded: bool, generation: u64) -> LayoutSnapshot {
    let mut snapshot = LayoutSnapshot::new(
        AnimationWindowMetrics {
            width_vp: 360.0,
            height_vp: 720.0,
            density: 1.0,
        },
        generation,
    );
    let root = snapshot.push(LayoutNode {
        id: LayoutId::owned("lab-layout-root"),
        parent: None,
        frame: LayoutFrame {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 150.0,
        },
        transform: TransformValue::default(),
        visible: true,
        clip: None,
        z_order: 0,
        mount_state: LayoutMountState::Mounted,
    });
    snapshot.push(LayoutNode {
        id: LayoutId::owned("lab-layout-node"),
        parent: Some(root),
        frame: if expanded {
            LayoutFrame {
                x: 136.0,
                y: 19.0,
                width: 184.0,
                height: 112.0,
            }
        } else {
            LayoutFrame {
                x: 0.0,
                y: 36.0,
                width: 118.0,
                height: 78.0,
            }
        },
        transform: TransformValue::default(),
        visible: true,
        clip: None,
        z_order: 1,
        mount_state: LayoutMountState::Mounted,
    });
    snapshot
}
