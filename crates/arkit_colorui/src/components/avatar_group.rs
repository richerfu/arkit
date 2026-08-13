//! Overlapping ColorUI avatar stack.

use super::{Avatar, AvatarFallback};
use arkit_prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct AvatarGroupProps {
    pub items: Vec<(Option<String>, String)>,
}

#[component]
pub fn AvatarGroup(props: AvatarGroupProps) -> Element {
    rsx! {
        row {
            align_items: "center",
            padding_left: 12.0,
            for (index, (src, fallback)) in props.items.iter().cloned().enumerate() {
                row {
                    margin_left: if index == 0 { 0.0 } else { -12.0 },
                    Avatar {
                        src,
                        ring: Some(true),
                        fallback: Some(rsx! { AvatarFallback { content: fallback } }),
                    }
                }
            }
        }
    }
}
