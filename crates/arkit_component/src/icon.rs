//! Icon rendering, delegated to `arkit_icon` (real lucide SVG → ArkUI Image).

use arkit_prelude::*;

/// Render a lucide icon by name at the given size/color.
///
/// Delegates to [`arkit_icon::icon`], which rasterizes the embedded SVG to a
/// `DrawableDescriptor` and applies it as an ArkUI `Image` source after mount.
pub fn icon_placeholder(name: &str, size: f32, color: u32) -> Element {
    arkit_icon::icon(name, size, color)
}
