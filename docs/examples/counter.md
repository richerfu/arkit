# counter

路径：`examples/counter`

counter 是最小应用示例，覆盖：

- `State` 保存计数。
- `Message::Increment` 表示按钮点击。
- `update` 修改状态并返回 `Task::none()`。
- `view` 用 `column_component`、`text`、`button` 生成 UI。
- `#[entry]` 暴露 OpenHarmony 入口。

## 适合参考的场景

- 新建应用骨架。
- 验证 `#[entry]` 和 `application`。
- 验证基础 button 事件和文字渲染。

## 构建

```sh
cd examples/counter
ohrs build --arch aarch
```
