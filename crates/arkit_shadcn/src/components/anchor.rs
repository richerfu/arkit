//! Anchor — 锚点导航容器。
//!
//! 长内容页中，点击导航项让右侧 `Scroll` 滚动到对应区块顶部，滚动过程中
//! 自动高亮当前可见区块（scrollspy）。
//!
//! [`Anchor`] 渲染 `row { nav, scroll }`：`nav` 通常是一列 [`AnchorItem`]，
//! `children` 内放置一组 [`AnchorSection`]。区块位置通过
//! `use_native_element_ref` + `use_layout_frame` 测量（物理像素、窗口相对），
//! 滚动使用 `<scroll>` 的 `scroll_offset` 一次性命令（vp），经
//! `WindowMetricsHandle.scale` 换算；`onscroll` 的 vp 增量累积为当前位置，
//! 与注册的区块帧比较得出激活项。

use std::cell::RefCell;
use std::rc::Rc;

use super::button::{Button, ButtonVariant};
use arkit_prelude::*;
use arkit_runtime::{RuntimeHandle, WindowMetricsHandle};

const SCROLL_BAR_DEFAULT: &str = "auto";
/// 判定"已滚到底部"的像素容差，用于回退选中最后一个区块。
const BOTTOM_EPSILON_PX: f32 = 1.0;

/// 一次 `scroll_offset` 跳转命令。
#[derive(Clone, Copy, PartialEq)]
struct ScrollJump {
    y_vp: f32,
    duration: u32,
}

/// Anchor 共享状态。经 `use_context_provider` 只提供一次，任何可变状态
/// 都通过内部可变性访问（dioxus 0.7 的 context 不传播更新）。
struct AnchorInner {
    frames: Rc<RefCell<Vec<(String, arkit_hooks::LayoutFrame)>>>,
    revision: Signal<u64>,
    scroll_frame: Signal<arkit_hooks::LayoutFrame>,
    position: Signal<f32>,
    command: Signal<Option<ScrollJump>>,
    active: Memo<Option<String>>,
    metrics: Option<WindowMetricsHandle>,
    runtime: RuntimeHandle,
    duration: u32,
}

impl AnchorInner {
    fn scale(&self) -> f32 {
        self.metrics
            .as_ref()
            .map(|metrics| metrics.get().scale)
            .unwrap_or(1.0)
            .max(f32::EPSILON)
    }

    fn update_section(&self, id: &str, frame: arkit_hooks::LayoutFrame) {
        let mut frames = self.frames.borrow_mut();
        if let Some((_, current)) = frames.iter_mut().find(|(registered, _)| registered == id) {
            if *current == frame {
                return;
            }
            *current = frame;
        } else {
            frames.push((id.to_owned(), frame));
        }
        drop(frames);
        self.bump_revision();
    }

    fn remove_section(&self, id: &str) {
        let removed = {
            let mut frames = self.frames.borrow_mut();
            let before = frames.len();
            frames.retain(|(registered, _)| registered != id);
            frames.len() != before
        };
        if removed {
            self.bump_revision();
        }
    }

    fn bump_revision(&self) {
        let mut revision = self.revision;
        let next = (*revision.peek()).wrapping_add(1);
        revision.set(next);
    }
}

/// Anchor 上下文句柄，通过 [`use_anchor`] 获取。
#[derive(Clone)]
pub struct AnchorContext {
    inner: Rc<AnchorInner>,
}

impl AnchorContext {
    /// 滚动到指定区块顶部。
    ///
    /// 区块或滚动容器尚未完成测量时静默 no-op（真实场景中点击必然发生在布局之后）。
    pub fn jump(&self, id: &str) {
        let inner = &self.inner;
        let Some(frame) = inner
            .frames
            .borrow()
            .iter()
            .find(|(registered, _)| registered == id)
            .map(|(_, frame)| *frame)
            .filter(|frame| frame.is_measured())
        else {
            return;
        };
        let scroll = inner.scroll_frame.peek();
        if !scroll.is_measured() {
            return;
        }
        let jump = ScrollJump {
            y_vp: ((frame.y - scroll.y).max(0.0) / inner.scale()).max(0.0),
            duration: inner.duration,
        };
        let mut command = inner.command;
        command.set(Some(jump));
        // `scroll_offset` 只在属性值变化时触发一次。下一 UI tick 的
        // `run_ui_effects`（先于渲染）把命令重置为 `None`，保证同位置
        // 重复点击也能经 None→Some 转换再次触发。
        inner.runtime.queue_ui(move || {
            command.set(None);
        });
    }

    /// 当前激活区块 id（随滚动联动，无命中时为 `None`）。
    pub fn active_id(&self) -> Option<String> {
        self.inner.active.read().clone()
    }

    /// 当前滚动位置（vp）。
    pub fn scroll_position(&self) -> f32 {
        let position = self.inner.position;
        position()
    }
}

/// 读取最近的 [`Anchor`] 上下文（必须在 Anchor 的子树内调用）。
pub fn use_anchor() -> Option<AnchorContext> {
    try_use_context::<AnchorContext>()
}

/// 计算当前激活区块：最后一个"顶边已进入视口阈值"的区块；
/// 滚动到底部且没有任何区块命中时，回退到最后一个区块。
///
/// 所有帧均为窗口相对物理像素；`position_px` 是滚动视口顶部相对内容
/// 顶部的偏移。
fn active_section(
    sections: &[(String, arkit_hooks::LayoutFrame)],
    scroll: arkit_hooks::LayoutFrame,
    position_px: f32,
    threshold_px: f32,
) -> Option<String> {
    let measured: Vec<&(String, arkit_hooks::LayoutFrame)> = sections
        .iter()
        .filter(|(_, frame)| frame.is_measured())
        .collect();
    if measured.is_empty() || !scroll.is_measured() {
        return None;
    }

    let mut candidate: Option<&String> = None;
    for (id, frame) in &measured {
        if frame.y - scroll.y <= position_px + threshold_px {
            candidate = Some(id);
        }
    }
    if let Some(id) = candidate {
        return Some(id.clone());
    }

    // 已滚到底部：最后一个区块的顶边从未进入阈值时（内容不够高），回退到它。
    let content_bottom_px = measured
        .iter()
        .map(|(_, frame)| frame.y + frame.height - scroll.y)
        .fold(f32::MIN, f32::max);
    let max_scroll_px = (content_bottom_px - scroll.height).max(0.0);
    if max_scroll_px > 0.0 && position_px >= max_scroll_px - BOTTOM_EPSILON_PX {
        return measured.last().map(|(id, _)| id.clone());
    }
    None
}

fn sanitize_offset(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

/// Props for [`Anchor`].
#[derive(Props, Clone, PartialEq)]
pub struct AnchorProps {
    /// 左侧导航内容（通常是一列 [`AnchorItem`]）。
    pub nav: Element,
    /// 右侧滚动内容（通常是一组 [`AnchorSection`]）。
    pub children: Element,
    /// 外部 scroll ref；缺省时 Anchor 自建。传入后调用方可以额外观察滚动容器。
    pub scroll_ref: Option<arkit_arkui::NativeElementRef>,
    /// 滚动条策略：`"off"` / `"auto"` / `"on"`，缺省 `"auto"`。
    pub scroll_bar: Option<String>,
    /// 跳转动画时长（ms），0 = 瞬跳。
    #[props(default)]
    pub scroll_duration: u32,
    /// 区块判定为"已进入视口"的阈值（vp），避免区块边界闪烁。
    #[props(default)]
    pub active_threshold: f32,
}

/// 锚点导航容器：左侧导航 + 右侧滚动内容，滚动联动高亮。
///
/// `scroll_duration` / `active_threshold` 只在挂载时生效，运行期修改不
/// 会回传上下文（与 `Guide` 的 `style` 行为一致）。
#[component]
pub fn Anchor(props: AnchorProps) -> Element {
    let internal_ref = arkit_hooks::use_native_element_ref();
    let scroll_ref = props
        .scroll_ref
        .clone()
        .unwrap_or_else(|| internal_ref.clone());
    let runtime = arkit_runtime::use_runtime_handle();
    let metrics = dioxus_core::try_consume_context::<WindowMetricsHandle>();

    let frames = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    let revision = use_signal(|| 0u64);
    let scroll_frame = use_signal(arkit_hooks::LayoutFrame::default);
    let position = use_signal(|| 0.0f32);
    let command = use_signal(|| None::<ScrollJump>);
    let threshold_vp = props.active_threshold;

    let active = {
        let frames = frames.clone();
        let metrics = metrics.clone();
        use_memo(move || {
            revision();
            let scale = metrics
                .as_ref()
                .map(|metrics| metrics.get().scale)
                .unwrap_or(1.0)
                .max(f32::EPSILON);
            let sections = frames.borrow();
            active_section(
                &sections,
                scroll_frame(),
                position() * scale,
                threshold_vp * scale,
            )
        })
    };

    let context = use_context_provider(move || AnchorContext {
        inner: Rc::new(AnchorInner {
            frames,
            revision,
            scroll_frame,
            position,
            command,
            active,
            metrics,
            runtime,
            duration: props.scroll_duration,
        }),
    });

    let scroll_frame = context.inner.scroll_frame;
    arkit_hooks::use_layout_frame(scroll_ref.clone(), move |frame| {
        let mut scroll_frame = scroll_frame;
        scroll_frame.set(frame);
    });

    let command = context.inner.command;
    let offset = command().map(|jump| format!("0,{},{}", jump.y_vp, jump.duration));
    let scroll_bar = props
        .scroll_bar
        .clone()
        .unwrap_or_else(|| SCROLL_BAR_DEFAULT.to_string());
    let position = context.inner.position;
    let children = props.children;
    let nav = props.nav;

    rsx! {
        row {
            width: "100%",
            height: "100%",
            {nav}
            scroll {
                native_ref: scroll_ref,
                width: "100%",
                layout_weight: 1.0,
                alignment: "top-start",
                scroll_bar: scroll_bar,
                scroll_enabled: true,
                scroll_offset: offset,
                onscroll: move |event| {
                    let data = *event.data();
                    if data.has_offset {
                        let mut position = position;
                        position.set(sanitize_offset(position() + data.offset_y));
                    }
                },
                {children}
            }
        }
    }
}

/// Props for [`AnchorSection`].
#[derive(Props, Clone, PartialEq)]
pub struct AnchorSectionProps {
    /// 区块 id，需在最近的 [`Anchor`] 内唯一。
    pub id: String,
    pub children: Element,
}

/// 标记一个可跳转的内容区块：包一层 `column { width: "100%" }` 并注册
/// 其测量位置到最近的 [`Anchor`]，卸载时自动注销。
#[component]
pub fn AnchorSection(props: AnchorSectionProps) -> Element {
    let section_ref = arkit_hooks::use_native_element_ref();
    let context = use_anchor();
    let frame_registry = context.clone();
    let frame_id = props.id.clone();
    arkit_hooks::use_layout_frame(section_ref.clone(), move |frame| {
        if let Some(context) = frame_registry.as_ref() {
            context.inner.update_section(&frame_id, frame);
        }
    });
    let drop_registry = context;
    let drop_id = props.id;
    use_drop(move || {
        if let Some(context) = drop_registry.as_ref() {
            context.inner.remove_section(&drop_id);
        }
    });
    rsx! {
        column {
            native_ref: section_ref,
            width: "100%",
            {props.children}
        }
    }
}

/// Props for [`AnchorItem`].
#[derive(Props, Clone, PartialEq)]
pub struct AnchorItemProps {
    /// 对应 [`AnchorSection`] 的 id。
    pub id: String,
    /// 导航文案。
    pub title: String,
    /// 手动指定激活态；缺省时由滚动位置自动计算。
    pub active: Option<bool>,
    #[props(default)]
    pub onclick: Option<EventHandler<()>>,
}

/// 单个锚点导航项。点击滚动到对应区块；随滚动自动高亮当前可见区块。
#[component]
pub fn AnchorItem(props: AnchorItemProps) -> Element {
    let context = use_anchor();
    let computed_active = context
        .as_ref()
        .and_then(AnchorContext::active_id)
        .as_deref()
        == Some(props.id.as_str());
    let active = props.active.unwrap_or(computed_active);
    let variant = if active {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Ghost
    };
    let id = props.id.clone();
    let title = props.title.clone();
    let onclick = props.onclick;
    let on_press = move |_| {
        if let Some(context) = context.as_ref() {
            context.jump(&id);
        }
        if let Some(handler) = onclick {
            handler.call(());
        }
    };

    rsx! {
        Button {
            variant: variant,
            width: "100%",
            onclick: on_press,
            "{title}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::active_section;
    use arkit_hooks::LayoutFrame;

    fn frame(y: f32, height: f32) -> LayoutFrame {
        LayoutFrame {
            x: 0.0,
            y,
            width: 400.0,
            height,
        }
    }

    #[test]
    fn none_before_any_section_enters_threshold() {
        let sections = vec![
            ("intro".to_string(), frame(500.0, 300.0)),
            ("install".to_string(), frame(900.0, 300.0)),
        ];
        assert_eq!(
            active_section(&sections, frame(100.0, 600.0), 150.0, 0.0),
            None
        );
    }

    #[test]
    fn first_section_active_after_its_top_passes_threshold() {
        let sections = vec![
            ("intro".to_string(), frame(500.0, 300.0)),
            ("install".to_string(), frame(900.0, 300.0)),
        ];
        assert_eq!(
            active_section(&sections, frame(100.0, 600.0), 450.0, 0.0),
            Some("intro".to_string())
        );
    }

    #[test]
    fn latest_section_past_threshold_wins() {
        let sections = vec![
            ("intro".to_string(), frame(500.0, 300.0)),
            ("install".to_string(), frame(900.0, 300.0)),
            ("api".to_string(), frame(1300.0, 300.0)),
        ];
        assert_eq!(
            active_section(&sections, frame(100.0, 600.0), 850.0, 0.0),
            Some("install".to_string())
        );
        assert_eq!(
            active_section(&sections, frame(100.0, 600.0), 1300.0, 0.0),
            Some("api".to_string())
        );
    }

    #[test]
    fn bottom_falls_back_to_last_section() {
        // 内容底 600，视口 400 → 最大滚动 200；滚到底时 intro 顶边(400)仍未进入阈值。
        let sections = vec![("intro".to_string(), frame(500.0, 200.0))];
        assert_eq!(
            active_section(&sections, frame(100.0, 400.0), 200.0, 0.0),
            Some("intro".to_string())
        );
    }

    #[test]
    fn unmeasured_frames_yield_none() {
        let sections = vec![("intro".to_string(), LayoutFrame::default())];
        assert_eq!(
            active_section(&sections, frame(100.0, 600.0), 450.0, 0.0),
            None
        );
        assert_eq!(
            active_section(&[], LayoutFrame::default(), 450.0, 0.0),
            None
        );
    }
}
