---
title: 播放控制
description: "play、seek、reverse、cancel 与快照。"
---

# 播放控制

`AnimationControls` 操作一个已注册 Timeline。命令语义与 sampled/native backend 无关；无法保持契约的 native 计划会回退或报错。

## 控制命令

| 操作                        | 语义                                     |
| --------------------------- | ---------------------------------------- |
| `play` / `pause` / `resume` | 播放、暂停、从当前位置恢复               |
| `restart`                   | reset 后开始新生命周期                   |
| `reverse`                   | 反转方向，不隐式开始或恢复               |
| `seek`                      | 立即采样，不触发 crossing event          |
| `seek_with_events`          | 采样并处理 call/loop/terminal crossing   |
| `complete`                  | finite timeline 到 terminal              |
| `cancel`                    | 停止并保留最后提交视觉值                 |
| `reset`                     | 回到计划逻辑起点                         |
| `revert`                    | 恢复首次播放前捕获的 baseline            |
| `stretch`                   | 保持 normalized progress 重映射 duration |
| `refresh`                   | 重做 target/unit/layout/baseline resolve |
| `set_timeline`              | 运行时替换计划                           |

`cancel`、`reset` 和 `revert` 不是同义词。关闭临时交互通常 cancel；需要回到设计态用 revert；要重新从计划起点播放用 restart。

## 读取状态

`snapshot` 返回播放状态、时间、进度和 outcome；`direction` 返回当前方向；`lowering_report` 说明 backend 选择。

```rust
let snapshot = controls.snapshot();
if snapshot.state == PlaybackState::Paused {
    controls.resume();
}
```

普通帧不会触发 Dioxus 重渲染。只有调用 `use_animation_snapshot` 或 `subscribe` 的 scope 才接收响应式进度，避免 60fps 全树 render。

## 等待结束

```rust
let outcome = controls.finished().await;
match outcome {
    AnimationOutcome::Completed => {}
    AnimationOutcome::Cancelled => {}
    _ => {}
}
```

`finished()` 等待真实 terminal outcome。Presence 和业务流程应依赖它，不用 `sleep(duration)` 猜结束时间。

## 回调顺序

begin → before_update → sample/compose → update → adapter commit → render → loop → complete/cancel → settle。

callback 只做轻量状态派发；重 I/O 放到异步任务。频繁读取 snapshot 做日志也会影响帧时间，应在开发诊断后关闭。
