//! ColorUI paint for the same headless primitives shadcn restyles.
//!
//! Public props are the headless props. Only appearance differs.

use arkit_component::components::{
    Alert as HeadlessAlert, AlertDescription as HeadlessAlertDescription, AlertDescriptionProps,
    AlertList as HeadlessAlertList, AlertListProps, AlertProps, AlertTitle as HeadlessAlertTitle,
    AlertTitleProps, Avatar as HeadlessAvatar, AvatarFallback as HeadlessAvatarFallback,
    AvatarFallbackProps, AvatarProps, Badge as HeadlessBadge, BadgeProps, Button as HeadlessButton,
    ButtonProps, Card as HeadlessCard, CardContent as HeadlessCardContent, CardContentProps,
    CardDescription as HeadlessCardDescription, CardDescriptionProps,
    CardFooter as HeadlessCardFooter, CardFooterProps, CardHeader as HeadlessCardHeader,
    CardHeaderProps, CardProps, CardTitle as HeadlessCardTitle, CardTitleProps,
    Checkbox as HeadlessCheckbox, CheckboxProps, Input as HeadlessInput, InputProps,
    Label as HeadlessLabel, LabelProps, Progress as HeadlessProgress, ProgressProps,
    Separator as HeadlessSeparator, SeparatorProps, Skeleton as HeadlessSkeleton, SkeletonProps,
    Switch as HeadlessSwitch, SwitchProps,
};
use arkit_prelude::*;

use crate::kit::{
    colorui_alert, colorui_avatar, colorui_badge, colorui_button, colorui_card, colorui_checkbox,
    colorui_input, colorui_label, colorui_progress, colorui_separator, colorui_skeleton,
    colorui_switch,
};
use crate::theme::use_colorui_theme;

pub use arkit_component::components::{
    AlertVariant, BadgeVariant, ButtonSize, ButtonVariant, InputMode,
};

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let theme = use_colorui_theme();
    let appearance = colorui_button(
        &theme,
        props.color,
        props.size,
        props.variant,
        false,
        props.round.unwrap_or(false),
        props.block.unwrap_or(false),
        props.disabled.unwrap_or(false),
        props.width.clone(),
        props.height,
        props.shadow,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessButton, props, "Button")
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let appearance = colorui_badge(
        &use_colorui_theme(),
        props.color,
        props.variant,
        false,
        props.pill.unwrap_or(false),
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessBadge, props, "Badge")
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let appearance = colorui_card(&use_colorui_theme(), props.shadow.unwrap_or(true));
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCard, props, "Card")
}

#[component]
pub fn CardHeader(props: CardHeaderProps) -> Element {
    let appearance = colorui_card(&use_colorui_theme(), false);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCardHeader, props, "CardHeader")
}

#[component]
pub fn CardTitle(props: CardTitleProps) -> Element {
    let appearance = colorui_card(&use_colorui_theme(), false);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCardTitle, props, "CardTitle")
}

#[component]
pub fn CardDescription(props: CardDescriptionProps) -> Element {
    let appearance = colorui_card(&use_colorui_theme(), false);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCardDescription, props, "CardDescription")
}

#[component]
pub fn CardContent(props: CardContentProps) -> Element {
    let appearance = colorui_card(&use_colorui_theme(), false);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCardContent, props, "CardContent")
}

#[component]
pub fn CardFooter(props: CardFooterProps) -> Element {
    let appearance = colorui_card(&use_colorui_theme(), false);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCardFooter, props, "CardFooter")
}

#[component]
pub fn Input(props: InputProps) -> Element {
    let appearance = colorui_input(&use_colorui_theme(), props.invalid, props.height);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessInput, props, "Input")
}

#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let appearance = colorui_progress(&use_colorui_theme(), None, props.height);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessProgress, props, "Progress")
}

#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let appearance = colorui_avatar(
        &use_colorui_theme(),
        props.ring.unwrap_or(false),
        props.radius,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAvatar, props, "Avatar")
}

#[component]
pub fn AvatarFallback(props: AvatarFallbackProps) -> Element {
    let appearance = colorui_avatar(&use_colorui_theme(), false, None);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAvatarFallback, props, "AvatarFallback")
}

#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let appearance = colorui_switch(&use_colorui_theme());
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessSwitch, props, "Switch")
}

#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let appearance = colorui_checkbox(&use_colorui_theme(), props.checked.unwrap_or(false));
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCheckbox, props, "Checkbox")
}

#[component]
pub fn Alert(props: AlertProps) -> Element {
    let appearance = colorui_alert(
        &use_colorui_theme(),
        matches!(props.variant, AlertVariant::Destructive),
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAlert, props, "Alert")
}

#[component]
pub fn AlertTitle(props: AlertTitleProps) -> Element {
    let appearance = colorui_alert(
        &use_colorui_theme(),
        matches!(props.variant, AlertVariant::Destructive),
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAlertTitle, props, "AlertTitle")
}

#[component]
pub fn AlertDescription(props: AlertDescriptionProps) -> Element {
    let appearance = colorui_alert(
        &use_colorui_theme(),
        matches!(props.variant, AlertVariant::Destructive),
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAlertDescription, props, "AlertDescription")
}

#[component]
pub fn AlertList(props: AlertListProps) -> Element {
    let appearance = colorui_alert(
        &use_colorui_theme(),
        matches!(props.variant, AlertVariant::Destructive),
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAlertList, props, "AlertList")
}

#[component]
pub fn Separator(props: SeparatorProps) -> Element {
    let appearance = colorui_separator(&use_colorui_theme());
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessSeparator, props, "Separator")
}

#[component]
pub fn Label(props: LabelProps) -> Element {
    let appearance = colorui_label(&use_colorui_theme());
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessLabel, props, "Label")
}

#[component]
pub fn Skeleton(props: SkeletonProps) -> Element {
    let appearance = colorui_skeleton(&use_colorui_theme());
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessSkeleton, props, "Skeleton")
}

fn forward<P, M: 'static>(
    render: impl dioxus_core::ComponentFunction<P, M>,
    props: P,
    name: &'static str,
) -> Element
where
    P: dioxus_core::Properties + 'static,
{
    let node =
        dioxus_core::DynamicNode::Component(dioxus_core::VComponent::new(render, props, name));
    rsx! { {node} }
}
