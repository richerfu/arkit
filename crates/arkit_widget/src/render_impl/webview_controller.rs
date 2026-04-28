use super::*;

pub(super) fn run_with_webview_dispatcher<R>(
    dispatcher: Option<arkit_runtime::internal::GlobalRuntimeDispatcher>,
    f: impl FnOnce() -> R,
) -> R {
    arkit_runtime::internal::with_global_dispatcher(dispatcher, f)
}

pub(super) static NEXT_WEBVIEW_ID: AtomicU64 = AtomicU64::new(1);

#[cfg(feature = "webview")]
pub(super) type WebViewReadyCallback = Rc<dyn Fn(&Webview)>;
#[cfg(feature = "webview")]
pub(super) type WebViewLifecycleCallback = Rc<RefCell<Box<dyn FnMut()>>>;
#[cfg(feature = "webview")]
pub(super) type DragAndDropCallback = Rc<dyn Fn(String)>;
#[cfg(feature = "webview")]
pub(super) type DownloadStartCallback = Rc<dyn Fn(String, &mut PathBuf) -> bool>;
#[cfg(feature = "webview")]
pub(super) type DownloadEndCallback = Rc<dyn Fn(String, Option<PathBuf>, bool)>;
#[cfg(feature = "webview")]
pub(super) type NavigationRequestCallback = Rc<dyn Fn(String) -> bool>;
#[cfg(feature = "webview")]
pub(super) type TitleChangeCallback = Rc<dyn Fn(String)>;

#[cfg(feature = "webview")]
#[derive(Debug, Clone, Default)]
pub(super) struct WebViewAppliedState {
    pub(super) url: Option<String>,
    pub(super) html: Option<String>,
    pub(super) headers: Option<HashMap<String, String>>,
    pub(super) style: Option<WebViewStyle>,
}

#[cfg(feature = "webview")]
pub(super) struct WebViewControllerState {
    pub(super) id: String,
    pub(super) webview: RefCell<Option<Webview>>,
    pub(super) embedded_node: RefCell<Option<ArkUINode>>,
    pub(super) ready_callbacks: RefCell<Vec<WebViewReadyCallback>>,
    pub(super) controller_attach_callbacks: RefCell<Vec<WebViewLifecycleCallback>>,
    pub(super) page_begin_callbacks: RefCell<Vec<WebViewLifecycleCallback>>,
    pub(super) page_end_callbacks: RefCell<Vec<WebViewLifecycleCallback>>,
    pub(super) destroy_callbacks: RefCell<Vec<WebViewLifecycleCallback>>,
    pub(super) applied: RefCell<Option<WebViewAppliedState>>,
    pub(super) active_binding: Cell<bool>,
    pub(super) attached: Cell<bool>,
}

#[cfg(feature = "webview")]
impl WebViewControllerState {
    fn new(id: String) -> Self {
        Self {
            id,
            webview: RefCell::new(None),
            embedded_node: RefCell::new(None),
            ready_callbacks: RefCell::new(Vec::new()),
            controller_attach_callbacks: RefCell::new(Vec::new()),
            page_begin_callbacks: RefCell::new(Vec::new()),
            page_end_callbacks: RefCell::new(Vec::new()),
            destroy_callbacks: RefCell::new(Vec::new()),
            applied: RefCell::new(None),
            active_binding: Cell::new(false),
            attached: Cell::new(false),
        }
    }
}

#[cfg(feature = "webview")]
#[derive(Clone)]
pub struct WebViewController {
    pub(super) inner: Rc<WebViewControllerState>,
}

#[cfg(feature = "webview")]
impl Default for WebViewController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "webview")]
impl WebViewController {
    pub fn new() -> Self {
        let id = format!(
            "arkit-webview-{}",
            NEXT_WEBVIEW_ID.fetch_add(1, Ordering::Relaxed)
        );
        Self::with_id(id)
    }

    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            inner: Rc::new(WebViewControllerState::new(id.into())),
        }
    }

    pub fn id(&self) -> &str {
        self.inner.id.as_str()
    }

    pub fn handle(&self) -> Result<Webview, String> {
        self.inner
            .webview
            .borrow()
            .clone()
            .ok_or_else(|| format!("webview '{}' is not mounted", self.id()))
    }

    pub fn on_ready(&self, callback: impl Fn(&Webview) + 'static) {
        let callback = Rc::new(callback) as WebViewReadyCallback;
        if let Some(webview) = self.inner.webview.borrow().as_ref() {
            callback(webview);
        }
        self.inner.ready_callbacks.borrow_mut().push(callback);
    }

    pub fn on_controller_attach(&self, callback: impl FnMut() + 'static) -> Result<(), String> {
        let callback: WebViewLifecycleCallback = Rc::new(RefCell::new(Box::new(callback)));
        if let Some(webview) = self.inner.webview.borrow().as_ref() {
            register_lifecycle_callback(webview, &callback, WebViewLifecycle::ControllerAttach)?;
        }
        self.inner
            .controller_attach_callbacks
            .borrow_mut()
            .push(callback);
        Ok(())
    }

    pub fn on_page_begin(&self, callback: impl FnMut() + 'static) -> Result<(), String> {
        let callback: WebViewLifecycleCallback = Rc::new(RefCell::new(Box::new(callback)));
        if let Some(webview) = self.inner.webview.borrow().as_ref() {
            register_lifecycle_callback(webview, &callback, WebViewLifecycle::PageBegin)?;
        }
        self.inner.page_begin_callbacks.borrow_mut().push(callback);
        Ok(())
    }

    pub fn on_page_end(&self, callback: impl FnMut() + 'static) -> Result<(), String> {
        let callback: WebViewLifecycleCallback = Rc::new(RefCell::new(Box::new(callback)));
        if let Some(webview) = self.inner.webview.borrow().as_ref() {
            register_lifecycle_callback(webview, &callback, WebViewLifecycle::PageEnd)?;
        }
        self.inner.page_end_callbacks.borrow_mut().push(callback);
        Ok(())
    }

    pub fn on_destroy(&self, callback: impl FnMut() + 'static) -> Result<(), String> {
        let callback: WebViewLifecycleCallback = Rc::new(RefCell::new(Box::new(callback)));
        if let Some(webview) = self.inner.webview.borrow().as_ref() {
            register_lifecycle_callback(webview, &callback, WebViewLifecycle::Destroy)?;
        }
        self.inner.destroy_callbacks.borrow_mut().push(callback);
        Ok(())
    }

    pub fn url(&self) -> Result<String, String> {
        self.with_handle(|webview| webview.url())
    }

    pub fn cookies_with_url(&self, url: &str) -> Result<String, String> {
        self.with_handle(|webview| webview.cookies_with_url(url))
    }

    pub fn load_url(&self, url: &str) -> Result<(), String> {
        self.with_handle(|webview| webview.load_url(url))
    }

    pub fn load_url_with_headers(
        &self,
        url: &str,
        headers: impl IntoIterator<Item = (String, String)>,
    ) -> Result<(), String> {
        let mut header_map = http::HeaderMap::new();
        for (key, value) in headers {
            let name = key
                .parse::<http::header::HeaderName>()
                .map_err(|error| error.to_string())?;
            let value = value
                .parse::<http::header::HeaderValue>()
                .map_err(|error| error.to_string())?;
            header_map.insert(name, value);
        }
        self.with_handle(|webview| webview.load_url_with_headers(url, header_map))
    }

    pub fn load_html(&self, html: &str) -> Result<(), String> {
        self.with_handle(|webview| webview.load_html(html))
    }

    pub fn reload(&self) -> Result<(), String> {
        self.with_handle(|webview| webview.reload())
    }

    pub fn focus(&self) -> Result<(), String> {
        self.with_handle(|webview| webview.focus())
    }

    pub fn set_zoom(&self, zoom: f64) -> Result<(), String> {
        self.with_handle(|webview| webview.set_zoom(zoom))
    }

    pub fn evaluate_script(&self, js: &str) -> Result<(), String> {
        self.with_handle(|webview| webview.evaluate_script(js))
    }

    pub fn evaluate_script_with_callback(
        &self,
        js: &str,
        callback: Option<Box<dyn Fn(String) + Send + 'static>>,
    ) -> Result<(), String> {
        let webview = self.handle()?;
        let env_state = get_main_thread_env();
        let env_borrow = env_state.borrow();
        let Some(env) = env_borrow.as_ref() else {
            return Err(String::from("failed to get main thread env"));
        };
        let run_javascript = webview
            .inner()
            .get_value(env)
            .map_err(|error| error.to_string())?
            .get_named_property::<Function<'_, FnArgs<(String, Function<'_, String, ()>)>, ()>>(
                "runJavaScript",
            )
            .map_err(|error| error.to_string())?;
        let callback_dispatcher = arkit_runtime::internal::global_dispatcher();

        let cb = env
            .create_function_from_closure("arkit_evaluate_js_callback", move |ctx| {
                let ret = ctx.try_get::<String>(0)?;
                let ret = match ret {
                    Either::A(value) => value,
                    Either::B(_undefined) => String::from("undefined"),
                };
                if let Some(callback) = callback.as_ref() {
                    run_with_webview_dispatcher(callback_dispatcher.clone(), move || callback(ret));
                }
                Ok(())
            })
            .map_err(|error| error.to_string())?;

        run_javascript
            .call((js.to_string(), cb).into())
            .map_err(|error| error.to_string())
    }

    pub fn clear_all_browsing_data(&self) -> Result<(), String> {
        self.with_handle(|webview| webview.clear_all_browsing_data())
    }

    pub fn set_background_color(&self, color: &str) -> Result<(), String> {
        self.with_handle(|webview| webview.set_background_color(color))
    }

    pub fn set_visible(&self, visible: bool) -> Result<(), String> {
        self.with_handle(|webview| webview.set_visible(visible))
    }

    fn with_handle<R, E: ToString>(
        &self,
        f: impl FnOnce(&Webview) -> Result<R, E>,
    ) -> Result<R, String> {
        let webview = self.handle()?;
        f(&webview).map_err(|error| error.to_string())
    }
}

#[cfg(feature = "webview")]
#[derive(Clone, Default)]
pub(super) struct WebViewSpec {
    pub(super) controller: Option<WebViewController>,
    pub(super) url: Option<String>,
    pub(super) html: Option<String>,
    pub(super) style: Option<WebViewStyle>,
    pub(super) javascript_enabled: Option<bool>,
    pub(super) devtools: Option<bool>,
    pub(super) transparent: Option<bool>,
    pub(super) autoplay: Option<bool>,
    pub(super) user_agent: Option<String>,
    pub(super) initialization_scripts: Option<Vec<String>>,
    pub(super) headers: Option<HashMap<String, String>>,
    pub(super) on_drag_and_drop: Option<DragAndDropCallback>,
    pub(super) on_download_start: Option<DownloadStartCallback>,
    pub(super) on_download_end: Option<DownloadEndCallback>,
    pub(super) on_navigation_request: Option<NavigationRequestCallback>,
    pub(super) on_title_change: Option<TitleChangeCallback>,
}

#[cfg(feature = "webview")]
#[derive(Clone, Copy)]
pub(super) enum WebViewLifecycle {
    ControllerAttach,
    PageBegin,
    PageEnd,
    Destroy,
}

pub fn web_view_component<Message, AppTheme>(
    controller: WebViewController,
) -> WebViewElement<Message, AppTheme> {
    let mut node = Node::new(NodeKind::WebViewHost)
        .key(controller.id().to_string())
        .clip(true)
        .hit_test_behavior(HitTestBehavior::Transparent);
    node.webview = Some(WebViewSpec {
        controller: Some(controller),
        ..WebViewSpec::default()
    });
    Component::from_node(node)
}

#[cfg(feature = "webview")]
pub fn web_view<Message, AppTheme>(
    controller: WebViewController,
    url: impl Into<String>,
) -> WebViewElement<Message, AppTheme> {
    web_view_component(controller)
        .percent_width(1.0)
        .percent_height(1.0)
        .url(url)
}
