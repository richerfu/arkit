# 安装与第一个页面

业务 crate 依赖 workspace facade：

```toml
[dependencies]
arkit.workspace = true
napi-ohos.workspace = true
napi-derive-ohos.workspace = true

[build-dependencies]
napi-build-ohos.workspace = true
```

入口函数返回 Dioxus `Element`：

```rust
use arkit::prelude::*;

#[entry]
fn app() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            align_items: "center",
            justify_content: "center",

            text { font_size: 28.0, "count = {count}" }
            button {
                margin_top: 12.0,
                onclick: move |_| count += 1,
                "increment"
            }
        }
    }
}
```

`#[entry]` 生成 OpenHarmony 的 `init/render/destroy` NAPI 入口。框架 root 自动安装 native-node/overlay context，业务 root 保持独立 Dioxus component scope。

先运行：

```sh
cd examples/counter
ohrs build --arch aarch
```

`cargo check` 可以辅助发现 host 类型错误，但不能替代 OpenHarmony 目标构建。`ohrs` 成功后，设备打包与部署参考 `app/run.sh`；一次只安装一个 example 并完成交互验收。
