use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use napi_ohos::bindgen_prelude::{Function, JsObjectValue, Object, ObjectRef};
use napi_ohos::{Either, Error, Result};
use ohos_arkui_binding::common::node::{ArkUINode, ArkUINodeRaw};
use ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use openharmony_ability::{get_helper, get_main_thread_env, WebViewInitData, Webview};

pub use openharmony_ability::WebViewStyle;

/// Layout frame used to size an embedded WebView inside a native ArkUI host.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebViewFrame {
    pub width: f32,
    pub height: f32,
}

impl WebViewFrame {
    pub fn is_valid(self) -> bool {
        self.width.is_finite() && self.height.is_finite() && self.width > 0.0 && self.height > 0.0
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
            webview: None,
            node: None,
            frame: None,
            current_url: None,
            current_html: None,
            desired_visible: true,
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
        host: &mut ArkUINode,
        mut init: EmbeddedWebViewInit,
        frame: Option<WebViewFrame>,
    ) -> Result<()> {
        if init.id.is_empty() {
            init.id = self.id();
        }
        if init.id.is_empty() {
            return Err(Error::from_reason("embedded webview id must not be empty"));
        }
        if self.inner.borrow().webview.is_some() && init.id != self.inner.borrow().id {
            return Err(Error::from_reason(format!(
                "embedded webview id cannot change after mount ({} -> {})",
                self.inner.borrow().id,
                init.id
            )));
        }

        let requested_url = init.url.clone();
        let requested_html = init.html.clone();

        if self.inner.borrow().webview.is_none() {
            let mount = create_embedded_webview(init)?;
            let desired_visible = self.inner.borrow().desired_visible;
            if let Err(error) = mount.webview.set_visible(desired_visible) {
                if let Err(dispose_error) = mount.webview.dispose() {
                    ohos_hilog_binding::error(format!(
                        "embedded webview cleanup after visibility failure failed: {dispose_error}"
                    ));
                }
                return Err(error);
            }
            if let Err(error) = attach_embedded_node(host, &mount.node) {
                // The ArkTS manager owns the external content node. Disposing
                // its controller releases that entry after a failed attach.
                if let Err(dispose_error) = mount.webview.dispose() {
                    ohos_hilog_binding::error(format!(
                        "embedded webview cleanup after attach failure failed: {dispose_error}"
                    ));
                }
                return Err(error);
            }
            let mut state = self.inner.borrow_mut();
            state.id = mount.id;
            state.node = Some(mount.node);
            state.webview = Some(mount.webview);
            state.current_url = requested_url;
            state.current_html = requested_html;
        } else if let Some(node) = self.inner.borrow().node.clone() {
            attach_embedded_node(host, &node)?;

            let (webview, previous_url, previous_html) = {
                let state = self.inner.borrow();
                (
                    state.webview.clone(),
                    state.current_url.clone(),
                    state.current_html.clone(),
                )
            };
            if let Some(webview) = webview {
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
        node.set_position(vec![0.0_f32, 0.0_f32])
            .map_err(map_arkui_error)?;
        node.set_size(vec![frame.width, frame.height])
            .map_err(map_arkui_error)?;
        node.set_layout_rect(vec![0.0_f32, 0.0_f32, frame.width, frame.height])
            .map_err(map_arkui_error)?;
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

fn map_arkui_error(error: impl ToString) -> Error {
    Error::from_reason(error.to_string())
}
