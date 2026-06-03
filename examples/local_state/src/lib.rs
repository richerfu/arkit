//! 本示例展示 arkit 中三种状态管理模式：
//!
//! 1. 全局状态（Elm 模式）—— 应用级 AppState + update + view
//! 2. 组件本地状态（非受控）—— Widget 通过 tree.state() 自管理内部状态
//! 3. 持久化 key —— 子组件在条件渲染/切换中保持状态不丢失

use arkit::entry;
use arkit::prelude::*;
use arkit::{application, Element, Task};
use std::cell::RefCell;
use std::rc::Rc;

// ── 全局消息 ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Message {
    /// 全局计数器递增
    Increment,
    /// 全局计数器递减
    Decrement,
    /// 切换面板可见性（演示 persistent_key 保持状态）
    TogglePanel,
}

// ── 全局状态 ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
struct AppState {
    global_count: i32,
    panel_visible: bool,
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Increment => state.global_count += 1,
        Message::Decrement => state.global_count -= 1,
        Message::TogglePanel => state.panel_visible = !state.panel_visible,
    }
    Task::none()
}

fn view(state: &AppState) -> Element<Message> {
    let mut children = vec![
        // 标题
        text("arkit local state demo")
            .font_size(28.0)
            .font_weight(FontWeight::W600)
            .line_height(32.0)
            .into(),
        text("Each counter below manages state differently")
            .font_size(14.0)
            .line_height(20.0)
            .margin_top(8.0)
            .into(),
        // ── 模式 1：全局状态计数器 ─────────────────────────────────────
        text("① Global State (Elm pattern)")
            .font_size(16.0)
            .font_weight(FontWeight::W500)
            .line_height(22.0)
            .margin_top(20.0)
            .into(),
        text("State lives in AppState, updated via Message")
            .font_size(12.0)
            .line_height(18.0)
            .into(),
        row_component()
            .margin_top(8.0)
            .align_items_center()
            .children(vec![
                button("−")
                    .padding([8.0, 14.0, 8.0, 14.0])
                    .on_press(Message::Decrement)
                    .into(),
                text(format!(" {} ", state.global_count))
                    .font_size(20.0)
                    .font_weight(FontWeight::W600)
                    .into(),
                button("+")
                    .padding([8.0, 14.0, 8.0, 14.0])
                    .on_press(Message::Increment)
                    .into(),
            ])
            .into(),
        // ── 模式 2：组件本地状态（两个独立实例）──────────────────────
        text("② Local State (uncontrolled widget)")
            .font_size(16.0)
            .font_weight(FontWeight::W500)
            .line_height(22.0)
            .margin_top(20.0)
            .into(),
        text("Each widget owns its state via tree.state()")
            .font_size(12.0)
            .line_height(18.0)
            .into(),
        row_component()
            .margin_top(8.0)
            .children(vec![
                local_counter("Counter A"),
                row_component()
                    .margin_left(16.0)
                    .children(vec![local_counter("Counter B")])
                    .into(),
            ])
            .into(),
        // ── 模式 3：持久化 key ───────────────────────────────────────
        text("③ Persistent Key (survives toggle)")
            .font_size(16.0)
            .font_weight(FontWeight::W500)
            .line_height(22.0)
            .margin_top(20.0)
            .into(),
        text("Toggle the panel — the counter keeps its value")
            .font_size(12.0)
            .line_height(18.0)
            .into(),
        button(if state.panel_visible {
            "Hide Panel"
        } else {
            "Show Panel"
        })
        .padding([8.0, 16.0, 8.0, 16.0])
        .margin_top(8.0)
        .on_press(Message::TogglePanel)
        .into(),
    ];

    // 条件渲染面板—— persistent_key 保证状态不丢失
    if state.panel_visible {
        children.push(persistent_counter());
    } else {
        children.push(
            text("(panel hidden — state preserved)")
                .font_size(12.0)
                .line_height(18.0)
                .margin_top(4.0)
                .into(),
        );
    }

    column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .padding(24.0)
        .children(children)
        .into()
}

// ══════════════════════════════════════════════════════════════════════════════
// 模式 2：组件本地状态（非受控）
// ══════════════════════════════════════════════════════════════════════════════

/// 创建一个自带本地状态的计数器 widget。
///
/// 关键原理：
/// - 每个调用点产生不同的 `Widget` 实例（不同的 `Tag`）
/// - `Widget::state()` 初始化 `Rc<RefCell<i32>>` 存入 `widget::Tree`
/// - `Widget::body()` 从 tree 中读取当前值来渲染
/// - `on_click` 回调中直接修改本地状态 + 调用 `request_widget_rerender()`
/// - 整个过程不经过全局 `update()` 函数
///
/// 由于每次调用 `local_counter()` 创建的是独立的 `LocalCounterWidget<M>` 实例，
/// 每个 widget 的 `Tag` 由其 `TypeId` 唯一标识，因此各自拥有独立的 `widget::Tree`
/// 节点和本地状态。
fn local_counter<M: 'static>(label: &str) -> Element<M> {
    Element::new(LocalCounterWidget {
        label: label.to_string(),
        _marker: std::marker::PhantomData,
    })
}

struct LocalCounterWidget<M> {
    label: String,
    _marker: std::marker::PhantomData<M>,
}

impl<M: 'static> arkit::advanced::Widget<M, arkit::Theme, arkit::Renderer>
    for LocalCounterWidget<M>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Element<M> {
        // 从 widget tree 获取或初始化本地状态
        let count: Rc<RefCell<i32>> = tree
            .state()
            .get_or_insert_with(|| Rc::new(RefCell::new(0i32)))
            .clone();

        let value = *count.borrow();
        let count_up = count.clone();
        let count_down = count.clone();

        column_component()
            .children(vec![
                text(&self.label)
                    .font_size(14.0)
                    .font_weight(FontWeight::W500)
                    .into(),
                row_component()
                    .margin_top(4.0)
                    .align_items_center()
                    .children(vec![
                        button("−")
                            .padding([6.0, 12.0, 6.0, 12.0])
                            // 使用 on_click（接收闭包）而非 on_press（接收 Message）
                            .on_click(move || {
                                *count_down.borrow_mut() -= 1;
                                request_widget_rerender();
                            })
                            .into(),
                        text(format!(" {} ", value))
                            .font_size(20.0)
                            .font_weight(FontWeight::W600)
                            .into(),
                        button("+")
                            .padding([6.0, 12.0, 6.0, 12.0])
                            .on_click(move || {
                                *count_up.borrow_mut() += 1;
                                request_widget_rerender();
                            })
                            .into(),
                    ])
                    .into(),
            ])
            .into()
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// 模式 3：持久化 key —— 子组件在条件渲染中保持状态
// ══════════════════════════════════════════════════════════════════════════════

/// 创建一个带 `persistent_state_key` 的计数器。
///
/// 关键原理：
/// - `persistent_state_key("persistent-counter")` 给 Node 打上稳定标识
/// - 当父级条件渲染导致该 widget 被移除时，`StateCache` 会按
///   `(tag, persistent_key)` 缓存其 `widget::Tree`
/// - 重新挂载时，`StateCache::take()` 从缓存恢复 Tree，本地状态完整保留
fn persistent_counter<M: 'static>() -> Element<M> {
    Element::new(PersistentCounterWidget {
        _marker: std::marker::PhantomData,
    })
}

struct PersistentCounterWidget<M> {
    _marker: std::marker::PhantomData<M>,
}

impl<M: 'static> arkit::advanced::Widget<M, arkit::Theme, arkit::Renderer>
    for PersistentCounterWidget<M>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Element<M> {
        let count: Rc<RefCell<i32>> = tree
            .state()
            .get_or_insert_with(|| Rc::new(RefCell::new(0i32)))
            .clone();

        let value = *count.borrow();
        let count_up = count.clone();
        let count_down = count.clone();

        // persistent_state_key 是关键！
        // 当这个 column 因为父级 TogglePanel 被移除时，
        // StateCache 会按 key 缓存其 Tree；重新出现时自动恢复。
        column_component()
            .persistent_state_key("persistent-counter")
            .margin_top(8.0)
            .padding(12.0)
            .background_color(0xFFF0F0F0)
            .border_radius(8.0)
            .children(vec![
                text("Persistent Counter")
                    .font_size(14.0)
                    .font_weight(FontWeight::W500)
                    .into(),
                text("Toggle visibility — value survives!")
                    .font_size(11.0)
                    .line_height(16.0)
                    .into(),
                row_component()
                    .margin_top(4.0)
                    .align_items_center()
                    .children(vec![
                        button("−")
                            .padding([6.0, 12.0, 6.0, 12.0])
                            .on_click(move || {
                                *count_down.borrow_mut() -= 1;
                                request_widget_rerender();
                            })
                            .into(),
                        text(format!(" {} ", value))
                            .font_size(20.0)
                            .font_weight(FontWeight::W600)
                            .into(),
                        button("+")
                            .padding([6.0, 12.0, 6.0, 12.0])
                            .on_click(move || {
                                *count_up.borrow_mut() += 1;
                                request_widget_rerender();
                            })
                            .into(),
                    ])
                    .into(),
            ])
            .into()
    }
}

// ── 工具函数 ──────────────────────────────────────────────────────────────────

/// 请求 widget 级 re-render（不经过全局 update）。
/// 等效于 arkit_shadcn 中的 `request_widget_rerender()`。
fn request_widget_rerender() {
    arkit::internal::queue_ui_loop(|| {
        if let Some(runtime) = arkit::internal::current_runtime() {
            runtime.request_rerender();
        }
    });
}

// ── 入口 ──────────────────────────────────────────────────────────────────────

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(AppState::default, update, view)
}
