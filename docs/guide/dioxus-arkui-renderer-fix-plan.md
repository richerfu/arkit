# Dioxus ArkUI Renderer 问题分析与修复计划

> 状态（2026-07-10）：HostTree/projection、mounted-wrapper remap、Dioxus scheduler waker、queued native event dispatch 和 LongPress gesture bridge 已实现。本文保留为架构决策与验收契约；“当前问题”描述的是迁移初期状态，后续章节记录最终规则而不是兼容旧实现的临时方案。

## 背景

`arkit_arkui` 使用 Dioxus `WriteMutations` 把 VirtualDom mutation 应用到 ArkUI native node。迁移初期 counter 示例只能渲染部分 UI，并暴露了以下问题：

- inspect 看到很多节点被渲染成 `Stack`，而不是真实的 `Column` / `Button` / `Text`。
- `Text` 和 `Button` 的文字显示异常，出现缺失、重叠或位置不在展示区域内的情况。
- `Button` 点击事件无效或不稳定。
- 重新打包安装后，`Button` 可以出现，但 `count = 0` 文本仍可能缺失。

截图里的斜线和圆点是截图工具问题，不属于 renderer 问题。

## 结论

这不是简单的“Stack 布局太多”问题。`Stack` 泛滥是一个独立 bug，但不是全部根因。

完整根因是当前 renderer 没有完整表达 Dioxus 的 RealDOM 语义。它直接把 mutation 栈上的节点映射为 ArkUI native node，导致 Dioxus 的 text node、placeholder、template path、ElementId 和 ArkUI native projection 混在一起处理。

正确方向不是把 Dioxus 语义“转回旧实现”，而是建立一个 Dioxus-aligned host tree，并把这个 host tree 投影到 ArkUI native tree。

## 旧实现与当前实现的关键差异

旧 `arkit_widget::render_impl` 是自有 Element 树：

- `text("xxx")` 直接设置 ArkUI `TextContent` 属性。
- `button("xxx")` 直接设置 ArkUI `ButtonLabel` 属性。
- mount 时递归创建真实 ArkUI node。
- patch 时通过 `parent.children()[index]` 访问真实 mounted wrapper。
- metadata 与 native children 顺序绑定，不存在 Dioxus `ElementId` / placeholder / text node diff 语义。

当前 Dioxus renderer 是 mutation 模型：

- `text { "count = {count}" }` 会拆成父 `Text` element 加子 text node。
- dynamic text 可能通过 placeholder / replace / `set_node_text` 更新。
- `ElementId` 是 Dioxus RealDOM 节点身份，不等于 ArkUI native node 身份。
- `TemplateNode::Dynamic` 必须作为 placeholder 保留，不能直接吞掉。
- mutation stack 和 path walking 必须严格符合 Dioxus 语义。

因此不能把 `text child` 直接吃掉来模拟旧实现，否则后续 diff 会失真。

## 当前实现中的具体问题

### 1. tag canonical 不完整

Dioxus element 的 `TAG_NAME` 是 CamelCase，例如：

- `Column`
- `Button`
- `Text`
- `TextInput`

如果 `canonical_tag` / `kind_from_tag` 只识别 lowercase，就会 fallback 到 `Stack`。

这会导致 inspect 看到大量无意义 `Stack`，同时布局属性也会按错误 tag 分发。

### 2. Text/Button 被当成普通容器挂 text child

ArkUI native `Text` 和 `Button` 不应该嵌套 native `Text` 来显示文字。

错误 native projection：

```text
Text
  Text("count = 0")
```

正确 native projection：

```text
Text(TextContent = "count = 0")
```

但是 Dioxus 逻辑 text node 仍然必须保留在 host tree 中，不能被删除。

### 3. text child early-return 会破坏 Dioxus 语义

类似下面的逻辑是错误方向：

```rust
if apply_text_children_as_content(&parent, tag, &children) {
    return;
}
```

它会让 Dioxus child node 没有进入 renderer tree，后续 `set_node_text(id)`、placeholder replace、remove、diff 都会和 Dioxus 语义不一致。

### 4. ArkUINode wrapper 身份不稳定

`ArkUINode::clone()` 共享 native raw handle，但 Rust 侧字段会被 clone：

- `children`
- `event_handle`

如果 renderer 在 attach 之后仍然持有挂载前 wrapper，事件注册可能写到错误 wrapper 的 `event_handle` 上。inspect / layout 查询也可能使用了不完整的 wrapper tree。

旧实现通过 mounted parent 的 `children()` 获取真实 mounted wrapper；当前 Dioxus renderer 需要显式处理这个映射问题。

### 5. native child index 不能等同于 Dioxus child index

Dioxus child 可能投影成：

- 0 个 native node：例如 `Text` / `Button` 下的 text child，只贡献父属性。
- 1 个 native node：普通 element 或普通容器下的 text node。
- 多个 native node：未来 fragment 或复杂 projection。

因此 `insert_child(parent, child, logical_index)` 不能直接使用 Dioxus logical index 作为 ArkUI native index。

## 不做的事情

本修复不做以下事情：

- 不修改 registry 源码。
- 不修改 `@ohos-rs/ability` package 源码作为 renderer 修复手段。
- 不把 Dioxus text node 删除或折叠成旧实现语义。
- 不把 Dioxus mutation 重新解释成旧 `arkit_widget` Element tree。
- 不用 counter 示例做特化处理。

## 目标架构

在 `arkit_arkui` 中引入 renderer-owned host tree。

```rust
type HostId = usize;

struct HostNode {
    kind: HostKind,
    parent: Option<HostId>,
    children: Vec<HostId>,
    native: NativeProjection,
}

enum HostKind {
    Root,
    Element { tag: &'static str },
    Text { value: String },
    Placeholder,
}

enum NativeProjection {
    Native(NodeRef),
    None,
}
```

renderer 维护：

```rust
hosts: Vec<HostNode>,
element_to_host: Vec<Option<HostId>>,
stack: Vec<HostId>,
```

原则：

- Dioxus mutation 更新 host tree。
- host tree 是 renderer 的真实结构来源。
- ArkUI native tree 是 host tree 的 projection。
- `ElementId` 映射到 host node，不直接表达 native child index。
- text node 始终存在于 host tree。

## Projection 规则

### 普通 element

普通 element 投影成对应 ArkUI native node：

```text
Host Element(column) -> ArkUI Column
Host Element(row)    -> ArkUI Row
Host Element(stack)  -> ArkUI Stack
```

其 native children 由 projected children 计算得出。

### Text element

`Text` element 本身投影成 ArkUI `Text`。

它的 text children 不投影成 nested native `Text`，而是合并为父 `TextContent`。

```text
Host Element(text)
  Host Text("count = ")
  Host Text("0")

-> ArkUI Text(TextContent = "count = 0")
```

### Button element

`button` 是 Dioxus 语义元素，对外仍只有一个 logical/native root。ArkUI 原生 Button 只支持 label，不能承载任意 Dioxus children，因此最终 projection 使用可点击、可聚焦并带 Button accessibility role 的 outer container，加一个内部 Row 承载普通 child projection。

```text
Host Element(button)
  Host Text("increment")

-> ArkUI pressable container (Button semantics/default skin)
     ArkUI Row
       ArkUI Text(TextContent = "increment")
```

该 composite projection 必须继续支持图标、文本和任意子组件；禁止退回只接受字符串 label 的第二套 Button API。

### 普通容器下的 text node

如果 text node 出现在普通容器下，它需要投影成 native `Text`：

```text
Host Element(column)
  Host Text("hello")

-> ArkUI Column
     ArkUI Text(TextContent = "hello")
```

### Placeholder

Placeholder 必须保留在 host tree 中，作为 Dioxus replace path 的锚点。

它默认可以投影成 0 个 native node，除非后续需要一个 native anchor 来支持 ArkUI API 限制。是否需要 native anchor 应由插入/删除算法决定，不能影响 Dioxus logical tree。

## Mutation 处理计划

### load_template

当前直接构建 native subtree 的方式要改为构建 host subtree：

- `TemplateNode::Element` -> `HostKind::Element`
- `TemplateNode::Text` -> `HostKind::Text`
- `TemplateNode::Dynamic` -> `HostKind::Placeholder`

静态 template 内部节点即使没有 `ElementId`，也必须进入 host tree。否则 path walking 仍会依赖 native children，和 projection 产生冲突。

### create_text_node

创建 `HostKind::Text { value }`，压入 mutation stack。

是否创建 native `Text` 由 parent projection 决定。

### create_placeholder

创建 `HostKind::Placeholder`，压入 mutation stack。

### append_children

只更新 host tree：

- 从 stack pop 出 child host ids。
- 设置 parent。
- 追加到 parent logical children。
- 调用 `sync_projection(parent)`。

不能因为 parent 是 `Text` / `Button` 就 early-return 丢掉 child。

### replace_placeholder_with_nodes

用 host tree 的 logical path 找到 placeholder：

- 替换 host logical children。
- 保留新节点 parent。
- 清理被替换 placeholder 的 projection。
- 调用最近需要更新的 projection parent。

### insert_nodes_before / insert_nodes_after

先基于 host tree 找 logical sibling 和 logical index。

再同步 native projection。native index 由 projection 计算，不由 Dioxus logical index 直接决定。

### replace_node_with / remove_node

删除或替换 host subtree，同时清理 native projection。

清理顺序：

1. 从 parent logical children 移除。
2. 从 native projection 移除对应 native roots。
3. dispose 被移除 subtree 的 native nodes。
4. 清空 `element_to_host` 中相关映射。

### set_node_text

更新 host text node 的 `value`。

然后根据 parent 类型决定同步方式：

- parent 是 `Text`：重新计算父 `TextContent`。
- parent 是 semantic `button`：更新它内部 content Row 对应的 native Text projection。
- parent 是普通容器：更新该 text node 自己投影出的 native `TextContent`。

## Native wrapper 处理计划

renderer 必须保证事件注册和 layout 查询使用当前 mounted wrapper。

attach 后需要更新 host node 的 `NativeProjection::Native(NodeRef)`，指向 parent.children 中的实际 wrapper。

建议提供统一函数：

```rust
fn attach_native_child(parent: HostId, child: HostId, native_index: usize);
fn remove_native_child(parent: HostId, child: HostId);
fn native_roots_for(host: HostId) -> Vec<NodeRef>;
fn projected_native_len(host: HostId) -> usize;
```

不要在 mutation handler 中直接操作 `ArkUINode::add_child` / `insert_child`，所有 native attach 都走 projection 层。

## Event 处理计划

`create_event_listener(name, id)`：

1. 通过 `ElementId` 找到 `HostId`。
2. 找到该 host 的 native projection target。
3. 在 mounted wrapper 上注册 native event。
4. callback dispatch 时仍然使用原始 `ElementId`。

逻辑 text node 没有 native target，不注册事件。

`Button` click 应绑定到 `Button` host 的 native projection，而不是它的 text child。

原生 callback 不得同步调用 Dioxus runtime。最终事件流必须是：

1. ArkUI node event/gesture callback 复制 owned payload 并写入 `RuntimeEventSink` queue。
2. callback 唤醒 OpenHarmony UI loop 后立即返回。
3. UI loop 在没有借用 `RuntimeInner` 时调用 `Runtime::handle_event`。
4. Dioxus scheduler 产生 ready work 后，runtime 才执行 `render_immediate`。

这是 renderer patch 期间可能同步触发 ArkUI callback 的必要隔离。用 `try_borrow` 丢事件、在 callback 中直接 clone Dioxus Runtime，或 catch panic 后继续执行都不符合最终架构。

`onlongpress` / `on_long_press` 走 ArkUI Gesture API，而不是映射到 `OnClick`。recognizer、callback context 与 native wrapper 由 HostNode 持有；listener remove、wrapper rebind 和 subtree dispose 必须先 remove/dispose recognizer，再释放 callback context。

## node_for_element 处理计划

`node_for_element(id)` 应返回该 `ElementId` 对应 host 的 native projection target。

对于没有 native projection 的逻辑节点：

- text child under `Text` / `Button` 可以返回 parent native node，或返回 `None`。
- 为避免误导 layout hooks，建议第一阶段返回 `None`。
- 需要文档说明：layout hook 应挂在有 native projection 的 element 上。

## 分阶段执行

### Phase 1：修正 tag 映射

目标：

- `Column` / `Button` / `Text` / `TextInput` 等 CamelCase tag 不再 fallback 到 `Stack`。

验证：

- inspect 不再显示全部是 Stack。
- counter 根节点是 Column；semantic button 的 composite Stack/Row 是明确 projection，不是 unknown tag fallback。

### Phase 2：引入 HostTree，不改变 projection 行为

目标：

- 加入 `HostNode` / `HostId` / `element_to_host`。
- mutation stack 从 `NodeRef` 改成 `HostId`。
- 先保持一 host 一 native 的简单 projection。

验证：

- `ohrs build --arch aarch`
- counter 能维持现有显示能力。

### Phase 3：实现 Text/Button text projection

目标：

- `Text` child text 合并到父 `TextContent`；`button` child 按普通内容投影到内部 Row。
- Dioxus text node 仍保留在 host tree。
- `set_node_text` 能更新父 `TextContent` 或 button content Text。

验证：

- `text { "count = {count}" }` 显示 `count = 0`。
- 点击后更新为 `count = 1`。
- `button { "increment" }` 显示带 Button semantics/default skin 的文本，并继续支持任意 children。

### Phase 4：修正 native index 与 wrapper remap

目标：

- native insert index 通过 projection 计算。
- attach 后 host native projection 指向 mounted wrapper。
- node event 与 gesture 注册使用 mounted wrapper。

验证：

- 点击事件稳定触发。
- 多次点击连续更新。
- insert/remove/replace 不破坏 native children 顺序。

### Phase 5：补测试与调试能力

目标：

- 为 HostTree mutation 增加单元测试或纯 Rust 测试。
- 增加可开关的树 dump，输出 logical tree 和 native projection tree。

建议覆盖：

- container text child。
- `Text` child text。
- `Button` child text。
- placeholder replace。
- insert before/after mixed projected children。
- remove logical-only text node。
- dynamic text update。

## 验证命令

```bash
cargo fmt -p arkit_arkui
cd examples/counter
ohrs build --arch aarch

cd ../..
app/run.sh counter all
```

`cargo check` 只能作为 host 诊断，不能替代 `ohrs build --arch aarch`。设备验收一次只打包安装一个 example。

期望结果：

- 页面居中显示 `count = 0`。
- 按钮显示 `increment`。
- 点击按钮后文本更新为 `count = 1`。
- inspect 中 root app node、Column、Text 和 semantic button composite projection 类型合理。
- `Text` element 的 text child 不生成 nested native Text；button 的普通 children 只投影到它的内部 content Row。

## 风险与约束

- HostTree 会重构 `arkit_arkui` 的核心数据结构，改动面比局部修补大，但这是保持 Dioxus 语义的必要成本。
- Projection 层必须清晰区分 logical index 和 native index，否则 insert/remove 会继续错。
- `node_for_element` 对 logical-only node 的行为需要谨慎定义，避免 layout hook 得到误导性 native node。
- ArkTS `DefaultXComponent` / `ContentSlot` 尺寸问题应单独验证，不应该混入 Dioxus renderer 语义修复。

## 最终标准

修复完成后，renderer 应满足：

- Dioxus mutation 是唯一语义来源。
- `ElementId` 身份稳定。
- text node 不被删除、不被吞掉。
- ArkUI native tree 是 host tree 的 projection。
- native `Text` 使用 `TextContent`；semantic `button` 使用单 root composite projection 承载任意 Dioxus children。
- 点击事件注册到正确 mounted native wrapper。
- native callback 只入队，Dioxus event dispatch 与 render 由 UI loop 分阶段串行执行。
- 长按等 gesture 具有 renderer-owned 生命周期，不降级为 click，不遗留裸 callback context。
- layout / inspect 看到的 native tree 与 projection 一致。
