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
use ohos_arkui_binding::common::attribute::{
    ArkUINodeAttributeItem, ArkUINodeAttributeNumber, ArkUINodeCompositeAttributeItem,
};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
#[cfg(feature = "webview")]
use ohos_arkui_binding::common::node::ArkUINodeRaw;
use ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUIEvent, ArkUIGesture,
};
use ohos_arkui_binding::component::built_in_component::{
    Button, CalendarPicker, Checkbox, Column, DatePicker, Flex, FlowItem, Grid, GridItem, Image,
    List, ListItem, ListItemGroup, Progress, Radio, Refresh, Row, Scroll, Slider, Stack, Swiper,
    Text, TextArea, TextInput, Toggle, WaterFlow,
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

mod types;
pub use types::*;
#[allow(unused_imports)]
use types::*;

mod runtime;
use runtime::*;

mod mounted;
pub use mounted::MountedNode;
use mounted::*;

mod virtual_adapter;
use virtual_adapter::*;

mod node_core;
pub use node_core::Node;

mod node_effects;
mod node_events;
mod node_layout;
mod node_specialized;
mod node_style;

mod component;
#[cfg(feature = "webview")]
pub use component::WebViewElement;
pub use component::{
    ButtonElement, CalendarPickerElement, CheckboxElement, ColumnElement, Component,
    ContainerElement, DatePickerElement, FlexElement, FlowItemElement, GridElement,
    GridItemElement, ImageElement, ListElement, ListItemElement, ListItemGroupElement,
    ProgressElement, RadioElement, RefreshElement, RowElement, ScrollElement, SliderElement,
    StackElement, SwiperElement, TextAreaElement, TextElement, TextInputElement, ToggleElement,
    WaterFlowElement,
};

mod lazy;
pub use lazy::{lazy, Lazy};

mod factories;
pub use factories::*;

mod layout_observer;
pub use layout_observer::*;

#[cfg(feature = "webview")]
mod webview_controller;
#[cfg(feature = "webview")]
use webview_controller::*;
#[cfg(feature = "webview")]
pub use webview_controller::{web_view, web_view_component, WebViewController};

#[cfg(feature = "webview")]
mod webview_native;
#[cfg(feature = "webview")]
use webview_native::*;

#[cfg(feature = "webview")]
mod webview_sync;
#[cfg(feature = "webview")]
use webview_sync::*;

mod native_node;
use native_node::*;

mod patch_helpers;
use patch_helpers::*;

mod tree_compile;
use tree_compile::*;

pub use mounted::{mount, patch, realize_attached_mount};

pub use ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem as AttributeValue;
pub use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType as Attribute;

#[cfg(test)]
mod tests;
