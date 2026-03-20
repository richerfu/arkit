use std::sync::OnceLock;

use rust_embed::{EmbeddedFile, RustEmbed};

#[derive(RustEmbed)]
#[folder = "assets/lucide"]
struct LucideAssets;

static ICON_NAMES: OnceLock<Vec<String>> = OnceLock::new();

pub(crate) fn normalize_icon_name(name: &str) -> String {
    let trimmed = name.trim();
    let without_ext = trimmed.strip_suffix(".svg").unwrap_or(trimmed);
    without_ext.replace('_', "-").to_ascii_lowercase()
}

fn asset_path(name: &str) -> String {
    format!("{}.svg", normalize_icon_name(name))
}

pub(crate) fn embedded_icon(name: &str) -> Option<EmbeddedFile> {
    LucideAssets::get(&asset_path(name))
}

pub fn has_icon(name: &str) -> bool {
    embedded_icon(name).is_some()
}

pub fn icon_names() -> &'static [String] {
    ICON_NAMES
        .get_or_init(|| {
            let mut names = LucideAssets::iter()
                .map(|path| path.as_ref().trim_end_matches(".svg").to_string())
                .collect::<Vec<_>>();
            names.sort();
            names
        })
        .as_slice()
}
