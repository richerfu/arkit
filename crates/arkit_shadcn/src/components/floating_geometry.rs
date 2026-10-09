//! Viewport-local placement without native queries.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloatingSide {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloatingAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Clone, Copy)]
pub(crate) struct AnchorRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
#[derive(Clone, Copy)]
pub(crate) struct PanelBounds {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

pub(crate) struct PanelPlacement {
    pub x: f32,
    pub y: f32,
}
impl PanelPlacement {
    pub fn resolve(
        anchor: AnchorRect,
        bounds: PanelBounds,
        size: (f32, f32),
        side: FloatingSide,
        align: FloatingAlign,
        offset: f32,
    ) -> Self {
        let width = size.0.min((bounds.right - bounds.left).max(1.0));
        let height = size.1.min((bounds.bottom - bounds.top).max(1.0));
        let fits = |side| match side {
            FloatingSide::Top => anchor.y - offset - height >= bounds.top,
            FloatingSide::Bottom => anchor.y + anchor.height + offset + height <= bounds.bottom,
            FloatingSide::Left => anchor.x - offset - width >= bounds.left,
            FloatingSide::Right => anchor.x + anchor.width + offset + width <= bounds.right,
        };
        let opposite = match side {
            FloatingSide::Top => FloatingSide::Bottom,
            FloatingSide::Bottom => FloatingSide::Top,
            FloatingSide::Left => FloatingSide::Right,
            FloatingSide::Right => FloatingSide::Left,
        };
        let side = if !fits(side) && fits(opposite) {
            opposite
        } else {
            side
        };
        let aligned = |origin, extent, panel| match align {
            FloatingAlign::Start => origin,
            FloatingAlign::Center => origin + (extent - panel) / 2.0,
            FloatingAlign::End => origin + extent - panel,
        };
        let x = match side {
            FloatingSide::Left => anchor.x - width - offset,
            FloatingSide::Right => anchor.x + anchor.width + offset,
            _ => aligned(anchor.x, anchor.width, width),
        };
        let y = match side {
            FloatingSide::Top => anchor.y - height - offset,
            FloatingSide::Bottom => anchor.y + anchor.height + offset,
            _ => aligned(anchor.y, anchor.height, height),
        };
        Self {
            x: x.clamp(bounds.left, (bounds.right - width).max(bounds.left)),
            y: y.clamp(bounds.top, (bounds.bottom - height).max(bounds.top)),
        }
    }
}
