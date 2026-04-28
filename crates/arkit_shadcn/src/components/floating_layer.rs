use super::*;
use std::rc::Rc;

pub type FloatingSide = arkit::FloatingSide;
pub type FloatingAlign = arkit::FloatingAlign;
pub(crate) type FloatingSurfaceRegistry = arkit_widget::FloatingSurfaceRegistry;

const FLOATING_SIDE_OFFSET_VP: f32 = spacing::XXS;

fn floating_spec(
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
    offset_vp: f32,
    on_dismiss: bool,
    pass_through_dismiss: bool,
    match_trigger_width: bool,
    native: bool,
) -> arkit::FloatingOverlaySpec {
    arkit::FloatingOverlaySpec {
        open,
        side,
        align,
        offset_vp,
        match_trigger_width,
        dismiss_mode: if on_dismiss {
            if pass_through_dismiss {
                arkit::OverlayDismissMode::PassThrough
            } else {
                arkit::OverlayDismissMode::Backdrop
            }
        } else {
            arkit::OverlayDismissMode::None
        },
        strategy: if native {
            arkit::OverlayStrategy::Native
        } else {
            arkit::OverlayStrategy::Portal
        },
    }
}

pub(crate) fn floating_panel_aligned<Message: 'static>(
    trigger: Element<Message>,
    panel: Element<Message>,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
    on_dismiss: Option<Rc<dyn Fn()>>,
    pass_through_dismiss: bool,
    register_surfaces: Vec<FloatingSurfaceRegistry>,
    dismiss_registry: Option<FloatingSurfaceRegistry>,
) -> Element<Message> {
    arkit_widget::floating_overlay_with_surfaces(
        trigger,
        if open { Some(panel) } else { None },
        floating_spec(
            open,
            side,
            align,
            FLOATING_SIDE_OFFSET_VP,
            on_dismiss.is_some(),
            pass_through_dismiss,
            false,
            false,
        ),
        on_dismiss,
        register_surfaces,
        dismiss_registry,
    )
}

pub(crate) fn floating_panel_aligned_with_builder<Message: 'static>(
    trigger: Element<Message>,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
    offset_vp: f32,
    panel_builder: Rc<dyn Fn(Option<f32>) -> Element<Message>>,
    on_dismiss: Option<Rc<dyn Fn()>>,
    pass_through_dismiss: bool,
    register_surfaces: Vec<FloatingSurfaceRegistry>,
    dismiss_registry: Option<FloatingSurfaceRegistry>,
) -> Element<Message> {
    arkit_widget::floating_overlay_with_builder_and_surfaces(
        trigger,
        floating_spec(
            open,
            side,
            align,
            offset_vp,
            on_dismiss.is_some(),
            pass_through_dismiss,
            false,
            false,
        ),
        panel_builder,
        on_dismiss,
        register_surfaces,
        dismiss_registry,
    )
}

pub(crate) fn floating_panel<Message>(
    trigger: Element<Message>,
    panel: Element<Message>,
    open: bool,
    side: FloatingSide,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message>
where
    Message: 'static,
{
    floating_panel_aligned(
        trigger,
        panel,
        open,
        side,
        FloatingAlign::Center,
        on_dismiss,
        false,
        Vec::new(),
        None,
    )
}

#[allow(dead_code)]
pub(crate) fn native_floating_panel<Message>(
    trigger: Element<Message>,
    panel: Element<Message>,
    open: bool,
    side: FloatingSide,
) -> Element<Message>
where
    Message: 'static,
{
    native_floating_panel_aligned(trigger, panel, open, side, FloatingAlign::Center)
}

#[allow(dead_code)]
pub(crate) fn native_floating_panel_aligned<Message>(
    trigger: Element<Message>,
    panel: Element<Message>,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
) -> Element<Message>
where
    Message: 'static,
{
    arkit::floating_overlay(
        trigger,
        if open { Some(panel) } else { None },
        floating_spec(
            open,
            side,
            align,
            FLOATING_SIDE_OFFSET_VP,
            false,
            false,
            false,
            true,
        ),
        None,
    )
}

pub(crate) fn floating_panel_with_builder<Message: 'static>(
    trigger: Element<Message>,
    open: bool,
    side: FloatingSide,
    align: FloatingAlign,
    panel_builder: Rc<dyn Fn(Option<f32>) -> Element<Message>>,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message> {
    arkit_widget::floating_overlay_with_builder_and_surfaces(
        trigger,
        floating_spec(
            open,
            side,
            align,
            FLOATING_SIDE_OFFSET_VP,
            on_dismiss.is_some(),
            false,
            true,
            false,
        ),
        panel_builder,
        on_dismiss,
        Vec::new(),
        None,
    )
}
