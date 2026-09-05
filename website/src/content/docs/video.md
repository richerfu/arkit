---
title: Video
description: "用 AVPlayer 在原生 Surface 播放视频，并完整处理控制、媒体事件与生命周期。"
---

# Video

`video` feature 提供基于 OpenHarmony AVPlayer 的原生播放器。结构采用成熟移动端播放器常见的 controller/view 分离：`VideoPlayer` 负责 source、Surface 和声明式配置，`VideoController` 负责播放命令与只读 snapshot。AVPlayer 的创建、控制、查询和销毁全部串行在独立 worker，native callback 只转换成 owned event，不阻塞 ArkUI 或 Dioxus UI loop。

## 基本使用

```toml
[dependencies]
arkit = { version = "*", features = ["video"] }
```

```rust
use std::time::Duration;
use arkit::prelude::*;

#[component]
fn Movie() -> Element {
    let controller = use_hook(VideoController::new);

    rsx! {
        VideoPlayer {
            source: VideoSource::url("https://cdn.example.com/movie.mp4"),
            controller: Some(controller.clone()),
            autoplay: true,
            resize_mode: VideoResizeMode::Contain,
            controls: Some(VideoControls::default()),
            progress_interval: Duration::from_millis(250),
            width: "100%",
            height: "240",
            on_progress: move |progress: VideoProgress| {
                println!("{:.1}s", progress.position.as_secs_f64());
            },
            on_error: move |error: VideoError| println!("video: {error}"),
        }
    }
}
```

网络播放需要在应用 `module.json5` 声明 `ohos.permission.INTERNET`。仓库通用 app shell 已包含该权限。

## Source

普通 URL 直接使用 `VideoSource::url`。需要鉴权或 CDN 参数时使用带稳定 key 的类型化 source；header value 不进入 `Debug`：

```rust
let request = VideoNetworkSource::new("https://cdn.example.com/private/master.m3u8")
    .with_key("episode-42-token-v2")
    .with_header("authorization", "Bearer token");
let source = VideoSource::network(request);
```

本地资源使用 retained file descriptor，支持 offset/size 子区间。`Arc<OwnedFd>` 在 AVPlayer 整个 source 生命周期内保持描述符有效：

```rust
let source = VideoSource::file(VideoFileSource::new(
    "bundled-intro-v1",
    owned_fd,
    0,
    file_len,
));
```

source 相等性只比较 key，避免每次 props 比较触碰资源本体。同一 URL 背后的内容或 header 改变时需要换 key。AVPlayer 自身负责容器、编解码与 HLS/DASH 等平台支持格式；组件不在 Rust 层重复下载或解码媒体。

## 解码与渲染路径

组件把 source 直接交给系统 `OH_AVPlayer`，并把 XComponent 的 `OHNativeWindow` 直接设为视频输出 Surface。Rust 侧不解码帧、不把 YUV/RGB 像素拷回 CPU，也没有逐帧 CPU 绘制。系统会依据设备、编解码格式、profile、分辨率和运行环境选择硬件或软件 codec；支持的真机通常走专用媒体解码器，Surface 的合成与缩放走图形栈。这里不能把“硬件视频解码”简单等同于“GPU 编解码”，也不能承诺不支持的格式或模拟器绝不回退到软件解码。

## 内置控制层与自定义 UI

`controls: Some(VideoControls::default())` 开启标准叠加控制层。控制层与加载状态都是 Surface `Stack` 的上层节点，不占用视频之外的布局空间；`Contain` 模式还会按媒体宽高比对齐实际可见画面，不会把控制器留在 letterbox 黑边底部。画面底部只叠加半透明 zinc 蒙层。默认是 shadcn zinc dark 视觉：32vp 紧凑 `Button`、无按钮底色、3vp `Slider` 轨道与 Lucide 图标；倍速与时间保留必要文字。

默认提供进度拖动、当前/总时长、播放/暂停、前后跳转、静音、倍速和全屏。拖动中进度由本地手势状态驱动，松手后在 AVPlayer 回报 seek completed 前持续显示目标位置和原生 `Spinner`，不会短暂跳回旧进度。播放时控制层自动隐藏，点按视频重新显示。全屏通过 root portal 填满应用窗口，退出按钮和系统返回键都能退出；Surface 重建时保持播放位置和播放意图。`on_fullscreen_change` 可同步业务状态。

每项功能和视觉 token 均可配置：

```rust
let controls = use_hook(|| {
    let mut controls = VideoControls::default();
    controls.show_rewind = false;
    controls.show_forward = false;
    controls.show_stop = true;
    controls.show_loop = true;
    controls.seek_step = Duration::from_secs(15);
    controls.playback_rates = vec![0.75, 1.0, 1.25, 1.5, 2.0];
    controls.auto_hide = Some(Duration::from_secs(4)); // None = 始终显示
    controls.style.accent_color = 0xFF22D3EE;
    controls.style.overlay_color = 0x7009090B;
    controls.style.icon_size = 20.0;
    controls.icons.fullscreen = "maximize".into(); // 任意内置 Lucide 名称
    controls.labels.fullscreen = "展开".into(); // prefer_icons = false 时使用
    controls
});

rsx! {
    VideoPlayer {
        source: VideoSource::url("https://cdn.example.com/movie.mp4"),
        controls: Some(controls.clone()),
    }
}
```

`prefer_icons` 默认为 `true`；设为 `false` 可整体切回 `VideoControlLabels` 文字按钮。`VideoControlIcons` 可分别替换 play/pause/rewind/forward/stop/mute/loop/fullscreen 的 Lucide 名称。`VideoControlsStyle` 可调整蒙层与按钮颜色、图标/加载指示器/进度拇指尺寸、轨道粗细、控件尺寸、圆角和间距；默认 `button_color` 为透明，需要有底色的业务再显式设置。

完全自定义结构时保持 `controls: None`，传入 `VideoController` 后用任意 ArkUI markup 调用 `play`、`seek`、`set_playback_rate`、`enter_fullscreen`、`exit_fullscreen` 等方法；进度、播放状态和全屏状态从 `snapshot()` 或对应事件读取。内置控制层和自定义控制层使用同一套 controller 能力，不存在两套播放实现。

## 控制器与播放配置

`VideoController` 提供：

- `play`、`pause`、`toggle`、`stop`；
- `seek`、`seek_with_mode`、`seek_by`；
- `set_volume`、`set_muted`、`set_looping`、`set_playback_rate`；
- `select_bitrate`、`select_track`、`deselect_track`；
- `replace_source`；
- `enter_fullscreen`、`exit_fullscreen`、`toggle_fullscreen`；
- `status` 和完整 `snapshot` 查询。

声明式 props 覆盖 autoplay、looping、muted、volume、`0.125..=4.0` 任意倍速、initial position、50 ms 到 10 s 的 progress cadence，以及 contain/cover/stretch/none 四种 Surface scaling。命令和 prop 都进入同一 worker 队列，避免同时操作 AVPlayer。

## 轨道、字幕与事件

外置字幕通过 `Vec<VideoSubtitleSource>` 随 source 一起配置；平台解析出的 cue 由 `on_subtitle` 返回。媒体轨和 adaptive bitrate 列表位于 `VideoMetadata`/`VideoSnapshot`，可以按 index 选择或取消音频、视频和字幕轨。

完整事件面包括 `on_load_start`、`on_load`、`on_status_change`、`on_progress`、`on_buffer`、`on_seek`、`on_playback_rate_change`、`on_volume_change`、`on_bitrate_change`、`on_available_bitrates`、`on_ready_for_display`、`on_tracks_change`、`on_subtitle`、`on_audio_interrupted`、`on_fullscreen_change`、`on_end` 和 `on_error`。高频 position update 按 `progress_interval` 合并；controller snapshot 会同步 position/duration/buffered、video size、live、volume/mute/rate/looping/fullscreen、tracks 和 available bitrates。

## 生命周期与所有权

应用进入后台时默认暂停，回到前台后按原 playback intent 恢复；确需后台音频可设 `play_in_background: true`。`active` 是额外业务 gate。Surface 销毁时 worker 先同步释放 AVPlayer，再释放它保留的 NativeWindow 引用；新 Surface 到达后以同一 source/configuration 重建。组件卸载会注销 XComponent callback、关闭 worker channel、join 线程，最后释放 player 和 descriptor/source owners。

## 构建验证

```sh
cd examples/video
ohrs build --arch aarch
cd ../../
./app/run.sh video all
```

示例覆盖可配置样式的内置控制层、进度拖动、播放/暂停/停止、静音、0.5x 到 2x 倍速、循环、全屏/退出全屏与返回键退出；也覆盖带 HTTP header 的 MP4 与 adaptive HLS source 切换、外置字幕和轨道选择、码率选择、首帧以及 contain/cover/stretch 切换，并在界面显示 status、duration、position、buffer 与错误信息。
