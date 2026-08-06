//! WebView example — pluginized WebView positioned from dioxus layout.
//!
//! The webview runs through the `ohos.webview` bridge plugin
//! (`openharmony-ability-plugin-webview`). The framework registers the plugin
//! facade and injects its initialization during the ability-init stage, so
//! this example only enables the `webview` feature and drives the client
//! through [`RuntimeHandle::webview`] — no plugin plumbing on the business
//! side.
//!
//! The WebView surface mounts into the ArkTS session tree (not the dioxus
//! tree); the `WebviewStyle` `x`/`y`/`width`/`height` (vp) is derived from
//! the dioxus layout frame of the placeholder element, so the page occupies
//! exactly the same screen area as the old embedded WebView — control panel
//! on top, page below. Bridge calls are async promises issued from a dioxus
//! coroutine (`use_coroutine`) that runs on the local executor (the UI
//! thread), so awaiting a call re-enters the UI loop and signals can be
//! written directly.

use arkit::entry;
use arkit::prelude::*;
use futures_channel::mpsc::UnboundedReceiver;
use futures_util::StreamExt;
use napi_ohos::Either;

const RUST_URL: &str = "https://www.rust-lang.org";
const DOCS_URL: &str = "https://docs.rs";
const WEBVIEW_ID: &str = "arkit-example-webview";

/// Commands sent from the UI to the WebView coroutine.
enum WebviewCommand {
    Create(WebviewStyle),
    LoadUrl(String),
    Reload,
    EvalTitle,
    Hide,
    Show,
}

/// Resolves the `ohos.webview` client and the facade for `WEBVIEW_ID`.
fn resolve_handle(runtime: &RuntimeHandle) -> Result<WebviewHandle, String> {
    runtime
        .webview()
        .map(|client| client.handle(WEBVIEW_ID))
        .map_err(|error| error.to_string())
}

/// Creates the WebView with the given layout-derived style.
async fn create_webview(runtime: &RuntimeHandle, style: WebviewStyle) -> Result<(), String> {
    let client = runtime.webview().map_err(|error| error.to_string())?;
    client
        .create(
            WebviewCreateRequest::new(WEBVIEW_ID)
                .style(style)
                .url(RUST_URL.to_string())
                .transparent(true),
        )
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// Creates with a short retry: the bridge plugin only activates after the
/// ArkTS host delivers `ui-context-ready`, so a create racing startup (the
/// automatic first mount) can fail with "UIContext is not ready".
async fn create_webview_with_retry(
    runtime: &RuntimeHandle,
    style: WebviewStyle,
) -> Result<(), String> {
    let mut attempt = 0;
    loop {
        match create_webview(runtime, style.clone()).await {
            Ok(()) => return Ok(()),
            Err(error) if attempt < 10 => {
                attempt += 1;
                // Sleep on the framework tokio runtime; the JoinHandle itself
                // is polled here on the dioxus local executor.
                runtime
                    .tokio()
                    .spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    })
                    .await
                    .map_err(|error| error.to_string())?;
            }
            Err(error) => return Err(error),
        }
    }
}

/// Maps a dioxus layout frame (physical px) onto the plugin style (vp).
fn style_from_frame(frame: LayoutFrame, scale: f32) -> WebviewStyle {
    WebviewStyle {
        x: Some(Either::A(f64::from(frame.x / scale))),
        y: Some(Either::A(f64::from(frame.y / scale))),
        width: Some(Either::A(f64::from(frame.width / scale))),
        height: Some(Either::A(f64::from(frame.height / scale))),
        visible: None,
        background_color: Some("#FFFFFFFF".to_string()),
    }
}

#[entry]
fn app() -> Element {
    let runtime = use_runtime_handle();

    let url = use_signal(|| RUST_URL.to_string());
    let mut title = use_signal(|| String::from("loading..."));
    let mut status = use_signal(|| String::from("ready"));
    let mut created = use_signal(|| false);
    let mut visible = use_signal(|| false);

    // One worker coroutine owns the bridge conversation. It runs on the
    // dioxus local executor (the UI thread), so awaiting bridge promises
    // re-enters the UI loop and signals can be captured and written directly.
    let coroutine_runtime = runtime.clone();
    let commands = use_coroutine(move |mut rx: UnboundedReceiver<WebviewCommand>| {
        // The init closure may run more than once; clone per call so each
        // async block owns its own handle.
        let webview_runtime = coroutine_runtime.clone();
        async move {
            while let Some(command) = rx.next().await {
                let (outcome, message) = match command {
                    WebviewCommand::Create(style) => {
                        let outcome = create_webview_with_retry(&webview_runtime, style).await;
                        if outcome.is_ok() {
                            created.set(true);
                            visible.set(true);
                        }
                        (outcome, "webview created")
                    }
                    WebviewCommand::LoadUrl(target) => (
                        (async {
                            resolve_handle(&webview_runtime)?
                                .load_url(target)
                                .await
                                .map_err(|error| error.to_string())
                        })
                        .await,
                        "load",
                    ),
                    WebviewCommand::Reload => (
                        (async {
                            resolve_handle(&webview_runtime)?
                                .reload()
                                .await
                                .map_err(|error| error.to_string())
                        })
                        .await,
                        "reload",
                    ),
                    WebviewCommand::EvalTitle => {
                        let result: Result<Option<String>, String> = (async {
                            let page_title = resolve_handle(&webview_runtime)?
                                .evaluate_script("document.title")
                                .await
                                .map_err(|error| error.to_string())?;
                            Ok::<Option<String>, String>(page_title)
                        })
                        .await;
                        match result {
                            Ok(Some(page_title)) => {
                                title.set(page_title);
                                (Ok(()), "title updated")
                            }
                            Ok(None) => (Err("script returned no value".to_string()), "eval"),
                            Err(error) => (Err(error), "eval"),
                        }
                    }
                    WebviewCommand::Hide => {
                        let outcome = (async {
                            resolve_handle(&webview_runtime)?
                                .set_visible(false)
                                .await
                                .map_err(|error| error.to_string())
                        })
                        .await;
                        if outcome.is_ok() {
                            visible.set(false);
                        }
                        (outcome, "webview hidden")
                    }
                    WebviewCommand::Show => {
                        let outcome = (async {
                            resolve_handle(&webview_runtime)?
                                .set_visible(true)
                                .await
                                .map_err(|error| error.to_string())
                        })
                        .await;
                        if outcome.is_ok() {
                            visible.set(true);
                        }
                        (outcome, "webview shown")
                    }
                };
                status.set(match outcome {
                    Ok(()) => message.to_string(),
                    Err(error) => format!("{message} failed: {error}"),
                });
            }
        }
    });

    let title_display = (title.read()).clone();
    let status_display = (status.read()).clone();
    let url_display = (url.read()).clone();

    // Per-handler clones (each `move` closure owns its own).
    let mut url_onchange = url;
    let mut url_rust = url.clone();
    let mut url_docs = url.clone();
    let commands_show = commands.clone();
    let commands_rust = commands.clone();
    let commands_docs = commands.clone();
    let commands_reload = commands.clone();
    let commands_eval = commands.clone();
    let commands_hide = commands.clone();

    rsx! {
        column {
            width: "100%",
            height: "100%",
            background_color: "#FFF6F7FB",

            column {
                padding: 16.0,
                background_color: "#FFFFFFFF",

                text {
                    font_size: 24.0,
                    line_height: 28.0,
                    font_weight: "700",
                    "arkit webview example"
                }
                text {
                    margin_top: 8.0,
                    font_size: 14.0,
                    line_height: 18.0,
                    font_color: "#FF334155",
                    "title: {title_display}"
                }
                text {
                    margin_top: 4.0,
                    font_size: 13.0,
                    line_height: 18.0,
                    font_color: "#FF64748B",
                    "url: {url_display}"
                }
                text {
                    margin_top: 4.0,
                    font_size: 13.0,
                    line_height: 18.0,
                    font_color: "#FF0F766E",
                    "status: {status_display}"
                }

                textinput {
                    margin_top: 12.0,
                    padding: 10.0,
                    font_size: 14.0,
                    font_color: "#FF0F172A",
                    background_color: "#FFF1F5F9",
                    border_radius: 8.0,
                    value: url_display.clone(),
                    placeholder: "enter url",
                    onchange: move |evt| {
                        url_onchange.set(evt.string_value.clone());
                    }
                }

                row {
                    margin_top: 12.0,
                    button {
                        onclick: move |_| {
                            commands_show.send(WebviewCommand::Show);
                        },
                        "Open webview"
                    }
                    button {
                        margin_left: 8.0,
                        onclick: move |_| {
                            url_rust.set(RUST_URL.to_string());
                            commands_rust.send(WebviewCommand::LoadUrl(RUST_URL.to_string()));
                        },
                        "rust-lang.org"
                    }
                    button {
                        margin_left: 8.0,
                        onclick: move |_| {
                            url_docs.set(DOCS_URL.to_string());
                            commands_docs.send(WebviewCommand::LoadUrl(DOCS_URL.to_string()));
                        },
                        "docs.rs"
                    }
                }

                row {
                    margin_top: 8.0,
                    button {
                        onclick: move |_| {
                            commands_reload.send(WebviewCommand::Reload);
                        },
                        "Reload"
                    }
                    button {
                        margin_left: 8.0,
                        onclick: move |_| {
                            commands_eval.send(WebviewCommand::EvalTitle);
                        },
                        "Get title"
                    }
                    button {
                        margin_left: 8.0,
                        onclick: move |_| {
                            commands_hide.send(WebviewCommand::Hide);
                        },
                        "Hide"
                    }
                }
            }

            // The page area: its measured dioxus frame becomes the WebView
            // style, so the plugin surface lands exactly on this rectangle.
            WebviewArea {
                runtime: runtime.clone(),
                created: created.clone(),
            }
        }
    }
}

#[component]
fn WebviewArea(runtime: RuntimeHandle, created: Signal<bool>) -> Element {
    let node_ref = use_native_element_ref();
    let scale = runtime.scale();
    let commands = dioxus_hooks::use_coroutine_handle::<WebviewCommand>();
    let created_sig = created;

    // Auto-mount on the first measured layout: convert the frame (physical
    // px) into the plugin style (vp) and hand it to the coroutine.
    use_layout_frame(node_ref.clone(), move |frame| {
        if !frame.is_measured() || *created_sig.read() {
            return;
        }
        commands.send(WebviewCommand::Create(style_from_frame(frame, scale)));
    });

    rsx! {
        stack {
            native_ref: node_ref,
            width: "100%",
            height: 400.0,
            background_color: "#FFFFFFFF",
        }
    }
}
