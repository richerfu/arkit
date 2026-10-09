// Use the production placement algorithm independently of ArkUI.
#[path = "../src/components/floating_geometry.rs"]
mod geometry;
use geometry::{AnchorRect, FloatingAlign, FloatingSide, PanelBounds, PanelPlacement};

fn bounds() -> PanelBounds {
    PanelBounds {
        left: 8.0,
        top: 8.0,
        right: 792.0,
        bottom: 592.0,
    }
}
fn anchor() -> AnchorRect {
    AnchorRect {
        x: 300.0,
        y: 200.0,
        width: 100.0,
        height: 40.0,
    }
}

#[test]
fn left_and_right_are_outside_the_anchor_and_align_on_the_vertical_axis() {
    let left = PanelPlacement::resolve(
        anchor(),
        bounds(),
        (120.0, 80.0),
        FloatingSide::Left,
        FloatingAlign::Center,
        4.0,
    );
    let right = PanelPlacement::resolve(
        anchor(),
        bounds(),
        (120.0, 80.0),
        FloatingSide::Right,
        FloatingAlign::End,
        4.0,
    );
    assert_eq!((left.x, left.y), (176.0, 180.0));
    assert_eq!((right.x, right.y), (404.0, 160.0));
}
#[test]
fn panels_flip_at_edges_before_clamping() {
    let trigger = AnchorRect {
        x: 760.0,
        y: 550.0,
        width: 24.0,
        height: 30.0,
    };
    let right = PanelPlacement::resolve(
        trigger,
        bounds(),
        (120.0, 80.0),
        FloatingSide::Right,
        FloatingAlign::Start,
        4.0,
    );
    let bottom = PanelPlacement::resolve(
        trigger,
        bounds(),
        (120.0, 80.0),
        FloatingSide::Bottom,
        FloatingAlign::End,
        4.0,
    );
    assert_eq!(right.x, 636.0);
    assert_eq!(bottom.y, 466.0);
    assert!(right.y + 80.0 <= bounds().bottom);
}
#[test]
fn oversized_panels_do_not_expand_the_window_used_for_placement() {
    let viewport = PanelBounds {
        left: 20.0,
        top: 24.0,
        right: 300.0,
        bottom: 200.0,
    };
    let placement = PanelPlacement::resolve(
        anchor(),
        viewport,
        (520.0, 360.0),
        FloatingSide::Bottom,
        FloatingAlign::Center,
        4.0,
    );
    assert_eq!((placement.x, placement.y), (20.0, 24.0));
}
