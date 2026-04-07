use crate::{
    dispatch, ArkUINodeAttributeType, ButtonElement, CheckboxElement, ContainerElement,
    TextAreaElement, TextElement, TextInputElement, Theme,
};

const FLEX_ALIGN_START: i32 = 1;
const FLEX_ALIGN_CENTER: i32 = 2;
const FLEX_ALIGN_END: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Shrink,
    Fill,
    FillPortion(u16),
    Fixed(f32),
}

impl From<f32> for Length {
    fn from(value: f32) -> Self {
        Self::Fixed(value)
    }
}

impl From<f64> for Length {
    fn from(value: f64) -> Self {
        Self::Fixed(value as f32)
    }
}

impl From<i32> for Length {
    fn from(value: i32) -> Self {
        Self::Fixed(value as f32)
    }
}

impl From<u16> for Length {
    fn from(value: u16) -> Self {
        Self::Fixed(f32::from(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn to_vec(self) -> Vec<f32> {
        vec![self.top, self.right, self.bottom, self.left]
    }
}

impl From<f32> for Padding {
    fn from(value: f32) -> Self {
        Self::all(value)
    }
}

impl From<f64> for Padding {
    fn from(value: f64) -> Self {
        Self::all(value as f32)
    }
}

impl From<i32> for Padding {
    fn from(value: i32) -> Self {
        Self::all(value as f32)
    }
}

impl From<u16> for Padding {
    fn from(value: u16) -> Self {
        Self::all(f32::from(value))
    }
}

impl From<[f32; 2]> for Padding {
    fn from(value: [f32; 2]) -> Self {
        Self::symmetric(value[0], value[1])
    }
}

impl From<[f32; 4]> for Padding {
    fn from(value: [f32; 4]) -> Self {
        Self {
            top: value[0],
            right: value[1],
            bottom: value[2],
            left: value[3],
        }
    }
}

impl From<[i32; 2]> for Padding {
    fn from(value: [i32; 2]) -> Self {
        Self::symmetric(value[0] as f32, value[1] as f32)
    }
}

impl From<[i32; 4]> for Padding {
    fn from(value: [i32; 4]) -> Self {
        Self {
            top: value[0] as f32,
            right: value[1] as f32,
            bottom: value[2] as f32,
            left: value[3] as f32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Horizontal {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    Top,
    Center,
    Bottom,
}

pub trait ButtonMessageExt {
    fn on_press<Message>(self, message: Message) -> Self
    where
        Message: Clone + 'static;
}

impl ButtonMessageExt for ButtonElement {
    fn on_press<Message>(self, message: Message) -> Self
    where
        Message: Clone + 'static,
    {
        self.on_click(move || dispatch(message.clone()))
    }
}

pub trait TextInputMessageExt {
    fn on_input<Message>(self, handler: impl Fn(String) -> Message + 'static) -> Self
    where
        Message: 'static;

    fn on_submit_message<Message>(self, message: Message) -> Self
    where
        Message: Clone + 'static;
}

impl TextInputMessageExt for TextInputElement {
    fn on_input<Message>(self, handler: impl Fn(String) -> Message + 'static) -> Self
    where
        Message: 'static,
    {
        self.on_change(move |value| dispatch(handler(value)))
    }

    fn on_submit_message<Message>(self, message: Message) -> Self
    where
        Message: Clone + 'static,
    {
        self.on_submit(move |_| dispatch(message.clone()))
    }
}

pub trait TextAreaMessageExt {
    fn on_input<Message>(self, handler: impl Fn(String) -> Message + 'static) -> Self
    where
        Message: 'static;

    fn on_submit_message<Message>(self, message: Message) -> Self
    where
        Message: Clone + 'static;
}

impl TextAreaMessageExt for TextAreaElement {
    fn on_input<Message>(self, handler: impl Fn(String) -> Message + 'static) -> Self
    where
        Message: 'static,
    {
        self.on_change(move |value| dispatch(handler(value)))
    }

    fn on_submit_message<Message>(self, message: Message) -> Self
    where
        Message: Clone + 'static,
    {
        self.on_submit(move |_| dispatch(message.clone()))
    }
}

pub trait CheckboxMessageExt {
    fn on_toggle<Message>(self, handler: impl Fn(bool) -> Message + 'static) -> Self
    where
        Message: 'static;
}

impl CheckboxMessageExt for CheckboxElement {
    fn on_toggle<Message>(self, handler: impl Fn(bool) -> Message + 'static) -> Self
    where
        Message: 'static,
    {
        self.on_change(move |value| dispatch(handler(value)))
    }
}

pub trait ContainerAlignmentExt {
    fn align_x(self, alignment: Horizontal) -> Self;
    fn align_y(self, alignment: Vertical) -> Self;
    fn center_x(self) -> Self;
    fn center_y(self) -> Self;
}

impl ContainerAlignmentExt for ContainerElement {
    fn align_x(self, alignment: Horizontal) -> Self {
        match alignment {
            Horizontal::Left => self.align_items_start(),
            Horizontal::Center => self.align_items_center(),
            Horizontal::Right => self.align_items_end(),
        }
    }

    fn align_y(self, alignment: Vertical) -> Self {
        let justify = match alignment {
            Vertical::Top => FLEX_ALIGN_START,
            Vertical::Center => FLEX_ALIGN_CENTER,
            Vertical::Bottom => FLEX_ALIGN_END,
        };
        self.style(ArkUINodeAttributeType::ColumnJustifyContent, justify)
    }

    fn center_x(self) -> Self {
        self.align_x(Horizontal::Center)
    }

    fn center_y(self) -> Self {
        self.align_y(Vertical::Center)
    }
}

pub mod style {
    use super::*;

    pub mod button {
        use super::*;

        #[derive(Debug, Clone, Copy)]
        pub enum Status {
            Active,
            Disabled,
        }

        #[derive(Debug, Clone, Copy)]
        pub struct Style {
            pub background: Option<u32>,
            pub text_color: Option<u32>,
            pub border_radius: [f32; 4],
            pub border_width: [f32; 4],
            pub border_color: [u32; 4],
        }

        pub type Catalog = fn(&Theme, Status) -> Style;

        pub fn primary(_theme: &Theme, status: Status) -> Style {
            match status {
                Status::Active => Style {
                    background: Some(0xFF18181B),
                    text_color: Some(0xFFFAFAFA),
                    border_radius: [8.0, 8.0, 8.0, 8.0],
                    border_width: [0.0, 0.0, 0.0, 0.0],
                    border_color: [0x00000000; 4],
                },
                Status::Disabled => Style {
                    background: Some(0xFF71717A),
                    text_color: Some(0xFFE4E4E7),
                    border_radius: [8.0, 8.0, 8.0, 8.0],
                    border_width: [0.0, 0.0, 0.0, 0.0],
                    border_color: [0x00000000; 4],
                },
            }
        }

        pub fn secondary(_theme: &Theme, _status: Status) -> Style {
            Style {
                background: Some(0xFFF4F4F5),
                text_color: Some(0xFF18181B),
                border_radius: [8.0, 8.0, 8.0, 8.0],
                border_width: [0.0, 0.0, 0.0, 0.0],
                border_color: [0x00000000; 4],
            }
        }

        pub fn outline(_theme: &Theme, _status: Status) -> Style {
            Style {
                background: Some(0xFFFFFFFF),
                text_color: Some(0xFF18181B),
                border_radius: [8.0, 8.0, 8.0, 8.0],
                border_width: [1.0, 1.0, 1.0, 1.0],
                border_color: [0xFFE4E4E7; 4],
            }
        }

        pub fn ghost(_theme: &Theme, _status: Status) -> Style {
            Style {
                background: Some(0x00000000),
                text_color: Some(0xFF18181B),
                border_radius: [8.0, 8.0, 8.0, 8.0],
                border_width: [0.0, 0.0, 0.0, 0.0],
                border_color: [0x00000000; 4],
            }
        }

        pub fn destructive(_theme: &Theme, _status: Status) -> Style {
            Style {
                background: Some(0xFFDC2626),
                text_color: Some(0xFFFFFFFF),
                border_radius: [8.0, 8.0, 8.0, 8.0],
                border_width: [0.0, 0.0, 0.0, 0.0],
                border_color: [0x00000000; 4],
            }
        }
    }

    pub mod container {
        use super::*;

        #[derive(Debug, Clone, Copy)]
        pub struct Style {
            pub background: Option<u32>,
            pub border_radius: [f32; 4],
            pub border_width: [f32; 4],
            pub border_color: [u32; 4],
        }

        pub type Catalog = fn(&Theme) -> Style;

        pub fn transparent(_theme: &Theme) -> Style {
            Style {
                background: None,
                border_radius: [0.0, 0.0, 0.0, 0.0],
                border_width: [0.0, 0.0, 0.0, 0.0],
                border_color: [0x00000000; 4],
            }
        }

        pub fn rounded_box(_theme: &Theme) -> Style {
            Style {
                background: Some(0xFFFFFFFF),
                border_radius: [12.0, 12.0, 12.0, 12.0],
                border_width: [1.0, 1.0, 1.0, 1.0],
                border_color: [0xFFE4E4E7; 4],
            }
        }
    }

    pub mod text_input {
        use super::*;

        #[derive(Debug, Clone, Copy)]
        pub struct Style {
            pub background: Option<u32>,
            pub text_color: Option<u32>,
            pub border_radius: [f32; 4],
            pub border_width: [f32; 4],
            pub border_color: [u32; 4],
        }

        pub type Catalog = fn(&Theme) -> Style;

        pub fn default(_theme: &Theme) -> Style {
            Style {
                background: Some(0xFFFFFFFF),
                text_color: Some(0xFF09090B),
                border_radius: [8.0, 8.0, 8.0, 8.0],
                border_width: [1.0, 1.0, 1.0, 1.0],
                border_color: [0xFFE4E4E7; 4],
            }
        }
    }

    pub mod checkbox {
        use super::*;

        #[derive(Debug, Clone, Copy)]
        pub struct Style {
            pub background: Option<u32>,
            pub border_radius: [f32; 4],
            pub border_width: [f32; 4],
            pub border_color: [u32; 4],
        }

        pub type Catalog = fn(&Theme) -> Style;

        pub fn default(_theme: &Theme) -> Style {
            Style {
                background: Some(0xFFFFFFFF),
                border_radius: [4.0, 4.0, 4.0, 4.0],
                border_width: [1.0, 1.0, 1.0, 1.0],
                border_color: [0xFFD4D4D8; 4],
            }
        }
    }

    pub mod text {
        use super::*;

        #[derive(Debug, Clone, Copy)]
        pub struct Style {
            pub color: u32,
        }

        pub type Catalog = fn(&Theme) -> Style;

        pub fn default(_theme: &Theme) -> Style {
            Style { color: 0xFF09090B }
        }

        pub fn muted(_theme: &Theme) -> Style {
            Style { color: 0xFF71717A }
        }
    }
}

pub trait ButtonStyleExt {
    fn style(self, catalog: style::button::Catalog) -> Self;
}

impl ButtonStyleExt for ButtonElement {
    fn style(self, catalog: style::button::Catalog) -> Self {
        let style = catalog(&Theme, style::button::Status::Active);
        let mut button = self
            .style(
                ArkUINodeAttributeType::BorderRadius,
                style.border_radius.to_vec(),
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                style.border_width.to_vec(),
            )
            .style(
                ArkUINodeAttributeType::BorderColor,
                style.border_color.to_vec(),
            );

        if let Some(background) = style.background {
            button = button.background_color(background);
        }

        if let Some(text_color) = style.text_color {
            button = button
                .style(ArkUINodeAttributeType::FontColor, text_color)
                .patch_attr(ArkUINodeAttributeType::FontColor, text_color);
        }

        button
    }
}

pub trait ContainerStyleExt {
    fn style(self, catalog: style::container::Catalog) -> Self;
}

impl ContainerStyleExt for ContainerElement {
    fn style(self, catalog: style::container::Catalog) -> Self {
        let style = catalog(&Theme);
        let mut container = self
            .style(
                ArkUINodeAttributeType::BorderRadius,
                style.border_radius.to_vec(),
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                style.border_width.to_vec(),
            )
            .style(
                ArkUINodeAttributeType::BorderColor,
                style.border_color.to_vec(),
            );

        if let Some(background) = style.background {
            container = container.background_color(background);
        }

        container
    }
}

pub trait TextInputStyleExt {
    fn style(self, catalog: style::text_input::Catalog) -> Self;
}

impl TextInputStyleExt for TextInputElement {
    fn style(self, catalog: style::text_input::Catalog) -> Self {
        let style = catalog(&Theme);
        let mut input = self
            .style(
                ArkUINodeAttributeType::BorderRadius,
                style.border_radius.to_vec(),
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                style.border_width.to_vec(),
            )
            .style(
                ArkUINodeAttributeType::BorderColor,
                style.border_color.to_vec(),
            );

        if let Some(background) = style.background {
            input = input.background_color(background);
        }

        if let Some(text_color) = style.text_color {
            input = input.style(ArkUINodeAttributeType::ForegroundColor, text_color);
        }

        input
    }
}

pub trait CheckboxStyleExt {
    fn style(self, catalog: style::checkbox::Catalog) -> Self;
}

impl CheckboxStyleExt for CheckboxElement {
    fn style(self, catalog: style::checkbox::Catalog) -> Self {
        let style = catalog(&Theme);
        let mut checkbox = self
            .style(
                ArkUINodeAttributeType::BorderRadius,
                style.border_radius.to_vec(),
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                style.border_width.to_vec(),
            )
            .style(
                ArkUINodeAttributeType::BorderColor,
                style.border_color.to_vec(),
            );

        if let Some(background) = style.background {
            checkbox = checkbox.background_color(background);
        }

        checkbox
    }
}

pub trait TextStyleExt {
    fn style(self, catalog: style::text::Catalog) -> Self;
}

impl TextStyleExt for TextElement {
    fn style(self, catalog: style::text::Catalog) -> Self {
        let style = catalog(&Theme);
        self.style(ArkUINodeAttributeType::FontColor, style.color)
    }
}
