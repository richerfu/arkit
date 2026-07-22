//! Async task example — dioxus `use_resource` (replaces legacy `Task::perform`).
//!
//! A button increments a request-id signal; `use_resource` reruns whenever the
//! id changes and awaits a real 800 ms `tokio::time::sleep` (driven by the
//! framework's tokio runtime via `arkit::tokio_handle`) before
//! producing a result string.

use std::time::Duration;

use arkit::entry;
use arkit::prelude::*;

#[entry]
fn app() -> Element {
    let mut request_id = use_signal(|| 0_u32);
    let handle = arkit::tokio_handle();

    let result = use_resource(move || {
        let handle = handle.clone();
        async move {
            let id = request_id();
            if id == 0 {
                return String::from("idle");
            }
            // Real async timing on the framework tokio runtime.
            handle
                .spawn(async move {
                    tokio::time::sleep(Duration::from_millis(800)).await;
                })
                .await
                .ok();
            format!("finished task #{id}")
        }
    });

    let status = match (result.value())() {
        Some(value) => value,
        None => String::from("running..."),
    };

    rsx! {
        column {
            width: "100%",
            height: "100%",
            align_items: "center",
            justify_content: "center",
            padding: 24.0,

            text { font_size: 28.0, line_height: 32.0, "arkit async task" }
            text { margin_top: 12.0, font_size: 18.0, line_height: 24.0, "{status}" }
            text {
                margin_top: 8.0,
                font_size: 14.0,
                line_height: 20.0,
                "latest request = {request_id}"
            }
            button {
                margin_top: 20.0,
                onclick: move |_| request_id += 1,
                "start async task"
            }
        }
    }
}
