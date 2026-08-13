//! shadcn paint for kit primitives, computed from [`crate::theme::Theme`].

use arkit_component::appearance::{
    AlertStyleInput, AvatarStyleInput, BadgeStyleInput, ButtonStyleInput, CardStyleInput,
    CheckboxStyleInput, InputStyleInput, LabelStyleInput, ProgressStyleInput, SeparatorStyleInput,
    SkeletonStyleInput, SwitchStyleInput,
};
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

use crate::kit;
use crate::theme::use_theme;

pub use arkit_component::components::{
    AlertVariant, BadgeVariant, ButtonSize, ButtonVariant, InputMode,
};

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let theme = use_theme();
    let disabled = props.disabled.unwrap_or(false);
    let appearance = kit::button_appearance(
        &ButtonStyleInput {
            variant: props.variant,
            size: props.size,
            disabled,
            color: props.color,
            round: props.round.unwrap_or(false),
            block: props.block.unwrap_or(false),
            height: props.height,
            border_radius: props.border_radius,
            width: props.width.clone(),
            shadow: props.shadow,
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessButton, props, "Button")
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let theme = use_theme();
    let appearance = kit::badge_appearance(
        &BadgeStyleInput {
            variant: props.variant,
            pill: props.pill.unwrap_or(false),
            color: props.color,
            icon_colors: props.icon_colors,
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessBadge, props, "Badge")
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let theme = use_theme();
    let appearance = kit::card_appearance(
        &CardStyleInput {
            shadow: props.shadow.unwrap_or(true),
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCard, props, "Card")
}

#[component]
pub fn CardHeader(props: CardHeaderProps) -> Element {
    let theme = use_theme();
    let appearance = kit::card_appearance(&CardStyleInput { shadow: false }, &theme);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCardHeader, props, "CardHeader")
}

#[component]
pub fn CardTitle(props: CardTitleProps) -> Element {
    let theme = use_theme();
    let appearance = kit::card_appearance(&CardStyleInput { shadow: false }, &theme);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCardTitle, props, "CardTitle")
}

#[component]
pub fn CardDescription(props: CardDescriptionProps) -> Element {
    let theme = use_theme();
    let appearance = kit::card_appearance(&CardStyleInput { shadow: false }, &theme);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCardDescription, props, "CardDescription")
}

#[component]
pub fn CardContent(props: CardContentProps) -> Element {
    let theme = use_theme();
    let appearance = kit::card_appearance(&CardStyleInput { shadow: false }, &theme);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCardContent, props, "CardContent")
}

#[component]
pub fn CardFooter(props: CardFooterProps) -> Element {
    let theme = use_theme();
    let appearance = kit::card_appearance(&CardStyleInput { shadow: false }, &theme);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCardFooter, props, "CardFooter")
}

#[component]
pub fn Input(props: InputProps) -> Element {
    let theme = use_theme();
    let appearance = kit::input_appearance(
        &InputStyleInput {
            invalid: props.invalid,
            disabled: props.disabled,
            read_only: props.read_only,
            password: matches!(props.mode, InputMode::Password),
            height: props.height,
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessInput, props, "Input")
}

#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let theme = use_theme();
    let appearance = kit::progress_appearance(
        &ProgressStyleInput {
            height: props.height,
            track_color: props.track_color,
            indicator_color: props.indicator_color,
            radius: props.radius,
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessProgress, props, "Progress")
}

#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let theme = use_theme();
    let appearance = kit::avatar_appearance(
        &AvatarStyleInput {
            ring: props.ring.unwrap_or(false),
            radius: props.radius,
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAvatar, props, "Avatar")
}

#[component]
pub fn AvatarFallback(props: AvatarFallbackProps) -> Element {
    let theme = use_theme();
    let appearance = kit::avatar_appearance(
        &AvatarStyleInput {
            ring: false,
            radius: None,
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAvatarFallback, props, "AvatarFallback")
}

#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let theme = use_theme();
    let appearance = kit::switch_appearance(
        &SwitchStyleInput {
            checked: props.checked.unwrap_or(false),
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessSwitch, props, "Switch")
}

#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let theme = use_theme();
    let appearance = kit::checkbox_appearance(
        &CheckboxStyleInput {
            checked: props.checked.unwrap_or(false),
            checked_color: props.checked_color,
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessCheckbox, props, "Checkbox")
}

#[component]
pub fn Alert(props: AlertProps) -> Element {
    let theme = use_theme();
    let appearance = kit::alert_appearance(
        &AlertStyleInput {
            variant: props.variant,
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAlert, props, "Alert")
}

#[component]
pub fn AlertTitle(props: AlertTitleProps) -> Element {
    let theme = use_theme();
    let appearance = kit::alert_appearance(
        &AlertStyleInput {
            variant: props.variant,
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAlertTitle, props, "AlertTitle")
}

#[component]
pub fn AlertDescription(props: AlertDescriptionProps) -> Element {
    let theme = use_theme();
    let appearance = kit::alert_appearance(
        &AlertStyleInput {
            variant: props.variant,
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAlertDescription, props, "AlertDescription")
}

#[component]
pub fn AlertList(props: AlertListProps) -> Element {
    let theme = use_theme();
    let appearance = kit::alert_appearance(
        &AlertStyleInput {
            variant: props.variant,
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessAlertList, props, "AlertList")
}

#[component]
pub fn Separator(props: SeparatorProps) -> Element {
    let theme = use_theme();
    let appearance = kit::separator_appearance(
        &SeparatorStyleInput {
            vertical: props.vertical_height.is_some(),
        },
        &theme,
    );
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessSeparator, props, "Separator")
}

#[component]
pub fn Label(props: LabelProps) -> Element {
    let theme = use_theme();
    let appearance = kit::label_appearance(&LabelStyleInput, &theme);
    let mut props = props;
    props.appearance = Some(appearance);
    forward(HeadlessLabel, props, "Label")
}

#[component]
pub fn Skeleton(props: SkeletonProps) -> Element {
    let theme = use_theme();
    let appearance = kit::skeleton_appearance(
        &SkeletonStyleInput {
            width: props.width,
            height: props.height,
        },
        &theme,
    );
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
