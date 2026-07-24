---
title: Lottie
description: "在原生 Surface 上播 Lottie，并处理好暂停与生命周期。"
---

# Lottie

Lottie 走 ThorVG 和独立 Surface，不塞进 WebView。播、停、seek 和可见性暂停都接到组件生命周期上，后台时不会空转。

## 渲染路径

`LottiePlayer` 不把动画拆成 ArkUI/Dioxus 节点，也不在 60 FPS 下触发 VirtualDom diff。组件创建一个 Surface 模式 XComponent，使用系统 frame callback 驱动专用 worker；worker 在同一线程持有 ThorVG engine、composition、timeline 和 canvas，并将最终像素直接写入当前 NativeWindow buffer。每帧完成 `draw + sync` 后才 flush buffer，不分配中间 RGBA bitmap，也不把整帧复制回 UI 线程。

ThorVG 软件引擎启用内部线程、Lottie、PNG 和字体加载能力。NativeWindow 可能轮换多个 buffer，因此 renderer 每帧完整清屏绘制，不使用依赖上一帧像素的 dirty-region 模式。这样保留了多线程矢量栅格化和零中间帧复制，同时避免 swapchain buffer 内容不连续导致的残影。

## 基本使用

```rust
const ANIMATION: &[u8] = include_bytes!("../assets/loading.json");

#[component]
fn Loading() -> Element {
    rsx! {
        LottiePlayer {
            source: LottieSource::embedded("loading-v1", ANIMATION),
            repeat: LottieRepeatMode::Loop,
            fit: LottieFit::Contain,
            quality: 50,
            max_frames_per_second: 60,
            width: Some(160.0),
            height: Some(160.0),
        }
    }
}
```

`LottieSource` 的 `key` 是 composition 身份。Dioxus props 比较只检查 key，不逐次扫描整份 JSON；数据发生变化时必须同步修改 key。内存入口接收 JSON bytes，内嵌 PNG/font data 可由 ThorVG loader 解析；dotLottie ZIP container 需要业务先解包为 JSON 和内嵌资源。

## 网络 URL

网络数据源需要开启对应 feature：

```toml
[dependencies]
arkit = { version = "*", features = ["lottie-network"] }
```

直接使用公开的 HTTP/HTTPS JSON：

```rust
LottiePlayer {
    source: LottieSource::url("https://cdn.example.com/animations/loading.json"),
    width: Some(160.0),
    height: Some(160.0),
    on_error: move |error| println!("lottie download: {error}"),
}
```

需要鉴权、版本身份或更严格资源限制时，使用类型化网络数据源：

```rust
use std::time::Duration;

let source = LottieNetworkSource::new("https://cdn.example.com/private/success.json")
    .with_key("success-v3")
    .with_header("authorization", "Bearer token")
    .with_timeout(Duration::from_secs(8))
    .with_max_download_bytes(4 * 1024 * 1024);

rsx! {
    LottiePlayer { source: LottieSource::network(source) }
}
```

URL 默认同时作为 composition key；同一个 URL 内容或请求头发生变化时，应通过 `with_key` 提升版本。默认超时 20 秒、解压后响应上限 16 MiB、重定向上限 5 次，支持 gzip、brotli 和 deflate。所有播放器共享一个线程安全 HTTP client 与连接池；下载运行在框架 Tokio 线程池而不是 UI 或 ThorVG worker，数据按块读取并在每次扩容前检查上限。切换 source 或销毁组件会取消未完成请求，晚到响应还会由 worker 根据 key 丢弃。HTTP 状态、超时、连接、header 和体积错误通过 `LottieErrorKind::Network` 与 `on_error` 返回，header value 不写入 Debug 或错误信息。

应用模块必须在 `module.json5` 声明 `{ "name": "ohos.permission.INTERNET" }`。仓库的通用 `app` shell 已包含该权限。

## 控制与状态

```rust
let controller = use_hook(LottieController::new);

rsx! {
    LottiePlayer {
        source: LottieSource::embedded("success-v2", ANIMATION),
        controller: Some(controller.clone()),
        playing: true,
        repeat: LottieRepeatMode::None,
        speed: 1.0,
        on_complete: move |_| println!("complete"),
        on_error: move |error| println!("lottie: {error}"),
    }
    button { onclick: move |_| { let _ = controller.seek(0.5); }, "50%" }
}
```

控制器提供 `play`、`pause`、`toggle`、`stop`、`seek`、`seek_frame`、`set_speed` 和 `set_repeat_mode`。`LottieStatus` 描述 loading、surface、playing、paused、completed 与 error 状态；composition metadata 和 coalesced frame progress 可分别从 controller 读取。`on_frame` 最多每 100 ms 回调一次，默认渲染路径不会为每个显示帧唤醒 Dioxus。

`LottieFit` 支持 contain、cover、fill 和原始尺寸，`LottieAlignment` 提供九宫格对齐。`quality` 的 `0..=100` 主要影响 blur/shadow 等效果，默认 50；低端设备可同时降低 quality 与 `max_frames_per_second`。

## 生命周期

最终活动条件为业务 `active`、应用处于前台和组件可见三者同时成立。进入后台、父组件隐藏、Tab 切走或组件完全离开可见区域时，worker 保留已解析 composition 和当前 frame，但停止申请 NativeWindow buffer；恢复后立即重绘当前帧并继续计时。Surface 销毁会释放 NativeWindow 引用，组件卸载则取消网络请求、注销 frame callback、停止并 join worker，再销毁 ThorVG engine。

## 构建验证

```sh
cd examples/lottie
ohrs build --arch aarch
cd ../../
./app/run.sh lottie all
```

示例包含内嵌/网络 URL 切换、多层 shape、渐变、缩放、正反向轨道和粒子，能同时观察下载状态、流畅度、fit、倍速、seek 与生命周期恢复。模拟器可验证软件渲染与 Surface 路径；最终帧时延、温升和包体应在目标真机的 release 构建上测量。
