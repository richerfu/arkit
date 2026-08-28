//! Index — alphabet sidebar over a grouped list (contacts / city picker).
//!
//! The rail is at most a few dozen keys and is never virtualized. The content
//! list is an ArkUI `List` + [`VirtualSource`]. Default grouping puts symbols
//! and ASCII digits under [`INDEX_SYMBOL`] (`#`), and Latin letters under
//! `A`–`Z`. [`IndexProps::show_empty_indexes`] keeps or hides rail keys that
//! have no rows.

use crate::theme::*;
use arkit_arkui::VirtualKind;
use arkit_hooks::use_virtual_source_items_keyed;
use arkit_prelude::*;
use dioxus_core::Callback;

const RAIL_WIDTH: f32 = 22.0;
const RAIL_LETTER_SIZE: f32 = 10.0;
const RAIL_LETTER_LINE: f32 = 14.0;
const HEADER_HEIGHT: f32 = 32.0;
const ROW_HEIGHT: f32 = 56.0;
const POPUP_SIZE: f32 = 56.0;
const LIST_CACHED_COUNT: i32 = 8;
const HIT_FILL: u32 = 0x0100_0000;
const EMPTY_RAIL_ALPHA: u8 = 0x55;

/// Rail / list bucket for punctuation, ASCII digits, and other non-letter keys.
pub const INDEX_SYMBOL: &str = "#";

/// Default rail: `#`, then `A`–`Z`.
pub fn default_index_keys() -> Vec<String> {
    let mut keys = Vec::with_capacity(27);
    keys.push(INDEX_SYMBOL.to_string());
    keys.extend(('A'..='Z').map(|letter| letter.to_string()));
    keys
}

/// Maps a title (or explicit key) onto the default buckets.
pub fn classify_index_key(text: &str) -> String {
    let Some(ch) = text.trim().chars().find(|ch| !ch.is_whitespace()) else {
        return INDEX_SYMBOL.to_string();
    };
    if ch.is_ascii_alphabetic() {
        return ch.to_ascii_uppercase().to_string();
    }
    INDEX_SYMBOL.to_string()
}

/// Resolves an item's group key.
///
/// Empty `index` classifies `title`. A one-character letter stays `A`–`Z`;
/// digits and symbols fold into `#`. Longer labels (`"VIP"`) are kept as-is.
pub fn normalize_index_key(index: &str, title: &str) -> String {
    let trimmed = index.trim();
    if trimmed.is_empty() {
        return classify_index_key(title);
    }
    if trimmed == INDEX_SYMBOL || trimmed == "0-9" {
        return INDEX_SYMBOL.to_string();
    }
    if trimmed.chars().nth(1).is_none() {
        return classify_index_key(trimmed);
    }
    trimmed.to_string()
}

fn item_group_keys(items: &[IndexItemSpec]) -> Vec<String> {
    items
        .iter()
        .map(|item| normalize_index_key(&item.index, &item.title))
        .collect()
}

fn unique_indexes(keys: &[String]) -> Vec<String> {
    let mut indexes = Vec::new();
    for key in keys {
        if !indexes.iter().any(|index| index == key) {
            indexes.push(key.clone());
        }
    }
    indexes
}

fn resolve_rail(indexes: Option<&[String]>, populated: &[String], show_empty: bool) -> Vec<String> {
    let mut rail = indexes
        .map(|keys| keys.to_vec())
        .unwrap_or_else(default_index_keys);
    for key in populated {
        if !rail.iter().any(|index| index == key) {
            rail.push(key.clone());
        }
    }
    if show_empty {
        rail
    } else {
        rail.into_iter()
            .filter(|key| populated.iter().any(|index| index == key))
            .collect()
    }
}

/// One row in [`Index`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexItemSpec {
    /// Group key. Empty classifies [`Self::title`] into `#` or `A`–`Z`.
    pub index: String,
    pub title: String,
    /// Muted copy under the title.
    pub description: Option<String>,
}

impl IndexItemSpec {
    /// Title under `index`. Pass `""` to classify `title`.
    pub fn new(index: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            index: index.into(),
            title: title.into(),
            description: None,
        }
    }

    /// Adds muted supporting copy under the title.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Context for a custom list row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexItemContext {
    pub item: IndexItemSpec,
    /// Index into the original [`IndexProps::items`] vec.
    pub item_index: usize,
    /// Normalized group key (`"A"`, `"#"`).
    pub index: String,
}

/// Context for a custom section header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexHeaderContext {
    pub index: String,
}

/// Context for a custom rail cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexBarSlot {
    pub index: String,
    pub active: bool,
    /// True when this rail key has no rows (only if empty groups are shown).
    pub empty: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlatKind {
    Header,
    Item(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FlatRow {
    kind: FlatKind,
    index: String,
}

fn flatten_items(keys: &[String], rail: &[String]) -> Vec<FlatRow> {
    let mut rows = Vec::with_capacity(keys.len().saturating_add(rail.len()));
    for index in rail {
        let mut members = Vec::new();
        for (item_index, key) in keys.iter().enumerate() {
            if key == index {
                members.push(item_index);
            }
        }
        if members.is_empty() {
            continue;
        }
        rows.push(FlatRow {
            kind: FlatKind::Header,
            index: index.clone(),
        });
        for item_index in members {
            rows.push(FlatRow {
                kind: FlatKind::Item(item_index),
                index: index.clone(),
            });
        }
    }
    rows
}

fn first_row_for_index(rows: &[FlatRow], index: &str) -> Option<i32> {
    rows.iter()
        .position(|row| row.kind == FlatKind::Header && row.index == index)
        .and_then(|position| i32::try_from(position).ok())
}

fn index_for_row(rows: &[FlatRow], row: i32) -> Option<&str> {
    let position = usize::try_from(row).ok()?;
    rows.get(position).map(|row| row.index.as_str())
}

/// Visible-group spy: the first on-screen row, except at the bottom.
///
/// A short tail group (`Z` with one city) can never pin its header to the top.
/// `first_index` then stays on `W`/`X` even though the list is fully scrolled.
/// When the last row is on screen and content remains above, use that last
/// group instead.
fn spy_index_for_range(rows: &[FlatRow], first: i32, last: i32) -> Option<&str> {
    if rows.is_empty() {
        return None;
    }
    let last_row = i32::try_from(rows.len() - 1).ok()?;
    if last >= last_row && first > 0 {
        return rows.last().map(|row| row.index.as_str());
    }
    index_for_row(rows, first)
}

fn letter_at(y: f32, height: f32, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    let t = (y / height.max(1.0)).clamp(0.0, 0.999_999);
    ((t * count as f32) as usize).min(count - 1)
}

fn is_empty_index(empty: &[String], key: &str) -> bool {
    empty.iter().any(|index| index == key)
}

/// Props for [`Index`].
#[derive(Props, Clone, PartialEq)]
pub struct IndexProps {
    pub items: Vec<IndexItemSpec>,
    /// Rail keys. Defaults to [`default_index_keys`]: `#`, `A`–`Z`.
    #[props(default)]
    pub indexes: Option<Vec<String>>,
    /// When false, rail keys with no rows are omitted. Defaults to false.
    #[props(default)]
    pub show_empty_indexes: bool,
    #[props(default)]
    pub render_item: Option<Callback<IndexItemContext, Element>>,
    #[props(default)]
    pub render_header: Option<Callback<IndexHeaderContext, Element>>,
    #[props(default)]
    pub render_bar: Option<Callback<IndexBarSlot, Element>>,
    #[props(default)]
    pub on_select: EventHandler<usize>,
    #[props(default)]
    pub on_index_change: EventHandler<String>,
    #[props(default = "100%".to_string())]
    pub width: String,
    #[props(default = "100%".to_string())]
    pub height: String,
}

/// Grouped list with an alphabet rail. Content is a virtual `List`.
#[component]
pub fn Index(props: IndexProps) -> Element {
    let theme = use_theme();
    let runtime = arkit_runtime::use_runtime_handle();
    let items = props.items;
    let group_keys = item_group_keys(&items);
    let populated = unique_indexes(&group_keys);
    let indexes = resolve_rail(
        props.indexes.as_deref(),
        &populated,
        props.show_empty_indexes,
    );
    let empty: Vec<String> = indexes
        .iter()
        .filter(|key| !populated.iter().any(|index| index == *key))
        .cloned()
        .collect();
    let rows = flatten_items(&group_keys, &indexes);
    let rail = indexes.clone();
    let mut active = use_signal(|| indexes.first().cloned().unwrap_or_default());
    let mut scrub = use_signal(|| None::<String>);
    let jump = use_signal(|| None::<i32>);
    let item_count = items.len();
    let reset_runtime = runtime.clone();
    use_effect(use_reactive((&item_count,), move |(_item_count,)| {
        let mut jump = jump;
        jump.set(Some(0));
        reset_runtime.queue_ui(move || {
            jump.set(None);
        });
    }));
    let jump_token = jump().map(|index| index.to_string());

    let display = scrub()
        .or_else(|| {
            let current = active();
            if current.is_empty() {
                None
            } else {
                Some(current)
            }
        })
        .unwrap_or_default();

    let render_tag = match (props.render_item.is_some(), props.render_header.is_some()) {
        (true, true) => "ih",
        (true, false) => "i",
        (false, true) => "h",
        (false, false) => "d",
    };
    let item_keys: Vec<String> = rows
        .iter()
        .enumerate()
        .map(|(row_index, row)| match row.kind {
            FlatKind::Header => format!("h:{render_tag}:{row_index}:{}", row.index),
            FlatKind::Item(item_index) => {
                let item = &items[item_index];
                format!(
                    "i:{render_tag}:{item_index}:{}:{}:{}",
                    row.index,
                    item.title,
                    item.description.as_deref().unwrap_or("")
                )
            }
        })
        .collect();

    let list_items = items.clone();
    let list_rows = rows.clone();
    let list_keys = group_keys.clone();
    let header_bg = theme.colors.muted;
    let header_fg = theme.colors.muted_foreground;
    let title_fg = theme.colors.foreground;
    let desc_fg = theme.colors.muted_foreground;
    let row_bg = theme.colors.background;
    let on_select = props.on_select;
    let render_item = props.render_item;
    let render_header = props.render_header;
    let source = use_virtual_source_items_keyed(VirtualKind::List, item_keys, move |index| {
        let Some(row) = list_rows.get(index as usize) else {
            return rsx! { row { height: 1.0 } };
        };
        match row.kind {
            FlatKind::Header => {
                if let Some(render_header) = render_header {
                    render_header.call(IndexHeaderContext {
                        index: row.index.clone(),
                    })
                } else {
                    default_header_row(row.index.clone(), header_bg, header_fg)
                }
            }
            FlatKind::Item(item_index) => {
                let item = list_items[item_index].clone();
                let group = list_keys
                    .get(item_index)
                    .cloned()
                    .unwrap_or_else(|| row.index.clone());
                let inner = if let Some(render_item) = render_item {
                    render_item.call(IndexItemContext {
                        item: item.clone(),
                        item_index,
                        index: group,
                    })
                } else {
                    default_item_row(item, title_fg, desc_fg)
                };
                rsx! {
                    column {
                        width: "100%",
                        background_color: row_bg,
                        onclick: move |_| on_select.call(item_index),
                        {inner}
                    }
                }
            }
        }
    });

    let scroll_rows = rows.clone();
    let on_index_change = props.on_index_change;
    rsx! {
        stack {
            width: props.width.clone(),
            height: props.height.clone(),
            alignment: "center",
            list {
                virtual_source: source,
                width: "100%",
                height: "100%",
                scroll_bar: "off",
                list_cached_count: LIST_CACHED_COUNT,
                scroll_to_index: jump_token,
                ontouch: move |event| {
                    let Some(pointer) = event.data().pointer else {
                        return;
                    };
                    if pointer.action == dioxus_elements::event::PointerAction::Down
                        && scrub.peek().is_some()
                    {
                        scrub.set(None);
                    }
                },
                onscroll: move |event| {
                    if jump.peek().is_some() || scrub.peek().is_some() {
                        return;
                    }
                    let data = event.data();
                    let Some(index) =
                        spy_index_for_range(&scroll_rows, data.first_index, data.last_index)
                    else {
                        return;
                    };
                    if active.peek().as_str() != index {
                        active.set(index.to_string());
                        on_index_change.call(index.to_string());
                    }
                }
            }
            row {
                width: "100%",
                height: "100%",
                justify_content: "end",
                align_items: "center",
                hit_test_behavior: "transparent",
                ontouch: move |event| {
                    let Some(pointer) = event.data().pointer else {
                        return;
                    };
                    if pointer.action != dioxus_elements::event::PointerAction::Down {
                        return;
                    }
                    if pointer.x + RAIL_WIDTH < pointer.target_width {
                        if scrub.peek().is_some() {
                            scrub.set(None);
                        }
                    }
                },
                column {
                    layout_weight: 1.0,
                    hit_test_behavior: "none",
                }
                IndexBar {
                    indexes: rail,
                    active: if display.is_empty() { None } else { Some(display.clone()) },
                    empty: empty,
                    render_index: props.render_bar,
                    on_select: {
                        let rows = rows.clone();
                        let on_index_change = props.on_index_change;
                        let runtime = runtime.clone();
                        move |letter: String| {
                            scrub.set(Some(letter.clone()));
                            if active.peek().as_str() != letter.as_str() {
                                active.set(letter.clone());
                                on_index_change.call(letter.clone());
                            }
                            if let Some(row) = first_row_for_index(&rows, &letter) {
                                let mut jump = jump;
                                jump.set(Some(row));
                                runtime.queue_ui(move || {
                                    jump.set(None);
                                });
                            }
                        }
                    },
                    on_scrub_end: move |_| {
                        scrub.set(None);
                    },
                }
            }
            if scrub().is_some() && !display.is_empty() {
                column {
                    width: POPUP_SIZE,
                    height: POPUP_SIZE,
                    border_radius: theme.radii.lg,
                    background_color: with_alpha(theme.colors.foreground, 0xE6),
                    align_items: "center",
                    justify_content: "center",
                    hit_test_behavior: "none",
                    text {
                        content: display,
                        font_size: typography::XXL,
                        font_weight: 600,
                        font_color: theme.colors.background,
                    }
                }
            }
        }
    }
}

fn default_header_row(index: String, background: u32, color: u32) -> Element {
    rsx! {
        row {
            width: "100%",
            height: HEADER_HEIGHT,
            padding_left: spacing::MD,
            padding_right: spacing::MD,
            align_items: "center",
            background_color: background,
            text {
                content: index,
                font_size: typography::XS,
                font_weight: 600,
                font_color: color,
            }
        }
    }
}

fn default_item_row(item: IndexItemSpec, title_fg: u32, desc_fg: u32) -> Element {
    rsx! {
        column {
            width: "100%",
            height: ROW_HEIGHT,
            padding_left: spacing::MD,
            padding_right: spacing::XXL,
            align_items: "start",
            justify_content: "center",
            text {
                content: item.title.clone(),
                width: "100%",
                font_size: typography::SM,
                font_weight: 500,
                font_color: title_fg,
                text_align: "start",
                max_lines: 1,
                text_overflow: "ellipsis",
            }
            if let Some(description) = item.description.clone() {
                text {
                    content: description,
                    width: "100%",
                    font_size: typography::XS,
                    font_color: desc_fg,
                    text_align: "start",
                    margin_top: 2.0,
                    max_lines: 1,
                    text_overflow: "ellipsis",
                }
            }
        }
    }
}

fn default_bar_slot(slot: IndexBarSlot, theme: Theme) -> Element {
    let color = if slot.empty {
        with_alpha(theme.colors.muted_foreground, EMPTY_RAIL_ALPHA)
    } else if slot.active {
        theme.colors.primary
    } else {
        theme.colors.muted_foreground
    };
    rsx! {
        text {
            content: slot.index,
            font_size: RAIL_LETTER_SIZE,
            font_weight: if slot.active { 700 } else { 500 },
            font_color: color,
            line_height: RAIL_LETTER_LINE,
            text_align: "center",
            hit_test_behavior: "none",
        }
    }
}

/// Props for [`IndexBar`].
#[derive(Props, Clone, PartialEq)]
pub struct IndexBarProps {
    pub indexes: Vec<String>,
    #[props(default)]
    pub active: Option<String>,
    /// Keys on the rail that currently have no list rows.
    #[props(default)]
    pub empty: Vec<String>,
    #[props(default)]
    pub render_index: Option<Callback<IndexBarSlot, Element>>,
    #[props(default)]
    pub on_select: EventHandler<String>,
    #[props(default)]
    pub on_scrub_end: EventHandler<()>,
}

/// Vertical letter rail. Drag or tap to emit the letter under the pointer.
#[component]
pub fn IndexBar(props: IndexBarProps) -> Element {
    let theme = use_theme();
    let indexes = props.indexes;
    let count = indexes.len();
    let touch_indexes = indexes.clone();
    let active = props.active.clone();
    let empty = props.empty;
    let render_index = props.render_index;
    let on_select = props.on_select;
    let on_scrub_end = props.on_scrub_end;

    rsx! {
        column {
            width: RAIL_WIDTH,
            height: "100%",
            padding_top: spacing::XS,
            padding_bottom: spacing::XS,
            align_items: "center",
            background_color: HIT_FILL,
            on_touch: move |event| {
                let Some(pointer) = event.data().pointer else {
                    return;
                };
                if count == 0 {
                    return;
                }
                let height = if pointer.target_height > 1.0 {
                    pointer.target_height
                } else {
                    count as f32 * RAIL_LETTER_LINE
                };
                let slot = letter_at(pointer.y, height, count);
                match pointer.action {
                    dioxus_elements::event::PointerAction::Down
                    | dioxus_elements::event::PointerAction::Move => {
                        if let Some(letter) = touch_indexes.get(slot) {
                            on_select.call(letter.clone());
                        }
                    }
                    dioxus_elements::event::PointerAction::Up
                    | dioxus_elements::event::PointerAction::Cancel => {
                        if let Some(letter) = touch_indexes.get(slot) {
                            on_select.call(letter.clone());
                        }
                        on_scrub_end.call(());
                    }
                    dioxus_elements::event::PointerAction::Unknown => {}
                }
            },
            for letter in indexes.iter() {
                {
                    let slot = IndexBarSlot {
                        index: letter.clone(),
                        active: active.as_deref() == Some(letter.as_str()),
                        empty: is_empty_index(&empty, letter),
                    };
                    let cell = if let Some(render_index) = render_index {
                        render_index.call(slot)
                    } else {
                        default_bar_slot(slot, theme)
                    };
                    rsx! {
                        column {
                            width: RAIL_WIDTH,
                            layout_weight: 1.0,
                            align_items: "center",
                            justify_content: "center",
                            hit_test_behavior: "none",
                            {cell}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_index_key, default_index_keys, first_row_for_index, flatten_items, index_for_row,
        item_group_keys, letter_at, normalize_index_key, resolve_rail, spy_index_for_range,
        unique_indexes, FlatKind, IndexItemSpec, INDEX_SYMBOL,
    };

    fn sample() -> Vec<IndexItemSpec> {
        vec![
            IndexItemSpec::new("A", "Ada"),
            IndexItemSpec::new("A", "Alan"),
            IndexItemSpec::new("C", "Carol").with_description("ext 12"),
        ]
    }

    #[test]
    fn classify_puts_digits_and_symbols_in_default_buckets() {
        assert_eq!(classify_index_key("Beijing"), "B");
        assert_eq!(classify_index_key("ada"), "A");
        assert_eq!(classify_index_key("12306"), INDEX_SYMBOL);
        assert_eq!(classify_index_key("7-eleven"), INDEX_SYMBOL);
        assert_eq!(classify_index_key("*star"), INDEX_SYMBOL);
        assert_eq!(classify_index_key("@help"), INDEX_SYMBOL);
        assert_eq!(classify_index_key("北京"), INDEX_SYMBOL);
        assert_eq!(classify_index_key("   "), INDEX_SYMBOL);
        assert_eq!(normalize_index_key("", "42 Store"), INDEX_SYMBOL);
        assert_eq!(normalize_index_key("3", "ignored"), INDEX_SYMBOL);
        assert_eq!(normalize_index_key("#", "ignored"), INDEX_SYMBOL);
        assert_eq!(normalize_index_key("0-9", "ignored"), INDEX_SYMBOL);
        assert_eq!(normalize_index_key("VIP", "Ada"), "VIP");
    }

    #[test]
    fn default_keys_start_with_symbol_then_a_to_z() {
        let keys = default_index_keys();
        assert_eq!(keys[0], INDEX_SYMBOL);
        assert_eq!(keys[1], "A");
        assert_eq!(keys.last().map(String::as_str), Some("Z"));
        assert_eq!(keys.len(), 27);
        assert!(!keys.iter().any(|key| key == "0-9"));
    }

    #[test]
    fn flatten_follows_rail_order_and_skips_empty_groups() {
        let items = sample();
        let keys = item_group_keys(&items);
        let rail = resolve_rail(None, &unique_indexes(&keys), false);
        assert_eq!(rail, vec!["A", "C"]);
        let rows = flatten_items(&keys, &rail);
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].kind, FlatKind::Header);
        assert_eq!(rows[0].index, "A");
        assert_eq!(rows[1].kind, FlatKind::Item(0));
        assert_eq!(rows[3].kind, FlatKind::Header);
        assert_eq!(rows[3].index, "C");
        assert_eq!(first_row_for_index(&rows, "C"), Some(3));
        assert_eq!(index_for_row(&rows, 4), Some("C"));
        assert_eq!(first_row_for_index(&rows, "B"), None);
        assert_eq!(spy_index_for_range(&rows, 0, 2), Some("A"));
        assert_eq!(spy_index_for_range(&rows, 0, 4), Some("A"));
        assert_eq!(spy_index_for_range(&rows, 1, 4), Some("C"));
        assert_eq!(spy_index_for_range(&rows, 3, 4), Some("C"));
    }

    #[test]
    fn spy_uses_last_group_when_a_short_tail_cannot_reach_the_top() {
        let items = vec![
            IndexItemSpec::new("W", "Wuhan"),
            IndexItemSpec::new("W", "Urumqi"),
            IndexItemSpec::new("X", "Xian"),
            IndexItemSpec::new("X", "Xiamen"),
            IndexItemSpec::new("Y", "Yinchuan"),
            IndexItemSpec::new("Z", "Zhengzhou"),
        ];
        let keys = item_group_keys(&items);
        let rail = resolve_rail(None, &unique_indexes(&keys), false);
        let rows = flatten_items(&keys, &rail);
        let last = i32::try_from(rows.len() - 1).unwrap();
        assert_eq!(spy_index_for_range(&rows, 0, 5), Some("W"));
        assert_eq!(spy_index_for_range(&rows, 3, last - 1), Some("X"));
        assert_eq!(spy_index_for_range(&rows, 1, last), Some("Z"));
        assert_eq!(spy_index_for_range(&rows, 6, last), Some("Z"));
        assert!(spy_index_for_range(&[], 0, 0).is_none());
    }

    #[test]
    fn empty_groups_stay_on_the_rail_when_requested() {
        let keys = item_group_keys(&sample());
        let populated = unique_indexes(&keys);
        let hidden = resolve_rail(None, &populated, false);
        let shown = resolve_rail(None, &populated, true);
        assert_eq!(hidden, vec!["A", "C"]);
        assert!(shown.contains(&INDEX_SYMBOL.to_string()));
        assert!(shown.contains(&"B".to_string()));
        assert!(shown.contains(&"A".to_string()));
        assert_eq!(shown.len(), 27);
        assert!(!shown.iter().any(|key| key == "0-9"));
        let custom = resolve_rail(
            Some(&["热门".to_string(), "A".to_string()]),
            &populated,
            false,
        );
        assert_eq!(custom, vec!["A", "C"]);
        let custom_shown = resolve_rail(
            Some(&["热门".to_string(), "A".to_string()]),
            &populated,
            true,
        );
        assert_eq!(
            custom_shown,
            vec!["热门".to_string(), "A".to_string(), "C".to_string()]
        );
    }

    #[test]
    fn letter_at_maps_pointer_y_across_the_rail() {
        assert_eq!(letter_at(0.0, 260.0, 26), 0);
        assert_eq!(letter_at(10.0, 260.0, 26), 1);
        assert_eq!(letter_at(259.0, 260.0, 26), 25);
        assert_eq!(letter_at(-4.0, 260.0, 26), 0);
        assert_eq!(letter_at(400.0, 260.0, 26), 25);
        assert_eq!(letter_at(0.0, 100.0, 0), 0);
    }
}
