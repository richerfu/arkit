//! Custom `openharmony-ability` bridge plugin registration.
//!
//! Demonstrates both `#[entry]` registration styles for application-owned
//! `BridgePlugin` facades:
//!
//! 1. Declarative list — `#[entry(plugins = [DeclarativeBridgePlugin])]`.
//!    The generated `init` registers each expression after the
//!    framework-owned plugins, before any ArkTS plugin event is delivered.
//!    Items resolve at module scope, so unit types and constructor calls
//!    both work; registration failures are logged, not fatal.
//! 2. App handle argument — the entry fn receives an `OpenHarmonyApp`
//!    clone at first render (after the bridge bindings are installed), so
//!    `handle.register_plugin(...)` can run arbitrary setup.
//!
//! The ArkTS counterpart of a plugin (when one exists) is installed in the
//! host ability's `bridgePlugins` array — that side is hand-managed.

use arkit::entry;
use arkit::prelude::*;
use openharmony_ability::{AsyncBridge, BridgePlugin, OpenHarmonyApp, PluginLifecycleEvent};

/// Style 1: registered declaratively via `#[entry(plugins = [...])]` at
/// ability-init, before any ArkTS plugin event reaches the registry.
struct DeclarativeBridgePlugin;

impl BridgePlugin for DeclarativeBridgePlugin {
    type Mode = AsyncBridge;

    const ID: &'static str = "arkit.example.declarative";

    fn on_lifecycle(&self, event: &PluginLifecycleEvent) -> napi_ohos::Result<()> {
        ohos_hilog_binding::info(format!(
            "plugins_example: declarative({}) lifecycle: {event:?}",
            Self::ID
        ));
        Ok(())
    }
}

/// Style 2: an instance registered manually from the entry fn, which
/// receives a clone of the shared ability handle at first render.
struct ManualBridgePlugin {
    label: &'static str,
}

impl BridgePlugin for ManualBridgePlugin {
    type Mode = AsyncBridge;

    const ID: &'static str = "arkit.example.manual";

    fn on_lifecycle(&self, event: &PluginLifecycleEvent) -> napi_ohos::Result<()> {
        ohos_hilog_binding::info(format!(
            "plugins_example: manual({}) lifecycle: {event:?}",
            self.label
        ));
        Ok(())
    }
}

#[entry(plugins = [DeclarativeBridgePlugin])]
fn app(handle: OpenHarmonyApp) -> Element {
    // Manual registration: runs inside the first dioxus render, after
    // `openharmony_ability::render` installed the bridge bindings. Late
    // registration is safe — the registry replays the bounded lifecycle
    // history to plugins whose `REQUIRED_CONTEXTS` are satisfied.
    match handle.register_plugin(ManualBridgePlugin {
        label: "from-entry-fn",
    }) {
        Ok(()) => ohos_hilog_binding::info("plugins_example: manual plugin registered"),
        Err(error) => ohos_hilog_binding::error(format!(
            "plugins_example: manual plugin registration failed: {error}"
        )),
    }
    rsx! {
        column {
            text { "custom bridge plugins registered" }
        }
    }
}
