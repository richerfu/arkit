---
title: 实时更新与动画
---

# 实时更新与动画

常规更新传入新的受控 `ChartOption`；高频追加点只在 scatter/lines 使用 `appendData`。初始和更新 transition 由 root AnimationHost 驱动。

## 受控 Option

```rust
let option = use_memo(move || {
    ChartOption::new()
        .x_axis(Axis::category(labels()))
        .y_axis(Axis::value())
        .push_series(Series::line("值", values()))
});

rsx! { ECharts { option: option() } }
```

批量更新领域数据后一次生成 option，避免同一业务 tick 连续创建多份中间配置。

## appendData

```rust
controller.append_data(ChartAppendData::scatter(
    0,
    [DataPoint::values([12.0, 36.0])],
));
```

当前增量限制：

- 只支持 scatter 和 lines。
- series index 必须指向匹配类型。
- 其他 series 使用受控 option 更新。

不要手工复制 renderer 内部 data，再同时 append 和替换 option；选定一个所有权路径。

## 窗口策略

实时曲线常保留最近 N 点或时间窗口。丢弃旧点应在领域缓冲区完成；只让画布看不见旧点但永久累积 data 会让 layout、hit-test 和内存继续增长。

## 动画配置

`ChartOption::animation` 分别配置 initial/update duration 与 easing。图表不创建私有 clock，因此页面 Timeline、Chart update 和其他 Drawing 动画共享帧提交。

高频数据到达速度快于动画时长时，缩短/关闭 update animation，或按帧/时间批量合并。排队播放每个历史状态会持续落后于实时数据。

## Thread 边界

WebSocket/后台采集只产生 owned 数据；通过 Signal/resource 或 `queue_ui_loop` 回到 UI。Controller 绑定 native instance，不在后台线程直接调用。

## 验证

`examples/chart` 包含 realtime option 和 appendData。检查长时间运行的内存、输入频率突增、窗口 resize、暂停恢复和 instance 卸载后的生产者清理。
