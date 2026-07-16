---
title: Custom 自定义系列
description: "Custom renderer 与热路径约束。"
---

# Custom 自定义系列

Custom series 允许调用方根据数据与坐标上下文生成自定义绘制结构。

```rust
let data = vec![DataPoint::values([10.0, 24.0])];
let series = Series::custom("自定义标记", data, move |context| {
    render_custom_item(context);
});
```

构造签名：

```rust
Series::custom(
    name,
    data: Vec<DataPoint>,
    renderer: impl for<'a> Fn(CustomRenderContext<'a>) + 'static,
)
```

renderer 位于热路径：不得执行网络或文件 I/O，不要逐 item 解析配置；缓存可复用几何与文本测量，并保持 hit-test identity 与返回 shape 一致。

Custom 支持的是 Arkit 原生 render context，不保证任意 ECharts JavaScript `renderItem` 无修改运行。不可信远程配置不得注册任意本地 callback。
