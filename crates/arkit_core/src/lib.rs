use std::any::{Any, TypeId};

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
}

impl From<f32> for Padding {
    fn from(value: f32) -> Self {
        Self::all(value)
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

pub mod theme {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Palette {
        pub background: u32,
        pub foreground: u32,
        pub primary: u32,
        pub success: u32,
        pub danger: u32,
        pub muted: u32,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Style {
        pub background: u32,
        pub text_color: u32,
    }

    pub trait Base: Clone + 'static {
        fn palette(&self) -> Palette;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl theme::Base for Theme {
    fn palette(&self) -> theme::Palette {
        match self {
            Self::Light => theme::Palette {
                background: 0xFFFFFFFF,
                foreground: 0xFF111827,
                primary: 0xFF18181B,
                success: 0xFF16A34A,
                danger: 0xFFDC2626,
                muted: 0xFFF3F4F6,
            },
            Self::Dark => theme::Palette {
                background: 0xFF09090B,
                foreground: 0xFFF5F5F5,
                primary: 0xFFE5E7EB,
                success: 0xFF22C55E,
                danger: 0xFFEF4444,
                muted: 0xFF18181B,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub default_text_size: f32,
    pub antialiasing: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_text_size: 16.0,
            antialiasing: true,
        }
    }
}

pub mod window {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Id(pub u64);

    impl Id {
        pub const MAIN: Self = Self(0);
    }

    #[derive(Debug, Clone)]
    pub struct Settings {
        pub title: String,
    }

    impl Default for Settings {
        fn default() -> Self {
            Self {
                title: String::from("arkit"),
            }
        }
    }
}

pub mod mouse {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum Interaction {
        #[default]
        Idle,
        Pointer,
    }

    #[derive(Debug, Clone, Copy, Default)]
    pub struct Cursor;
}

pub mod event {
    #[derive(Debug, Clone, Copy, Default)]
    pub enum Event {
        #[default]
        None,
    }
}

pub mod clipboard {
    pub trait Clipboard {
        fn read(&self) -> Option<String> {
            None
        }

        fn write(&mut self, _contents: String) {}
    }

    #[derive(Default)]
    pub struct Null;

    impl Clipboard for Null {}
}

pub mod layout {
    use crate::Size;

    #[derive(Debug, Clone, Copy, Default)]
    pub struct Node {
        pub size: Size<f32>,
    }

    impl Node {
        pub fn new(size: Size<f32>) -> Self {
            Self { size }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Layout<'a> {
        node: &'a Node,
    }

    impl<'a> Layout<'a> {
        pub fn new(node: &'a Node) -> Self {
            Self { node }
        }

        pub fn size(&self) -> Size<f32> {
            self.node.size
        }
    }
}

pub mod advanced {
    use super::{clipboard, event, layout, mouse, theme, Element, Length, Size, Theme};
    use std::any::{Any, TypeId};

    pub mod widget {
        use std::any::{Any, TypeId};

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct Tag(TypeId);

        impl Tag {
            pub fn of<T: 'static + ?Sized>() -> Self {
                Self(TypeId::of::<T>())
            }
        }

        pub struct State {
            inner: Option<Box<dyn Any>>,
        }

        impl State {
            pub fn none() -> Self {
                Self { inner: None }
            }

            pub fn new(value: Box<dyn Any>) -> Self {
                Self { inner: Some(value) }
            }

            pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
                self.inner.as_mut()?.downcast_mut::<T>()
            }
        }

        pub struct Tree {
            tag: Tag,
            state: State,
            children: Vec<Tree>,
        }

        impl Tree {
            pub fn empty() -> Self {
                Self {
                    tag: Tag::of::<()>(),
                    state: State::none(),
                    children: Vec::new(),
                }
            }

            pub fn new<T: 'static>() -> Self {
                Self {
                    tag: Tag::of::<T>(),
                    state: State::none(),
                    children: Vec::new(),
                }
            }

            pub fn tag(&self) -> Tag {
                self.tag
            }

            pub fn state(&mut self) -> &mut State {
                &mut self.state
            }

            pub fn children(&self) -> &[Tree] {
                &self.children
            }

            pub fn replace_children(&mut self, children: Vec<Tree>) {
                self.children = children;
            }

            pub fn set_tag(&mut self, tag: Tag) {
                self.tag = tag;
            }
        }
    }

    pub trait Widget<Message, AppTheme = Theme, Renderer = ()> {
        fn tag(&self) -> widget::Tag
        where
            Self: 'static,
        {
            widget::Tag::of::<Self>()
        }

        fn state(&self) -> widget::State {
            widget::State::none()
        }

        fn children(&self) -> Vec<widget::Tree> {
            Vec::new()
        }

        fn diff(&self, tree: &mut widget::Tree)
        where
            Self: 'static,
        {
            tree.set_tag(self.tag());
        }

        fn size_hint(&self) -> Size<Length> {
            Size::new(Length::Shrink, Length::Shrink)
        }

        fn layout(&self) -> layout::Node {
            layout::Node::default()
        }

        fn update(
            &mut self,
            _tree: &mut widget::Tree,
            _event: &event::Event,
            _layout: layout::Layout<'_>,
            _cursor: mouse::Cursor,
            _clipboard: &mut dyn clipboard::Clipboard,
            _shell: &mut Shell<'_, Message>,
        ) {
        }

        fn overlay<'a>(
            &'a mut self,
            _tree: &'a mut widget::Tree,
            _layout: layout::Layout<'a>,
            _renderer: &Renderer,
        ) -> Option<Element<'a, Message, AppTheme, Renderer>> {
            None
        }

        #[doc(hidden)]
        fn into_any(self: Box<Self>) -> Box<dyn Any>;
    }

    pub struct Shell<'a, Message> {
        messages: &'a mut Vec<Message>,
        event_captured: bool,
        redraw_requested: bool,
        layout_invalid: bool,
    }

    impl<'a, Message> Shell<'a, Message> {
        pub fn new(messages: &'a mut Vec<Message>) -> Self {
            Self {
                messages,
                event_captured: false,
                redraw_requested: false,
                layout_invalid: false,
            }
        }

        pub fn publish(&mut self, message: Message) {
            self.messages.push(message);
        }

        pub fn capture_event(&mut self) {
            self.event_captured = true;
        }

        pub fn request_redraw(&mut self) {
            self.redraw_requested = true;
        }

        pub fn invalidate_layout(&mut self) {
            self.layout_invalid = true;
        }

        pub fn is_event_captured(&self) -> bool {
            self.event_captured
        }

        pub fn is_redraw_requested(&self) -> bool {
            self.redraw_requested
        }

        pub fn is_layout_invalid(&self) -> bool {
            self.layout_invalid
        }
    }

    pub fn default_style(theme: &impl theme::Base) -> theme::Style {
        let palette = theme.palette();
        theme::Style {
            background: palette.background,
            text_color: palette.foreground,
        }
    }
}

pub struct Element<'a, Message, AppTheme = Theme, Renderer = ()> {
    widget: Box<dyn advanced::Widget<Message, AppTheme, Renderer> + 'a>,
}

impl<'a, Message, AppTheme, Renderer> Element<'a, Message, AppTheme, Renderer> {
    pub fn new(widget: impl advanced::Widget<Message, AppTheme, Renderer> + 'a) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }

    pub fn as_widget(&self) -> &(dyn advanced::Widget<Message, AppTheme, Renderer> + 'a) {
        self.widget.as_ref()
    }

    pub fn as_widget_mut(
        &mut self,
    ) -> &mut (dyn advanced::Widget<Message, AppTheme, Renderer> + 'a) {
        self.widget.as_mut()
    }

    #[doc(hidden)]
    pub fn into_widget(self) -> Box<dyn advanced::Widget<Message, AppTheme, Renderer> + 'a> {
        self.widget
    }
}
