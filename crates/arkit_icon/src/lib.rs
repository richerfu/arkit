mod embed;
mod icon;

pub use embed::{has_icon, icon_names};
pub use icon::{
    icon, try_icon, IconElement, IconError, DEFAULT_ICON_COLOR, DEFAULT_ICON_SIZE,
    DEFAULT_STROKE_WIDTH,
};
