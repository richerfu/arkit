pub(crate) const SHOWCASE_COMPONENTS: [(&str, &str); 32] = [
    ("accordion", "Accordion"),
    ("alert", "Alert"),
    ("alert-dialog", "Alert Dialog"),
    ("aspect-ratio", "Aspect Ratio"),
    ("avatar", "Avatar"),
    ("badge", "Badge"),
    ("button", "Button"),
    ("card", "Card"),
    ("checkbox", "Checkbox"),
    ("collapsible", "Collapsible"),
    ("context-menu", "Context Menu"),
    ("dialog", "Dialog"),
    ("dropdown-menu", "Dropdown Menu"),
    ("hover-card", "Hover Card"),
    ("icon", "Icon"),
    ("input", "Input"),
    ("label", "Label"),
    ("menubar", "Menubar"),
    ("popover", "Popover"),
    ("progress", "Progress"),
    ("radio-group", "Radio Group"),
    ("select", "Select"),
    ("separator", "Separator"),
    ("skeleton", "Skeleton"),
    ("switch", "Switch"),
    ("tabs", "Tabs"),
    ("text", "Text"),
    ("textarea", "Textarea"),
    ("toggle", "Toggle"),
    ("toggle-group", "Toggle Group"),
    ("tooltip", "Tooltip"),
    ("table", "Table"),
];

pub(crate) fn component_title(slug: &str) -> String {
    SHOWCASE_COMPONENTS
        .iter()
        .find_map(|(item_slug, title)| {
            if *item_slug == slug {
                Some((*title).to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Unknown".to_string())
}
