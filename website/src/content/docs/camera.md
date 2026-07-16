---
title: 相机
description: "CameraKit 预览、前后摄切换、拍照、扫码与资源生命周期。"
---

# 相机

相机能力分为两层：`ohos-camera-binding` 负责 CameraKit RAII、输出 profile、控制项、图片/分析帧回调，并通过独立的 `xcomponent` feature 提供 Surface 直连；`arkit_camera` 只负责 Dioxus/ArkUI 生命周期、worker 和组件集成。

拍照模式通过 facade 的 `camera` feature 单独启用：

```toml
[dependencies]
arkit = { version = "*", features = ["camera"] }
```

需要扫码时改用 `camera-scan`：

```toml
[dependencies]
arkit = { version = "*", features = ["camera-scan"] }
```

`camera-scan` 包含 `camera`，并额外引入纯 Rust 条码解码依赖。只启用 `camera` 时，扫码解码器不会进入依赖图。

未启用该 feature 时，`arkit_camera`、CameraKit 及相机领域新增的 native surface/image 依赖边不会进入业务依赖图。基础 renderer 已经共享的 ArkUI 绑定不重复计入相机 feature。

## 相机与扫码两种模式

`CameraView` 提供两种互斥模式：

- `CameraMode::Photo`：闪光灯、预览/照片分辨率、变焦、镜头切换和快门工具栏；预览默认支持点击对焦、点击测光和双击切换镜头。
- `CameraMode::Scan`：手电筒、变焦、镜头切换、扫描框和提示工具栏；预览默认点击对焦，后台自动识别，无快门交互。

两种模式都采用接近 CameraKit 相机应用的沉浸式工具界面：顶部放置圆形快捷控制，照片模式在底部提供变焦、模式名、双层快门和镜头切换；预览/照片分辨率点击后通过紧凑下拉面板选择，不会自动轮询。扫码模式把四角扫描框、循环扫描线、提示、变焦和自动识别状态分层展示。工具栏元素、颜色、尺寸、文案、安全区和预览交互都通过 `CameraPhotoToolbarConfiguration` / `CameraScanToolbarConfiguration` 与 `CameraPhotoPreviewInteractions` / `CameraScanPreviewInteractions` 配置。业务要完全自绘工具栏时，直接使用下面的低层 `CameraPreview` 和 `CameraController`。

```rust
let mode = CameraMode::Photo(CameraPhotoModeConfiguration {
    toolbar: CameraPhotoToolbarConfiguration {
        show_preview_resolution: true,
        show_photo_resolution: true,
        show_flash: true,
        mode_label: "文档".into(),
        control_background_color: 0x73000000,
        shutter_size: 78.0,
        top_inset: 48.0,
        bottom_inset: 24.0,
        ..Default::default()
    },
    interactions: CameraPhotoPreviewInteractions {
        tap_to_focus: true,
        double_tap_to_switch_camera: true,
        ..Default::default()
    },
    ..Default::default()
});

rsx! {
    CameraView {
        mode,
        percent_width: 1.0,
        percent_height: Some(1.0),
    }
}
```

照片工具栏可配置 `show_*` 显隐项、`mode_label`、前景/面板/控件/强调色、`control_size`、`shutter_size`、变焦轨道/手柄颜色与宽度、上下工具区高度以及安全区。扫码工具栏在相同基础上提供 `hint`、`footer`、扫描框尺寸、描边宽度和圆角；扫描线可通过 `show_reticle_scan_line`、`reticle_scan_line_color`、`reticle_scan_line_height`、`reticle_scan_line_inset` 和 `reticle_scan_duration` 控制。扫描线由原生优先的动画时间线驱动，在上下端淡入淡出，模式退出或相机会话暂停时停止。所有尺寸均使用 vp；嵌入已有页面头部时可将 `top_inset` 设为 `0.0`，全屏 edge-to-edge 页面则保留系统状态栏对应的 inset。

## 低层预览

```rust
use arkit::prelude::*;

#[entry(edge_to_edge)]
fn app() -> Element {
    let controller = use_hook(CameraController::new);
    let mut position = use_signal(|| CameraPosition::Back);
    let mut status = use_signal(CameraStatus::default);

    rsx! {
        stack {
            percent_width: 1.0,
            percent_height: 1.0,

            CameraPreview {
                controller: Some(controller.clone()),
                position: position(),
                percent_width: 1.0,
                percent_height: Some(1.0),
                on_status_change: move |next| status.set(next),
                on_photo: move |photo: CapturedPhoto| {
                    println!("{} bytes, {}×{}", photo.bytes().len(), photo.size.width, photo.size.height);
                },
                on_error: move |error| println!("camera: {error}"),
            }

            button {
                enabled: status().is_running(),
                onclick: move |_| {
                    let _ = controller.capture();
                },
                "拍照"
            }
            button {
                onclick: move |_| position.set(position().opposite()),
                "切换相机"
            }
        }
    }
}
```

完整的全屏预览、暂停、切换和拍照状态界面位于 `examples/camera`。

## 权限

模块的 `module.json5` 必须声明相机权限：

```json5
{
  name: "ohos.permission.CAMERA",
  reason: "$string:camera_permission_reason",
  usedScene: {
    abilities: ["EntryAbility"],
    when: "inuse",
  },
}
```

在创建页面前通过 `abilityAccessCtrl.createAtManager().requestPermissionsFromUser(...)` 请求 `ohos.permission.CAMERA`。组件会再次检查授权状态；未授权时进入 `CameraStatus::PermissionDenied`，不会尝试打开设备。

## 状态、分辨率与控制器

`CameraStatus` 给出可用于 UI 的完整生命周期：`WaitingForSurface`、`Starting`、`Running`、`Capturing`、`Stopped`、`PermissionDenied`、`Unavailable` 和 `Error`。只有 `Running` 且 `CameraSessionInfo::supports_photo()` 为 `true` 时，`CameraController::capture()` 才会接受拍照命令。没有 JPEG profile 的设备仍可正常预览，`photo_size` 为 `None`。

`active = false` 会停止并释放 native session，但保留组件 Surface；恢复为 `true` 时重建 session。修改 `position` 同样会按反向顺序关闭当前 session，再使用目标相机支持的 profile 重建。

`on_capabilities_change` 返回当前镜头实际支持的预览/照片分辨率；把精确尺寸写入 `CameraProfileSelection` 即可重建对应 profile。不存在的尺寸会返回明确的 `Unsupported` 错误，不再静默回落。`CameraController::controls()` 可读取闪光灯、手电筒、变焦、曝光、对焦、防抖、白平衡、帧率、色彩空间、微距等实时能力和取值范围，并通过对应的 `set_*` 方法调整。

## 扫码性能

扫码模式使用 CameraKit 的第二路 YUV preview output 接入 ImageReceiver，只复制去除 stride 后的 Y 灰度平面。采集帧率可由 `max_frames_per_second` 配置；解码在线程独立、容量为 1 的队列中运行，解码繁忙时丢弃旧帧，不阻塞 CameraKit 或相机 worker。`formats`、归一化识别区域、连续/单次模式和重复结果冷却时间均可配置。

## 拍照数据与所有权

`CapturedPhoto` 持有一份 `Arc<[u8]>` JPEG 数据，并附带像素尺寸和 CameraKit 时间戳。native 回调只在 buffer 映射有效期内读取 ImageNative 内存，随后立即解除映射并释放图片；业务可以通过 `bytes()` 借用或 `shared_bytes()` 共享已复制的数据。

相机工作和 native session 固定在专用线程，UI 线程只接收状态、照片与错误事件。组件卸载时依次停止 capture session、注销回调路由并释放 output、input、capability、device list 和 manager，避免 Surface 消失后继续访问 UI 节点。

## 设备验证

```sh
cd examples/camera
ohrs build --arch aarch
cd ../../
./app/run.sh camera all
```

带 CameraKit 相机的真机可验证实际预览和 JPEG 拍照。未配置虚拟相机的模拟器通常只能验证加载、授权、Surface 和 `Unavailable` 路径；这不等价于真机相机验收。
