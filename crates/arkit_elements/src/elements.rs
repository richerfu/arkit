//! ArkUI element descriptors for `rsx!`.
//!
//! Each element is a module exposing `TAG_NAME`, `NAME_SPACE`, and `pub const`
//! attribute descriptors. The glob re-export at the crate root
//! (`pub use elements::*;`) makes both `dioxus_elements::column::font_size`
//! and `dioxus_elements::elements::column::font_size` resolve to the same item,
//! which is what `rsx!` emits.

// Attribute descriptors are lowercase to match the rsx! attribute names; this
// mirrors dioxus-html's convention.
#![allow(non_upper_case_globals)]

macro_rules! define_accessibility_attributes {
    () => {
        /// Text announced by accessibility services for this node.
        pub const accessibility_text: $crate::AttributeDescription =
            ("accessibility_text", None, false);
        /// Additional accessibility hint or description for this node.
        pub const accessibility_description: $crate::AttributeDescription =
            ("accessibility_description", None, false);
        /// Whether this node and its descendants form one accessibility target.
        pub const accessibility_group: $crate::AttributeDescription =
            ("accessibility_group", None, false);
        /// Accessibility exposure mode: `auto`, `enabled`, `disabled`, or
        /// `disabled_for_descendants`.
        pub const accessibility_mode: $crate::AttributeDescription =
            ("accessibility_mode", None, false);
        /// Semantic role. Accepts ArkUI role names such as `button`,
        /// `checkbox`, `radio`, `switch`, `slider`, `progress`, and `text`.
        pub const accessibility_role: $crate::AttributeDescription =
            ("accessibility_role", None, false);
        /// Supported action names separated by `|`, for example
        /// `click|long_click`, or the equivalent bit mask.
        pub const accessibility_actions: $crate::AttributeDescription =
            ("accessibility_actions", None, false);
        /// Explicit disabled state exposed to accessibility services.
        pub const accessibility_disabled: $crate::AttributeDescription =
            ("accessibility_disabled", None, false);
        /// Explicit selected state exposed to accessibility services.
        pub const accessibility_selected: $crate::AttributeDescription =
            ("accessibility_selected", None, false);
        /// Explicit checked state exposed to accessibility services.
        pub const accessibility_checked: $crate::AttributeDescription =
            ("accessibility_checked", None, false);
        /// Integer minimum value for range controls.
        pub const accessibility_value_min: $crate::AttributeDescription =
            ("accessibility_value_min", None, false);
        /// Integer maximum value for range controls.
        pub const accessibility_value_max: $crate::AttributeDescription =
            ("accessibility_value_max", None, false);
        /// Integer current value for range controls.
        pub const accessibility_value_current: $crate::AttributeDescription =
            ("accessibility_value_current", None, false);
        /// API-18 range minimum value.
        pub const accessibility_value_range_min: $crate::AttributeDescription =
            ("accessibility_value_range_min", None, false);
        /// API-18 range maximum value.
        pub const accessibility_value_range_max: $crate::AttributeDescription =
            ("accessibility_value_range_max", None, false);
        /// API-18 range current value.
        pub const accessibility_value_range_current: $crate::AttributeDescription =
            ("accessibility_value_range_current", None, false);
        /// Human-readable value announced for a range or stateful control.
        pub const accessibility_value_text: $crate::AttributeDescription =
            ("accessibility_value_text", None, false);
    };
}

macro_rules! define_accessibility_extension_methods {
    () => {
        fn accessibility_text(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_text", None, value, false)
        }

        fn accessibility_description(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_description", None, value, false)
        }

        fn accessibility_group(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_group", None, value, false)
        }

        fn accessibility_mode(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_mode", None, value, false)
        }

        fn accessibility_role(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_role", None, value, false)
        }

        fn accessibility_actions(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_actions", None, value, false)
        }

        fn accessibility_disabled(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_disabled", None, value, false)
        }

        fn accessibility_selected(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_selected", None, value, false)
        }

        fn accessibility_checked(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_checked", None, value, false)
        }

        fn accessibility_value_min(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_value_min", None, value, false)
        }

        fn accessibility_value_max(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_value_max", None, value, false)
        }

        fn accessibility_value_current(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_value_current", None, value, false)
        }

        fn accessibility_value_range_min(
            self,
            value: impl dioxus_core::IntoAttributeValue,
        ) -> Self {
            self.push_attribute("accessibility_value_range_min", None, value, false)
        }

        fn accessibility_value_range_max(
            self,
            value: impl dioxus_core::IntoAttributeValue,
        ) -> Self {
            self.push_attribute("accessibility_value_range_max", None, value, false)
        }

        fn accessibility_value_range_current(
            self,
            value: impl dioxus_core::IntoAttributeValue,
        ) -> Self {
            self.push_attribute("accessibility_value_range_current", None, value, false)
        }

        fn accessibility_value_text(self, value: impl dioxus_core::IntoAttributeValue) -> Self {
            self.push_attribute("accessibility_value_text", None, value, false)
        }
    };
}

macro_rules! define_element {
    (
        $(#[$meta:meta])*
        $name:ident => $tag:literal, $extension:ident {
            $( $attr:ident ),* $(,)?
        }
    ) => {
        define_element! {
            $(#[$meta])*
            $name => $tag {
                $( $attr ),*
            }
        }

        #[doc(hidden)]
        pub trait $extension: dioxus_core::HasAttributes + Sized {
            define_accessibility_extension_methods!();

            $(
                fn $attr(
                    self,
                    value: impl dioxus_core::IntoAttributeValue,
                ) -> Self {
                    let description = $name::$attr;
                    self.push_attribute(
                        description.0,
                        description.1,
                        value,
                        description.2,
                    )
                }
            )*
        }
    };
    (
        $(#[$meta:meta])*
        $name:ident => $tag:literal {
            $( $attr:ident ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        pub mod $name {
            /// The canonical lowercase ArkUI tag used as the Dioxus element name.
            pub const TAG_NAME: &'static str = $tag;
            /// Namespace for all ArkUI elements.
            pub const NAME_SPACE: Option<&'static str> = Some("arkui");
            /// Bind this exact logical element to a `NativeElementRef`.
            pub const native_ref: $crate::AttributeDescription = ("native_ref", None, false);

            define_accessibility_attributes!();

            $(
                pub const $attr: $crate::AttributeDescription = (stringify!($attr), None, false);
            )*
        }
    };
}

// Shared layout/box attribute list used by most containers and leaf nodes.
// (Inline since `define_element!` takes a literal comma-list.)

define_element! {
    /// Logical portal. Its Dioxus ancestry stays at the declaration site while
    /// the renderer projects its native stack into the selected root layer.
    portal => "portal" {
        portal_layer, width, height, alignment, hit_test_behavior, z_index,
    }
}

define_element! {
    /// Native media surface (ArkUI `XComponent`, surface mode).
    xcomponent => "xcomponent" {
        background_color, margin, margin_top, margin_bottom, margin_left,
        margin_right, margin_x, margin_y, margin_horizontal, margin_vertical, width, height,
        opacity, border_radius, border_width, border_color, visibility,
        enabled, clip, hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Column layout container (ArkUI `Column`).
    column => "column", ColumnExtension {
        font_size, font_color, font_weight, foreground_color, background_color, padding, padding_top, padding_right, padding_bottom, padding_left, padding_x, padding_y, padding_horizontal, padding_vertical, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, max_width_constraint, constraint_size, min_width, max_width, min_height, max_height, align_items, justify_content,
        align_self, item_alignment, layout_weight, opacity, border_radius, border_width,
        border_color, border_style, shadow, visibility, enabled, clip, focusable,
        focus_on_touch, hit_test_behavior, alignment, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Row layout container (ArkUI `Row`).
    row => "row", RowExtension {
        font_size, font_color, font_weight, foreground_color, background_color, padding, padding_top, padding_right, padding_bottom, padding_left, padding_x, padding_y, padding_horizontal, padding_vertical, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, max_width_constraint, constraint_size, min_width, max_width, min_height, max_height, align_items, justify_content,
        align_self, item_alignment, layout_weight, opacity, border_radius, border_width,
        border_color, border_style, shadow, visibility, enabled, clip, focusable,
        focus_on_touch, hit_test_behavior, alignment, aspect_ratio, position, z_index,
    }
}

/// Attribute-extension traits used by Dioxus component props.
#[doc(hidden)]
pub mod extensions {
    pub use super::{ColumnExtension, RowExtension};
}

define_element! {
    /// Custom drawing surface (ArkUI `Custom`).
    custom => "custom" {
        background_color, padding, margin, margin_top, margin_bottom, margin_left,
        margin_right, margin_x, margin_y, margin_horizontal, margin_vertical, width, height, opacity,
        border_radius, border_width, border_color, visibility, enabled, clip,
        hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Stack layout container (ArkUI `Stack`).
    stack => "stack" {
        background_color, padding, padding_top, padding_right, padding_bottom, padding_left, padding_x, padding_y, padding_horizontal, padding_vertical, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, max_width_constraint, constraint_size, min_width, max_width, min_height, max_height, alignment, opacity,
        border_radius, border_width, border_color, border_style, shadow, visibility,
        enabled, clip, focusable, focus_on_touch, hit_test_behavior, aspect_ratio,
        position, z_index,
    }
}

define_element! {
    /// Flex layout container (ArkUI `Flex`).
    flex => "flex" {
        background_color, padding, padding_top, padding_right, padding_bottom, padding_left, padding_x, padding_y, padding_horizontal, padding_vertical, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, max_width_constraint, constraint_size, min_width, max_width, min_height, max_height, align_items, justify_content,
        align_self, item_alignment, flex_direction, flex_wrap, flex_align_content,
        layout_weight, opacity, border_radius, border_width, border_color, border_style,
        shadow, visibility, enabled, clip, focusable, focus_on_touch,
        hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Text element (ArkUI `Text`).
    text => "text" {
        font_size, font_color, font_weight, font_family, font_style, line_height,
        text_align, text_letter_spacing, text_decoration, text_overflow, max_lines,
        content, background_color, padding, padding_top, padding_right, padding_bottom, padding_left, padding_x, padding_y, padding_horizontal, padding_vertical, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, max_width_constraint, constraint_size, min_width, max_width, min_height, max_height, opacity, border_radius,
        border_width, border_color, border_style, shadow, visibility, enabled, clip,
        focusable, focus_on_touch, hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Button element (ArkUI `Button`).
    button => "button" {
        font_size, font_color, font_weight, foreground_color, background_color, padding, padding_top, padding_right, padding_bottom, padding_left, padding_x, padding_y, padding_horizontal, padding_vertical, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, max_width_constraint, constraint_size, min_width, max_width, min_height, max_height, button_type, label,
        opacity, border_radius, border_width, border_color, border_style, shadow,
        visibility, enabled, clip, focusable, focus_on_touch, hit_test_behavior,
        align_self, item_alignment, alignment, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Image element (ArkUI `Image`).
    image => "image" {
        src, object_fit, background_color, padding, padding_top, padding_right, padding_bottom, padding_left, padding_x, padding_y, padding_horizontal, padding_vertical, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, opacity, border_radius,
        border_width, border_color, border_style, shadow, visibility, enabled, clip,
        focusable, focus_on_touch, hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Checkbox element (ArkUI `Checkbox`).
    checkbox => "checkbox" {
        checked, checkbox_select_color, background_color, padding, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, opacity, border_radius,
        border_width, border_color, visibility, enabled, clip, focusable, focus_on_touch, hit_test_behavior,
        aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Toggle element (ArkUI `Toggle`).
    toggle => "toggle" {
        checked, toggle_selected_color, toggle_unselected_color,
        toggle_switch_point_color, background_color, padding, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, opacity, border_radius,
        border_width, border_color, visibility, enabled, clip, focusable, focus_on_touch, hit_test_behavior,
        aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Radio element (ArkUI `Radio`).
    radio => "radio" {
        checked, radio_value, value, background_color, padding, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, opacity, border_radius,
        border_width, border_color, visibility, enabled, clip, focusable, focus_on_touch, hit_test_behavior,
        aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Slider element (ArkUI `Slider`).
    slider => "slider" {
        value, slider_value, slider_min, slider_max, slider_step, background_color,
        padding, margin, margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, opacity, border_radius,
        border_width, border_color, visibility, enabled, clip, focusable, focus_on_touch, hit_test_behavior,
        block_color, selected_color, track_color, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Progress element (ArkUI `Progress`).
    progress => "progress" {
        value, progress_value, progress_total, progress_color, progress_type,
        background_color, padding, margin, margin_top, margin_bottom, margin_left,
        margin_right, margin_x, margin_y, margin_horizontal, margin_vertical, width, height, opacity,
        border_radius, border_width, border_color, visibility, enabled, clip,
        hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Indeterminate loading indicator (ArkUI `LoadingProgress`).
    loadingprogress => "loadingprogress" {
        loading_progress_color, loading_progress_enable_loading,
        background_color, padding, margin, margin_top, margin_bottom, margin_left,
        margin_right, margin_x, margin_y, margin_horizontal, margin_vertical, width, height, opacity,
        border_radius, border_width, border_color, visibility, enabled, clip,
        hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Scroll container (ArkUI `Scroll`).
    ///
    /// `scroll_bar`: scrollbar visibility — `false`/`"off"`/`0` hide, `"auto"`/`1`
    /// auto-fade, `true`/`"on"`/`2` always show.
    ///
    /// `scroll_offset` is a one-shot `"x,y[,duration,...]"` scroll command.
    /// It is consumed after native attachment and is not declarative state.
    scroll => "scroll" {
        scroll_bar, scroll_direction, scroll_enabled, scroll_edge_effect, scroll_offset, background_color, padding, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, layout_weight, opacity, border_radius,
        border_width, border_color, visibility, enabled, clip, hit_test_behavior,
        alignment, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Swiper container (ArkUI `Swiper`).
    swiper => "swiper" {
        swiper_index, swiper_swipe_to_index, swiper_loop, swiper_auto_play, swiper_show_indicator,
        swiper_disable_swipe, swiper_cached_count, swiper_display_count,
        swiper_vertical, swiper_interval, swiper_duration, swiper_curve, swiper_item_space,
        background_color, padding, margin, margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, opacity, border_radius,
        border_width, border_color, border_style, shadow, visibility, enabled, clip, hit_test_behavior,
        aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Grid container (ArkUI `Grid`).
    ///
    /// `scroll_bar`: same modes as [`scroll`] — `false`/`"off"`/`0`, `"auto"`/`1`, `true`/`"on"`/`2`.
    grid => "grid" {
        virtual_source,
        scroll_bar, grid_column_template, grid_row_template, grid_column_gap, grid_row_gap,
        grid_cached_count, background_color, padding, margin, margin_top,
        margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, layout_weight, opacity, border_radius, border_width,
        border_color, visibility, enabled, clip, hit_test_behavior, aspect_ratio,
        position, z_index,
    }
}

define_element! {
    /// Grid item (ArkUI `GridItem`).
    griditem => "griditem" {
        background_color, padding, margin, margin_top, margin_bottom, margin_left,
        margin_right, margin_x, margin_y, margin_horizontal, margin_vertical, width, height, opacity,
        border_radius, border_width, border_color, visibility, enabled, clip,
        hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// List container (ArkUI `List`).
    ///
    /// `scroll_bar`: same modes as [`scroll`] — `false`/`"off"`/`0`, `"auto"`/`1`, `true`/`"on"`/`2`.
    ///
    /// `scroll_to_index` is a one-shot `"index[,smooth,align]"` jump command.
    /// It is consumed after native attachment and is not declarative state.
    list => "list" {
        virtual_source,
        scroll_bar, list_cached_count, list_sticky, scroll_to_index, background_color, padding, margin, margin_top,
        margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, layout_weight, opacity, border_radius, border_width,
        border_color, visibility, enabled, clip, hit_test_behavior, aspect_ratio,
        position, z_index,
    }
}

define_element! {
    /// List item (ArkUI `ListItem`).
    listitem => "listitem" {
        background_color, padding, margin, margin_top, margin_bottom, margin_left,
        margin_right, margin_x, margin_y, margin_horizontal, margin_vertical, width, height, opacity,
        border_radius, border_width, border_color, visibility, enabled, clip,
        hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Water flow container (ArkUI `WaterFlow`).
    ///
    /// `scroll_bar`: same modes as [`scroll`] — `false`/`"off"`/`0`, `"auto"`/`1`, `true`/`"on"`/`2`.
    waterflow => "waterflow" {
        virtual_source,
        scroll_bar, water_flow_column_template, water_flow_row_template, water_flow_column_gap,
        water_flow_row_gap, water_flow_cached_count, background_color, padding, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, layout_weight, opacity, border_radius,
        border_width, border_color, visibility, enabled, clip, hit_test_behavior,
        aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Flow item (ArkUI `FlowItem`).
    flowitem => "flowitem" {
        background_color, padding, margin, margin_top, margin_bottom, margin_left,
        margin_right, margin_x, margin_y, margin_horizontal, margin_vertical, width, height, opacity,
        border_radius, border_width, border_color, visibility, enabled, clip,
        hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Refresh container (ArkUI `Refresh`).
    refresh => "refresh" {
        refresh_state, refreshing, refresh_offset, refresh_pull_to_refresh,
        background_color, padding, margin, margin_top, margin_bottom, margin_left,
        margin_right, margin_x, margin_y, margin_horizontal, margin_vertical, width, height, opacity,
        border_radius, border_width, border_color, visibility, enabled, clip,
        hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Calendar picker (ArkUI `CalendarPicker`).
    calendar => "calendarpicker" {
        background_color, padding, margin,
        margin_top, margin_bottom, margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, opacity, border_radius, border_width,
        border_color, visibility, enabled, clip, hit_test_behavior, aspect_ratio,
        position, z_index,
    }
}

define_element! {
    /// Date picker (ArkUI `DatePicker`).
    datepicker => "datepicker" {
        background_color, padding, margin, margin_top, margin_bottom,
        margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical, width, height,
        opacity, border_radius, border_width, border_color, visibility, enabled, clip,
        hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Text input (ArkUI `TextInput`).
    textinput => "textinput" {
        value, placeholder, placeholder_color, caret_color, input_type, input_filter, max_length,
        show_password_icon,
        font_size, font_color,
        font_weight, font_family, font_style, line_height, text_align, background_color, padding,
        padding_top, padding_right, padding_bottom, padding_left, padding_x, padding_y, padding_horizontal, padding_vertical, margin, margin_top, margin_bottom,
        margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, opacity, border_radius, border_width,
        border_color, border_style, visibility, enabled, clip, focusable, focus_on_touch,
        focused, focus_status,
        hit_test_behavior, aspect_ratio, position, z_index,
    }
}

define_element! {
    /// Text area (ArkUI `TextArea`).
    textarea => "textarea" {
        value, placeholder, placeholder_color, caret_color, font_size, font_color,
        font_weight, font_family, font_style, line_height, text_align, background_color, padding,
        padding_top, padding_right, padding_bottom, padding_left, padding_x, padding_y, padding_horizontal, padding_vertical, margin, margin_top, margin_bottom,
        margin_left, margin_right, margin_x, margin_y, margin_horizontal, margin_vertical,
        width, height, opacity, border_radius, border_width,
        border_color, border_style, visibility, enabled, clip, focusable, focus_on_touch,
        focused, focus_status,
        hit_test_behavior, aspect_ratio, position, z_index,
    }
}

/// rust-analyzer completion helper. Mirrors `dioxus_html`'s
/// `CompleteWithBraces` enum so rsx! completion hints resolve.
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub mod completions {
    pub enum CompleteWithBraces {
        column {},
        custom {},
        row {},
        stack {},
        flex {},
        text {},
        button {},
        image {},
        checkbox {},
        toggle {},
        radio {},
        slider {},
        progress {},
        loadingprogress {},
        scroll {},
        swiper {},
        grid {},
        griditem {},
        list {},
        listitem {},
        waterflow {},
        flowitem {},
        refresh {},
        calendar {},
        datepicker {},
        textinput {},
        textarea {},
        xcomponent {},
        portal {},
    }
}
