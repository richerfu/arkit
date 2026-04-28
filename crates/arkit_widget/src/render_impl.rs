use std::any::{type_name, Any, TypeId};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::{align_of, size_of, ManuallyDrop};
use std::panic::{catch_unwind, AssertUnwindSafe};
#[cfg(feature = "webview")]
use std::path::PathBuf;
use std::rc::Rc;
#[cfg(feature = "webview")]
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::{Alignment, LayoutFrame, LayoutSize};
use arkit_core::{advanced, Horizontal, Length, Padding, Size, Vertical};
#[cfg(feature = "webview")]
use napi_ohos::{
    bindgen_prelude::{FnArgs, Function, JsObjectValue, Object, ObjectRef},
    Either,
};
use ohos_arkui_binding::api::attribute_option::{
    NodeAdapter, NodeAdapterEvent, ProgressLinearStyleOption, TextLayoutManager,
};
use ohos_arkui_binding::api::node_custom_event::NodeCustomEvent;
use ohos_arkui_binding::common::attribute::{ArkUINodeAttributeItem, ArkUINodeAttributeNumber};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
#[cfg(feature = "webview")]
use ohos_arkui_binding::common::node::ArkUINodeRaw;
use ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUIEvent, ArkUIGesture,
};
use ohos_arkui_binding::component::built_in_component::{
    Button, CalendarPicker, Checkbox, Column, DatePicker, FlowItem, Grid, GridItem, Image, List,
    ListItem, ListItemGroup, Progress, Radio, Refresh, Row, Scroll, Slider, Stack, Swiper, Text,
    TextArea, TextInput, Toggle, WaterFlow,
};
use ohos_arkui_binding::event::inner_event::Event as ArkEvent;
use ohos_arkui_binding::gesture::gesture_data::GestureEventData;
use ohos_arkui_binding::gesture::inner_gesture::Gesture;
use ohos_arkui_binding::types::advanced::{
    FontWeight, HorizontalAlignment, NodeAdapterEventType, NodeCustomEventType, ShadowStyle,
    VerticalAlignment,
};
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use ohos_arkui_binding::types::event::NodeEventType;
use ohos_arkui_binding::types::gesture_event::GestureEventAction;
use ohos_arkui_binding::types::text_alignment::TextAlignment;
#[cfg(feature = "webview")]
use openharmony_ability::{get_helper, get_main_thread_env, WebViewInitData};
#[cfg(feature = "webview")]
pub use openharmony_ability::{DownloadStartResult, WebViewStyle, Webview};

#[path = "render_impl/types.rs"]
mod types;
pub use types::*;
#[allow(unused_imports)]
use types::*;

#[path = "render_impl/runtime.rs"]
mod runtime;
use runtime::*;

#[path = "render_impl/mounted.rs"]
mod mounted;
pub use mounted::MountedNode;
use mounted::*;

#[path = "render_impl/virtual_adapter.rs"]
mod virtual_adapter;
use virtual_adapter::*;

#[path = "render_impl/node_core.rs"]
mod node_core;
pub use node_core::Node;

#[path = "render_impl/node_effects.rs"]
mod node_effects;
#[path = "render_impl/node_events.rs"]
mod node_events;
#[path = "render_impl/node_layout.rs"]
mod node_layout;
#[path = "render_impl/node_specialized.rs"]
mod node_specialized;
#[path = "render_impl/node_style.rs"]
mod node_style;
#[path = "render_impl/node_widget.rs"]
mod node_widget;

#[path = "render_impl/component.rs"]
mod component;
#[cfg(feature = "webview")]
pub use component::WebViewElement;
pub use component::{
    ButtonElement, CalendarPickerElement, CheckboxElement, ColumnElement, Component,
    ContainerElement, DatePickerElement, FlowItemElement, GridElement, GridItemElement,
    ImageElement, ListElement, ListItemElement, ListItemGroupElement, ProgressElement,
    RadioElement, RefreshElement, RowElement, ScrollElement, SliderElement, StackElement,
    SwiperElement, TextAreaElement, TextElement, TextInputElement, ToggleElement, WaterFlowElement,
};

#[path = "render_impl/factories.rs"]
mod factories;
pub use factories::*;

#[path = "render_impl/layout_observer.rs"]
mod layout_observer;
pub use layout_observer::*;

#[cfg(feature = "webview")]
#[path = "render_impl/webview_controller.rs"]
mod webview_controller;
#[cfg(feature = "webview")]
use webview_controller::*;
#[cfg(feature = "webview")]
pub use webview_controller::{web_view, web_view_component, WebViewController};

#[cfg(feature = "webview")]
#[path = "render_impl/webview_native.rs"]
mod webview_native;
#[cfg(feature = "webview")]
use webview_native::*;

#[cfg(feature = "webview")]
#[path = "render_impl/webview_sync.rs"]
mod webview_sync;
#[cfg(feature = "webview")]
use webview_sync::*;

#[path = "render_impl/native_node.rs"]
mod native_node;
use native_node::*;

#[path = "render_impl/tree_compile.rs"]
mod tree_compile;
use tree_compile::*;

#[path = "render_impl/patch_helpers.rs"]
mod patch_helpers;
use patch_helpers::*;

#[path = "render_impl/mount.rs"]
mod mount;
use mount::*;
pub use mount::{mount, patch, realize_attached_mount};

#[path = "render_impl/reconcile.rs"]
mod reconcile;
use reconcile::*;

pub use ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem as AttributeValue;
pub use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType as Attribute;

#[cfg(test)]
#[path = "render_impl/tests.rs"]
mod tests;
