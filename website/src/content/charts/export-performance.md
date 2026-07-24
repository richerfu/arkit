---
title: 导出与性能
description: "导出图片、性能上限，以及卡顿时先查哪里。"
---

# 导出与性能

卡顿通常来自数据量、布局、文字、命中测试、动画或导出尺寸。先分清瓶颈在哪一段，再针对性砍。

## 图片导出

toolbox `saveAsImage` 支持：

- PNG / JPEG
- pixelRatio
- backgroundColor
- excludeComponents
- 输出 path

renderer 在 CPU bitmap 上重绘当前 option/zoom/selection，再转 PixelMap 编码。导出反映当前 resolved 状态，不只是原始 option。

## 限制

单边最大 8192 像素，pixel buffer 最大 256 MiB。pixelRatio 会按平方增加内存；导出前根据 canvas size 计算目标像素，并为错误保留用户可见反馈。

文件路径使用 OpenHarmony 应用可写目录。不要让不可信输入直接决定绝对路径或文件名；JPEG 不能保留 alpha 时显式设置 backgroundColor。

## 性能检查

| 症状       | 优先检查                                           |
| ---------- | -------------------------------------------------- |
| 首次绘制慢 | JSON 解析、dataset normalize、文本量               |
| 拖动缩放卡 | dataZoom 后仍绘制的数据量、label、hit-test         |
| 更新卡     | option 重建频率、update animation、stable identity |
| 内存增长   | 未裁剪实时 data、地图缓存、超大导出 buffer         |
| 点击慢     | custom shape 数量、复杂 path 命中                  |

## 优化原则

1. 先减少不可见数据与 label。
2. 复用稳定 series/node identity。
3. 合并同一帧内的数据更新。
4. Custom renderer 热路径不做分配密集或 I/O 工作。
5. 只为必要 series 开启 effect/动画。

## Diagnostic

JSON extra/unsupported 字段在开发阶段作为诊断处理。Controller 查询 canvas size、resolved option 和 mount 状态可以帮助定位配置与生命周期问题。

## 完整验证

`examples/chart` 覆盖 22 series、realtime、appendData、actions、坐标转换和 event：

```sh
cd examples/chart
ohrs build --arch aarch
```

真机检查文字/颜色、touch hit、dataZoom、legend、tooltip、动画、导出路径与大数据帧率。桌面构建通过不能替代 ArkUI Drawing 真机验证。
