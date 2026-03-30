# arkit 设计分析报告

## 一、架构总览

arkit 是一个基于 SolidJS 细粒度响应式模型 + ArkUI C API 的 Rust UI 框架。核心设计理念：

- **渲染一次，信号驱动更新** — 无虚拟 DOM diff
- **自动依赖追踪** — 信号读取自动注册依赖
- **层级化 Owner 树** — 管理生命周期和上下文传播
- **直接操作原生节点** — 跳过 diff，直接 setAttribute

## 二、设计亮点（值得保留）

1. **SolidJS 的响应式模型忠实地移植到了 Rust** — `Signal`/`create_effect`/`create_memo`/`batch`/`untrack` 与 SolidJS 的原语一一对应
2. **Owner 树的层级化清理** — 子 scope 先于父 scope 清理，cleanups 按逆序执行
3. **控制流自管理** — `show`/`for_each`/`dynamic` 通过 `queue_ui_loop` 在 ArkUI 事件循环中执行 DOM 变更，避免了响应式计算中的重入问题
4. **两级调度** — 同步 `batch` + 异步 `UI_LOOP_EFFECTS` 分离了响应式批处理和平台调度

## 三、核心设计问题与优化点

### 问题 1：`watch_signal` 存在响应式短路 — 脱离了 Effect 追踪体系

**文件**: `view/core.rs:224-269`

```rust
pub fn watch_signal<S>(self, signal: Signal<S>, apply: ...) -> Self {
    let subscription_id = signal.subscribe(move || { ... }); // 直接 subscribe
}
```

**问题**: `watch_signal` 通过 `signal.subscribe()` 手动订阅，完全绕过了 `Computation` 追踪体系。这意味着：
- 不受 `batch()` 管理 — 订阅回调直接执行，不经过 `PENDING_EFFECTS` 队列
- 不受 `Owner` 管理 — 订阅清理依赖手动 `unsubscribe`，而非 `Computation::dispose()`
- 与 `create_effect` 的语义不一致 — 同一个框架中存在两种响应式绑定机制

**建议**: `watch_signal` 应该内部使用 `create_effect` 来追踪信号变更，而不是手动 `subscribe`。

---

### 问题 2：Signal 的 `notify()` 在写入时立即执行所有订阅者，缺乏调度

**文件**: `signal.rs:109-117`

```rust
fn notify(&self) {
    let subscribers = { /* clone snapshot */ };
    for callback in subscribers {
        callback();  // 同步立即执行
    }
}
```

**问题**: 虽然有 `batch()` 机制，但它只对 `Computation::mark_dirty()` 生效。对于 `watch_signal` 中直接 `subscribe` 的回调，`notify()` 会立即同步执行所有订阅者。

**建议**: 统一信号传播路径，使所有通知都经过 `mark_dirty` → `PENDING_EFFECTS` 或 `UI_LOOP_EFFECTS` 调度。

---

### 问题 3：控制流组件创建了不必要的 Column 容器节点

**文件**: `control_flow.rs`、`scope.rs`

每个 `show`、`for_each`、`dynamic`、`scope` 都创建了一个真实的 ArkUI `Column` 节点作为容器。在 ArkUI 的渲染管线中，每个额外的布局节点都会参与 measure/layout 两次遍历。

SolidJS 的控制流是**无容器**的 — 直接操作子节点的挂载/卸载，不引入额外的 DOM 节点。

**建议**: 利用 ArkUI C API 的 `insertChildAt`/`removeChild`/`insertChildAfter` 直接操作父节点的子列表。

---

### 问题 4：`reconcile_children` 的 "先全部移除再重新插入" 策略

**文件**: `view/core.rs:693-727`

对于 key 匹配的可复用子节点，算法先从父节点中移除，然后重新插入。这会导致 ArkUI 对这些节点触发 detach/reattach 和不必要的布局重计算。

**建议**: 引入 LIS 最长递增子序列算法，只移动真正需要移动的节点。

---

### 问题 5：`ComponentElement` 的 `patch()` 路径每次都重新执行 effects

**文件**: `view/core.rs:819-858`

每次 patch 时，所有 effects 都会重新执行，包括事件处理器注册。事件处理器通常是稳定的，不需要在每次 patch 时重新注册。

**建议**: 将 effects 分为 mount-only 和 patchable 两类。

---

### 问题 6：`Rc<RefCell<...>>` 模式过多

整个响应式系统大量使用 `Rc<RefCell<...>>`，带来运行时 panic 风险和循环引用风险。

---

### 问题 7：`for_each` 的协调效率 — 全量 O(n²) key 比较

**文件**: `control_flow.rs:252-283`

`current_keys.contains(key)` 对每个新元素做 O(n) 线性搜索。

**建议**: 使用 `HashMap` 替代线性搜索。

---

### 问题 8：`#[component]` 宏是空操作，缺少自动 scope 包装

**文件**: `arkit_derive/src/lib.rs`

组件内部创建的 signals/effects 的生命周期不受组件边界约束。

**建议**: `#[component]` 应自动包装 `with_child_owner`。

---

### 问题 9：ArkUI 节点 Dispose 语义不清晰

可复用节点在 `reconcile_children` 中也被 `remove_child` + `dispose`，导致被意外销毁。

**建议**: 明确区分 "移除但保留" 和 "移除并销毁"。

---

### 问题 10：`notify()` 克隆所有订阅者引用再遍历

每次 signal 值变更时都克隆所有 `Rc<dyn Fn()>`，高频场景下产生不必要的内存分配。

---

### 问题 11：缺少 `on_cleanup` 与组件卸载的自动绑定

普通组件（非 `scope()`）内部创建的 effect/signal 的 cleanup 依赖于组件在树中的位置。

**建议**: 每个 `ComponentElement` 的 `mount()` 自动在 `with_child_owner` 中执行。

---

### 问题 12：未利用 ArkUI NodeAdapter 进行列表虚拟化

`for_each` 一次性创建所有子节点，大列表场景下性能差。

---

## 四、修改优先级

| 优先级 | 问题 | 影响 |
|---|---|---|
| **P0** | 问题 1: watch_signal 绕过追踪体系 | 语义不一致，batch 失效 |
| **P0** | 问题 8: #[component] 无自动 scope | 内存泄漏风险 |
| **P1** | 问题 5: patch 重执行 mount-only effects | 事件处理器重复注册 |
| **P1** | 问题 7: for_each O(n²) key 比较 | 大列表性能 |
| **P1** | 问题 9: 节点 dispose 语义错误 | 可复用节点被意外销毁 |
| **P2** | 问题 3: 控制流引入多余 Column | 性能（布局节点膨胀）|
| **P2** | 问题 4: reconcile 先删后插策略 | 不必要的 detach/reattach |
| **P2** | 问题 2: Signal notify 缺乏统一调度 | 一致性 |
| **P2** | 问题 10: notify 克隆所有订阅者 | 高频场景性能 |
| **P3** | 问题 6: Rc<RefCell> 过多 | 运行时 panic 风险 |
| **P3** | 问题 11: 组件无自动 Owner | 生命周期管理复杂 |
| **P3** | 问题 12: 未用 NodeAdapter | 大列表场景必需 |
