use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use arkit_arkui::MountedNodeLease;
use napi_ohos::bindgen_prelude::{Function, JsObjectValue, Object, ObjectRef};
use napi_ohos::{Either, Error, Result};
use ohos_arkui_binding::common::node::{ArkUINode, ArkUINodeRaw};
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use openharmony_ability::{get_helper, get_main_thread_env, WebViewInitData, Webview};

pub use openharmony_ability::WebViewStyle;

/// Layout frame used to size an embedded WebView inside a native ArkUI host.
///
/// The ArkTS `ComponentContent` root is a `FrameNode` with no measure
/// callback of its own; inside a native (CAPI) tree it would measure to zero
/// and the Web component would never create its surface. The host's measured
/// frame is therefore applied explicitly to the node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebViewFrame {
    pub width: f32,
    pub height: f32,
}

impl WebViewFrame {
    pub fn is_valid(self) -> bool {
        self.layout_rect_px().is_some()
    }

    fn layout_rect_px(self) -> Option<[i32; 4]> {
        fn physical_extent(value: f32) -> Option<i32> {
            let rounded = f64::from(value).round();
            (rounded >= 1.0 && rounded <= f64::from(i32::MAX)).then_some(rounded as i32)
        }

        Some([
            0,
            0,
            physical_extent(self.width)?,
            physical_extent(self.height)?,
        ])
    }
}

/// Initial configuration for a renderer-owned embedded WebView.
#[derive(Clone)]
pub struct EmbeddedWebViewInit {
    pub id: String,
    pub url: Option<String>,
    pub html: Option<String>,
    pub style: Option<WebViewStyle>,
    pub javascript_enabled: Option<bool>,
    pub devtools: Option<bool>,
    pub user_agent: Option<String>,
    pub autoplay: Option<bool>,
    pub initialization_scripts: Option<Vec<String>>,
    pub headers: Option<HashMap<String, String>>,
    pub transparent: Option<bool>,
    pub on_navigation_request: Option<Rc<dyn Fn(String) -> bool>>,
    pub on_title_change: Option<Rc<dyn Fn(String)>>,
}

impl EmbeddedWebViewInit {
    pub fn url(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            url: Some(url.into()),
            html: None,
            style: None,
            javascript_enabled: None,
            devtools: None,
            user_agent: None,
            autoplay: None,
            initialization_scripts: None,
            headers: None,
            transparent: None,
            on_navigation_request: None,
            on_title_change: None,
        }
    }
}

struct EmbeddedWebViewState {
    id: String,
    mount_generation: u64,
    webview: Option<Webview>,
    node: Option<ArkUINode>,
    frame: Option<WebViewFrame>,
    current_url: Option<String>,
    current_html: Option<String>,
    desired_visible: bool,
}

impl Default for EmbeddedWebViewState {
    fn default() -> Self {
        Self {
            id: String::new(),
            mount_generation: 0,
            webview: None,
            node: None,
            frame: None,
            current_url: None,
            current_html: None,
            desired_visible: true,
        }
    }
}
///
/// Native WebView operations may synchronously invoke callbacks. Keeping an
/// owned snapshot guarantees that no `RefCell` borrow crosses that re-entrant
/// boundary.
enum EmbeddedWebViewSnapshot {
    Unmounted {
        desired_visible: bool,
    },
    Mounted {
        node: ArkUINode,
        webview: Webview,
        current_url: Option<String>,
        current_html: Option<String>,
    },
}

impl EmbeddedWebViewState {
    fn snapshot_for(&self, requested_id: &str) -> Result<EmbeddedWebViewSnapshot> {
        match (&self.webview, &self.node) {
            (None, None) => Ok(EmbeddedWebViewSnapshot::Unmounted {
                desired_visible: self.desired_visible,
            }),
            (Some(webview), Some(node)) => {
                if requested_id != self.id {
                    return Err(Error::from_reason(format!(
                        "embedded webview id cannot change after mount ({} -> {requested_id})",
                        self.id
                    )));
                }
                Ok(EmbeddedWebViewSnapshot::Mounted {
                    node: node.clone(),
                    webview: webview.clone(),
                    current_url: self.current_url.clone(),
                    current_html: self.current_html.clone(),
                })
            }
            _ => Err(Error::from_reason(
                "embedded webview controller state is inconsistent",
            )),
        }
    }
}

/// Imperative handle for a WebView mounted as an ArkUI native child.
///
/// This intentionally bypasses `openharmony_ability::WebViewBuilder`: that
/// builder creates overlay WebViews and its callback shims currently read NAPI
/// callback arguments from index 1. ArkTS calls those callbacks with the first
/// payload at index 0, so the overlay path crashes on title/navigation events.
#[derive(Clone, Default)]
pub struct EmbeddedWebViewController {
    inner: Rc<RefCell<EmbeddedWebViewState>>,
}

impl EmbeddedWebViewController {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(EmbeddedWebViewState {
                id: id.into(),
                ..EmbeddedWebViewState::default()
            })),
        }
    }

    pub fn id(&self) -> String {
        self.inner.borrow().id.clone()
    }

    pub fn is_mounted(&self) -> bool {
        self.inner.borrow().webview.is_some()
    }

    pub fn mount_or_sync(
        &self,
        host: &MountedNodeLease,
        mut init: EmbeddedWebViewInit,
        frame: Option<WebViewFrame>,
    ) -> Result<()> {
        if init.id.is_empty() {
            init.id = self.id();
        }
        if init.id.is_empty() {
            return Err(Error::from_reason("embedded webview id must not be empty"));
        }

        let requested_url = init.url.clone();
        let requested_html = init.html.clone();
        let snapshot = {
            let state = self.inner.borrow();
            state.snapshot_for(&init.id)?
        };

        match snapshot {
            EmbeddedWebViewSnapshot::Unmounted { desired_visible } => {
                let mount = create_embedded_webview(init)?;
                if let Err(error) = mount.webview.set_visible(desired_visible) {
                    if let Err(dispose_error) = mount.webview.dispose() {
                        ohos_hilog_binding::error(format!(
                        "embedded webview cleanup after visibility failure failed: {dispose_error}"
                    ));
                    }
                    return Err(error);
                }
                if let Err(error) = attach_embedded_node_to_lease(host, &mount.node) {
                    // The ArkTS manager owns the external content node. Disposing
                    // its controller releases that entry after a failed attach.
                    if let Err(dispose_error) = mount.webview.dispose() {
                        ohos_hilog_binding::error(format!(
                            "embedded webview cleanup after attach failure failed: {dispose_error}"
                        ));
                    }
                    return Err(error);
                }
                // ComponentContent recalculates its builder root when it is
                // inserted into the CAPI tree. Apply the explicit rect only
                // after attachment; setting it before insertion is silently
                // overwritten by that first parent measure and leaves the Web
                // surface at 0x0 even though navigation callbacks still fire.
                let applied_frame = if let Some(frame) = frame {
                    match apply_webview_frame(&mount.node, frame) {
                        Ok(true) => Some(frame),
                        Ok(false) => None,
                        Err(error) => {
                            if let Err(dispose_error) = mount.webview.dispose() {
                                ohos_hilog_binding::error(format!(
                                    "embedded webview cleanup after frame failure failed: {dispose_error}"
                                ));
                            }
                            return Err(error);
                        }
                    }
                } else {
                    None
                };
                let generation = {
                    let mut state = self.inner.borrow_mut();
                    state.mount_generation = state
                        .mount_generation
                        .checked_add(1)
                        .expect("arkit_runtime: embedded webview mount generation exhausted");
                    state.id = mount.id;
                    state.node = Some(mount.node);
                    state.webview = Some(mount.webview);
                    state.frame = applied_frame;
                    state.current_url = requested_url;
                    state.current_html = requested_html;
                    state.mount_generation
                };

                let teardown = self.clone();
                // SAFETY: cleanup only disposes this controller's external WebView
                // child before the renderer invalidates its host node.
                let installed = unsafe {
                    host.install_native_teardown(move || {
                        teardown.dispose_generation(generation);
                    })
                };
                if !installed {
                    self.dispose_generation(generation);
                    return Err(Error::from_reason(
                        "embedded webview host was unmounted during attachment",
                    ));
                }
            }
            EmbeddedWebViewSnapshot::Mounted {
                node,
                webview,
                current_url: previous_url,
                current_html: previous_html,
            } => {
                attach_embedded_node_to_lease(host, &node)?;
                if let Some(html) = requested_html.as_deref() {
                    if requested_html != previous_html {
                        webview.load_html(html)?;
                    }
                    let mut state = self.inner.borrow_mut();
                    state.current_url = requested_url;
                    state.current_html = requested_html;
                } else if let Some(url) = requested_url.as_deref() {
                    if previous_html.is_some() || requested_url != previous_url {
                        webview.load_url(url)?;
                    }
                    let mut state = self.inner.borrow_mut();
                    state.current_url = requested_url;
                    state.current_html = None;
                }
            }
        }

        if let Some(frame) = frame {
            self.sync_frame(frame)?;
        }

        Ok(())
    }

    /// Apply the host's measured frame to the embedded WebView node.
    ///
    /// The node resolved from the ArkTS `ComponentContent` is a builder node;
    /// the native node API rejects `POSITION`/`SIZE` attributes on such nodes
    /// with 106103 (`ARKTS_NODE_NOT_SUPPORTED`) and only permits
    /// `LAYOUT_RECT`, which sets position and size in a single attribute. That
    /// attribute specifically consumes four signed integers in physical pixels;
    /// passing ArkUI's float attribute variant makes native read the union with
    /// the wrong type and leaves the embedded surface with unusable bounds.
    /// Without an explicit frame the Web component would render at 0x0 inside
    /// the native tree and its surface would never be created (blank page).
    pub fn sync_frame(&self, frame: WebViewFrame) -> Result<()> {
        if !frame.is_valid() {
            return Ok(());
        }
        if self.inner.borrow().frame == Some(frame) {
            return Ok(());
        }
        let Some(node) = self.inner.borrow().node.clone() else {
            return Ok(());
        };
        apply_webview_frame(&node, frame)?;
        self.inner.borrow_mut().frame = Some(frame);
        Ok(())
    }

    pub fn with_webview<R>(&self, f: impl FnOnce(&Webview) -> Result<R>) -> Result<R> {
        let webview = self
            .inner
            .borrow()
            .webview
            .clone()
            .ok_or_else(|| Error::from_reason("webview not mounted"))?;
        f(&webview)
    }

    pub fn reload(&self) -> Result<()> {
        self.with_webview(|webview| webview.reload())
    }

    pub fn focus(&self) -> Result<()> {
        self.with_webview(|webview| webview.focus())
    }

    /// Set whether the embedded WebView participates in presentation.
    ///
    /// The desired value is retained before mount so application/component
    /// lifecycle hooks cannot race the first native WebView creation.
    pub fn set_visible(&self, visible: bool) -> Result<()> {
        let webview = self.inner.borrow().webview.clone();
        if let Some(webview) = webview {
            webview.set_visible(visible)?;
        }
        self.inner.borrow_mut().desired_visible = visible;
        Ok(())
    }

    pub fn is_visible(&self) -> bool {
        self.inner.borrow().desired_visible
    }

    pub fn load_url(&self, url: &str) -> Result<()> {
        self.with_webview(|webview| webview.load_url(url))?;
        let mut state = self.inner.borrow_mut();
        state.current_url = Some(url.to_owned());
        state.current_html = None;
        Ok(())
    }

    pub fn load_html(&self, html: &str) -> Result<()> {
        self.with_webview(|webview| webview.load_html(html))?;
        let mut state = self.inner.borrow_mut();
        state.current_url = None;
        state.current_html = Some(html.to_owned());
        Ok(())
    }

    pub fn set_zoom(&self, zoom: f64) -> Result<()> {
        self.with_webview(|webview| webview.set_zoom(zoom))
    }

    pub fn clear_all_browsing_data(&self) -> Result<()> {
        self.with_webview(|webview| webview.clear_all_browsing_data())
    }

    pub fn dispose(&self) {
        if let Err(error) = self.try_dispose() {
            ohos_hilog_binding::error(format!("embedded webview dispose failed: {error}"));
        }
    }

    fn dispose_generation(&self, generation: u64) {
        let is_current = {
            let state = self.inner.borrow();
            state.mount_generation == generation && state.webview.is_some()
        };
        if is_current {
            self.dispose();
        }
    }

    /// Dispose the ArkTS-owned WebView and clear the mounted snapshot only
    /// after native cleanup succeeds. Callers that need teardown diagnostics
    /// can use this instead of the best-effort [`dispose`](Self::dispose).
    pub fn try_dispose(&self) -> Result<()> {
        let webview = self.inner.borrow().webview.clone();
        if let Some(webview) = webview {
            webview.dispose()?;
        }
        let mut state = self.inner.borrow_mut();
        state.webview = None;
        state.node = None;
        state.frame = None;
        state.current_url = None;
        state.current_html = None;
        Ok(())
    }
}

impl Drop for EmbeddedWebViewController {
    fn drop(&mut self) {
        if Rc::strong_count(&self.inner) == 1 {
            self.dispose();
        }
    }
}

struct EmbeddedWebViewMount {
    id: String,
    webview: Webview,
    node: ArkUINode,
}

fn create_embedded_webview(init: EmbeddedWebViewInit) -> Result<EmbeddedWebViewMount> {
    // SAFETY: embedded WebViews are created from the registered UI-loop
    // effect. `set_helper` installs this thread-local N-API reference during
    // entry rendering; the borrow below verifies that installation before use.
    let helper = unsafe { get_helper() };
    let helper_borrow = helper.borrow();
    let helper_ref = helper_borrow
        .as_ref()
        .ok_or_else(|| Error::from_reason("arkts helper is not available"))?;

    let env = get_main_thread_env();
    let env_borrow = env.borrow();
    let env_ref = env_borrow
        .as_ref()
        .ok_or_else(|| Error::from_reason("main thread env is not available"))?;

    let helper_object = helper_ref.get_value(env_ref)?;
    let create_webview = helper_object
        .get_named_property::<Function<'_, WebViewInitData<'_>, ObjectRef>>(
            "createEmbeddedWebview",
        )?;

    let on_navigation_request = init
        .on_navigation_request
        .as_ref()
        .map(|handler| {
            let handler = handler.clone();
            env_ref.create_function_from_closure("arkit_on_navigation_request", move |ctx| {
                let url = ctx.try_get::<String>(0)?;
                let url = match url {
                    Either::A(value) => value,
                    Either::B(_) => String::new(),
                };
                Ok(handler(url))
            })
        })
        .transpose()?;

    let on_title_change = init
        .on_title_change
        .as_ref()
        .map(|handler| {
            let handler = handler.clone();
            env_ref.create_function_from_closure("arkit_on_title_change", move |ctx| {
                let title = ctx.try_get::<String>(0)?;
                let title = match title {
                    Either::A(value) => value,
                    Either::B(_) => String::new(),
                };
                handler(title);
                Ok(())
            })
        })
        .transpose()?;

    let embedded_webview = create_webview.call(WebViewInitData {
        url: init.url,
        id: Some(init.id.clone()),
        style: init.style,
        javascript_enabled: init.javascript_enabled,
        devtools: init.devtools,
        user_agent: init.user_agent,
        autoplay: init.autoplay,
        initialization_scripts: init.initialization_scripts,
        headers: init.headers,
        html: init.html,
        transparent: init.transparent,
        on_drag_and_drop: None,
        on_download_start: None,
        on_download_end: None,
        on_navigation_request,
        on_title_change,
    })?;

    let embedded_value = embedded_webview.get_value(env_ref)?;
    let controller_object = embedded_value
        .get::<Object>("controller")?
        .ok_or_else(|| Error::from_reason("embedded webview controller is missing"))?;
    let node_raw = embedded_value
        .get::<ArkUINodeRaw>("content")?
        .ok_or_else(|| Error::from_reason("embedded webview content is missing"))?;
    let controller_ref = controller_object.create_ref::<true>()?;
    let node = ArkUINode::from_raw_handle(node_raw.raw)
        .ok_or_else(|| Error::from_reason("embedded webview content handle is null"))?;
    let webview = Webview::new(init.id.clone(), controller_ref)?;

    Ok(EmbeddedWebViewMount {
        id: init.id,
        webview,
        node,
    })
}

fn attach_embedded_node(host: &mut ArkUINode, node: &ArkUINode) -> Result<()> {
    let raw = node.raw_handle();
    if host
        .children()
        .iter()
        .any(|child| child.borrow().raw_handle() == raw)
    {
        return Ok(());
    }
    host.add_existing_child(node.clone())
        .map_err(map_arkui_error)
}

fn attach_embedded_node_to_lease(host: &MountedNodeLease, node: &ArkUINode) -> Result<()> {
    // SAFETY: EmbeddedWebViewController is the framework-owned projection
    // boundary for this external content node. It only adds its own child and
    // never disposes or retains the borrowed renderer host.
    unsafe { host.with_native_mut(|host| attach_embedded_node(host, node)) }
        .ok_or_else(|| Error::from_reason("embedded webview host is no longer mounted"))?
}

fn apply_webview_frame(node: &ArkUINode, frame: WebViewFrame) -> Result<bool> {
    let Some(layout_rect) = frame.layout_rect_px() else {
        return Ok(false);
    };
    node.set_layout_rect(layout_rect.to_vec())
        .map_err(map_arkui_error)?;
    Ok(true)
}

fn map_arkui_error(error: impl ToString) -> Error {
    Error::from_reason(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webview_layout_rect_uses_integer_physical_pixels() {
        let frame = WebViewFrame {
            width: 320.6,
            height: 123.4,
        };

        assert_eq!(frame.layout_rect_px(), Some([0, 0, 321, 123]));
    }

    #[test]
    fn webview_layout_rect_rejects_invalid_extents() {
        assert!(!WebViewFrame {
            width: f32::NAN,
            height: 100.0,
        }
        .is_valid());
        assert!(!WebViewFrame {
            width: 100.0,
            height: 0.0,
        }
        .is_valid());
    }
}
