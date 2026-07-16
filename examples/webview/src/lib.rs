//! WebView example — a real embedded webview driven by dioxus signals.
//!
//! The webview is created through the local renderer/runtime embedded path,
//! not through `openharmony_ability::WebViewBuilder`. That keeps callback
//! argument handling under this crate's control and mounts the WebView as an
//! ArkUI native child of the dioxus host node.

use std::rc::Rc;

use arkit::entry;
use arkit::prelude::*;

const RUST_URL: &str = "https://www.rust-lang.org";
const DOCS_URL: &str = "https://docs.rs";
const WEBVIEW_ID: &str = "arkit-example-webview";

#[entry]
fn app() -> Element {
    let webview = use_context_provider(|| EmbeddedWebViewController::new(WEBVIEW_ID));

    let url = use_signal(|| RUST_URL.to_string());
    let title = use_signal(|| String::from("loading..."));
    let status = use_signal(|| String::from("ready"));
    let zoom = use_signal(|| 1.0_f64);

    let title_display = (title.read()).clone();
    let status_display = (status.read()).clone();
    let url_display = (url.read()).clone();
    let zoom_display = *zoom.read();

    let wv_reload = webview.clone();
    let mut status_reload = status;
    let wv_focus = webview.clone();
    let mut status_focus = status;
    let wv_zoom_in = webview.clone();
    let mut zoom_in = zoom;
    let mut status_zoom_in = status;
    let wv_zoom_out = webview.clone();
    let mut zoom_out = zoom;
    let mut status_zoom_out = status;
    let mut url_rust = url;
    let mut status_rust = status;
    let wv_rust = webview.clone();
    let mut url_docs = url;
    let mut status_docs = status;
    let wv_docs = webview.clone();
    let mut url_input = url;

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            background_color: 0xFFF6F7FBu32,

            column {
                padding: 16.0,
                background_color: 0xFFFFFFFFu32,

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
                    font_color: 0xFF334155u32,
                    "title: {title_display}"
                }
                text {
                    margin_top: 4.0,
                    font_size: 13.0,
                    line_height: 18.0,
                    font_color: 0xFF64748Bu32,
                    "url: {url_display}"
                }
                text {
                    margin_top: 4.0,
                    font_size: 13.0,
                    line_height: 18.0,
                    font_color: 0xFF0F766Eu32,
                    "status: {status_display}  (zoom {zoom_display:.2})"
                }

                textinput {
                    margin_top: 12.0,
                    padding: 10.0,
                    font_size: 14.0,
                    background_color: 0xFFF1F5F9u32,
                    border_radius: 8.0,
                    value: url_display.clone(),
                    placeholder: "enter url",
                    onchange: move |evt| {
                        url_input.set(evt.string_value.clone());
                    }
                }

                row {
                    margin_top: 12.0,
                    button {
                        onclick: move |_| {
                            let result = wv_reload.reload();
                            status_reload.set(match result {
                                Ok(()) => String::from("reloaded page"),
                                Err(e) => format!("reload failed: {e}"),
                            });
                        },
                        "Reload"
                    }
                    button {
                        margin_left: 8.0,
                        onclick: move |_| {
                            let result = wv_focus.focus();
                            status_focus.set(match result {
                                Ok(()) => String::from("webview focused"),
                                Err(e) => format!("focus failed: {e}"),
                            });
                        },
                        "Focus"
                    }
                    button {
                        margin_left: 8.0,
                        onclick: move |_| {
                            let next = *zoom_in.read() + 0.1;
                            zoom_in.set(next);
                            let result = wv_zoom_in.set_zoom(next);
                            status_zoom_in.set(match result {
                                Ok(()) => format!("zoom set to {next:.2}"),
                                Err(e) => format!("zoom failed: {e}"),
                            });
                        },
                        "Zoom +"
                    }
                    button {
                        margin_left: 8.0,
                        onclick: move |_| {
                            let next = (*zoom_out.read() - 0.1).max(0.1);
                            zoom_out.set(next);
                            let result = wv_zoom_out.set_zoom(next);
                            status_zoom_out.set(match result {
                                Ok(()) => format!("zoom set to {next:.2}"),
                                Err(e) => format!("zoom failed: {e}"),
                            });
                        },
                        "Zoom -"
                    }
                }

                row {
                    margin_top: 8.0,
                    button {
                        onclick: move |_| {
                            url_rust.set(RUST_URL.to_string());
                            let result = wv_rust.load_url(RUST_URL);
                            status_rust.set(match result {
                                Ok(()) => String::from("loaded rust-lang.org"),
                                Err(e) => format!("load failed: {e}"),
                            });
                        },
                        "rust-lang.org"
                    }
                    button {
                        margin_left: 8.0,
                        onclick: move |_| {
                            url_docs.set(DOCS_URL.to_string());
                            let result = wv_docs.load_url(DOCS_URL);
                            status_docs.set(match result {
                                Ok(()) => String::from("loaded docs.rs"),
                                Err(e) => format!("load failed: {e}"),
                            });
                        },
                        "docs.rs"
                    }
                }
            }

            WebviewArea {
                url,
                title,
                status,
            }
        }
    }
}

#[component]
fn WebviewArea(
    url: dioxus_signals::Signal<String>,
    title: dioxus_signals::Signal<String>,
    status: dioxus_signals::Signal<String>,
) -> Element {
    let webview: EmbeddedWebViewController = use_context();
    let lifecycle_visible = use_app_foreground() && use_component_visibility();

    let lifecycle_webview = webview.clone();
    let mut lifecycle_status = status;
    use_effect(use_reactive(&lifecycle_visible, move |visible| {
        if let Err(error) = lifecycle_webview.set_visible(visible) {
            lifecycle_status.set(format!("webview visibility update failed: {error}"));
        }
    }));

    let url_for_frame = url;
    let title_sig = title;
    let status_sig = status;
    let webview_for_frame = webview.clone();

    use_layout_frame_node(move |mut host_node, frame| {
        if !frame.is_measured() {
            return;
        }

        let mut init = EmbeddedWebViewInit::url(WEBVIEW_ID, (url_for_frame)());
        init.style = Some(WebViewStyle {
            x: None,
            y: None,
            visible: Some(true),
            background_color: Some("#FFFFFFFF".to_string()),
        });

        let title_cb_sig = title_sig;
        init.on_title_change = Some(Rc::new(move |new_title| {
            let mut sig = title_cb_sig;
            queue_ui_loop(move || {
                sig.set(new_title);
            });
        }));

        let result = webview_for_frame.mount_or_sync(
            &mut host_node,
            init,
            Some(WebViewFrame {
                width: frame.width,
                height: frame.height,
            }),
        );

        match result {
            Ok(()) if webview_for_frame.is_mounted() => {
                let mut sig = status_sig;
                queue_ui_loop(move || {
                    sig.set(String::from("webview mounted"));
                });
            }
            Ok(()) => {}
            Err(err) => {
                let mut sig = status_sig;
                let msg = err.to_string();
                queue_ui_loop(move || {
                    sig.set(format!("webview mount failed: {msg}"));
                });
            }
        }
    });

    let webview_for_drop = webview.clone();
    use_drop(move || {
        webview_for_drop.dispose();
    });

    rsx! {
        stack {
            percent_width: 1.0,
            height: 400.0,
            background_color: 0xFFFFFFFFu32,
        }
    }
}
