//! shadcn/ui tokens with citations to official new-york-v4 classes.
//!
//! Web sizes are Tailwind rem (`1rem = 16px`). HarmonyOS buttons use a larger
//! touch mapping (`h-12` / `h-9` / `h-14`) so 48 / 36 / 56 matches the
//! pre-split mobile kit. Overlay, dialog, and type stay 1:1 with the CSS.

/// Tailwind rem → vp.
pub const fn rem(value: f32) -> f32 {
    value * 16.0
}

// --- Zinc light (`registry/new-york-v4`, `--background` …) ---------------

pub const ZINC_BG: u32 = 0xFFFFFFFF;
pub const ZINC_FG: u32 = 0xFF09090B;
pub const ZINC_PRIMARY: u32 = 0xFF09090B;
pub const ZINC_PRIMARY_FG: u32 = 0xFFFAFAFA;
pub const ZINC_SECONDARY: u32 = 0xFFF4F4F5;
pub const ZINC_MUTED_FG: u32 = 0xFF71717A;
pub const ZINC_BORDER: u32 = 0xFFE4E4E7;
pub const ZINC_DESTRUCTIVE: u32 = 0xFFEF4444;

// --- geometry: `button.tsx` / `dialog.tsx` / `card.tsx` -------------------

/// Official `h-9`. Mobile default is `h-12` (48) for touch.
pub const BTN_WEB: f32 = rem(2.25);
pub const BTN_HEIGHT: f32 = rem(3.0);
pub const BTN_HEIGHT_SM: f32 = rem(2.25);
pub const BTN_HEIGHT_LG: f32 = rem(3.5);
pub const BTN_ICON: f32 = rem(2.5);
pub const BTN_PX: f32 = rem(1.25);
pub const BTN_PX_SM: f32 = rem(0.75);
pub const BTN_PX_LG: f32 = rem(2.0);
pub const TEXT_SM: f32 = rem(0.875);
pub const TEXT_BASE: f32 = rem(1.0);
pub const TEXT_LG: f32 = rem(1.125);
pub const TEXT_XL: f32 = rem(1.25);
pub const FONT_MEDIUM: u32 = 500;
pub const FONT_SEMIBOLD: u32 = 600;
/// `rounded-md`
pub const RADIUS_MD: f32 = rem(0.375);
/// `rounded-lg`
pub const RADIUS_LG: f32 = rem(0.5);
/// `rounded-xl`
pub const RADIUS_XL: f32 = rem(0.75);
/// `sm:max-w-lg`
pub const DIALOG_MAX_W: f32 = rem(32.0);
/// `p-6`
pub const DIALOG_PAD: f32 = rem(1.5);
/// `top-4 right-4`
pub const DIALOG_CLOSE: f32 = rem(1.0);
pub const DIALOG_CLOSE_SIZE: f32 = rem(1.75);
/// `bg-black/50`
pub const OVERLAY: u32 = 0x80000000;
/// `w-72` popover
pub const POPOVER_W: f32 = rem(18.0);
/// `w-64` hover card
pub const HOVER_CARD_W: f32 = rem(16.0);
/// Official switch `h-5 w-9` mapped to the pre-split 32×18.4 control.
pub const SWITCH_W: f32 = 32.0;
pub const SWITCH_H: f32 = 18.4;
/// `size-4` checkbox
pub const CHECK: f32 = rem(1.0);
/// Progress `h-2`
pub const PROGRESS_H: f32 = rem(0.5);
/// Avatar `size-8`
pub const AVATAR: f32 = rem(2.0);
/// Input mobile mapping of `h-9`
pub const INPUT_H: f32 = rem(3.0);
/// Badge `h-5` / `text-xs`
pub const BADGE_H: f32 = 22.0;
pub const BADGE_FONT: f32 = rem(0.75);
/// Sheet width `sm:max-w-sm` ≈ 384
pub const SHEET_W: f32 = rem(24.0);
pub const DISABLED_OPACITY: f32 = 0.5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zinc_matches_official_light() {
        assert_eq!(ZINC_BG, 0xFFFFFFFF);
        assert_eq!(ZINC_FG, 0xFF09090B);
        assert_eq!(ZINC_PRIMARY, 0xFF09090B);
        assert_eq!(ZINC_SECONDARY, 0xFFF4F4F5);
        assert_eq!(ZINC_MUTED_FG, 0xFF71717A);
        assert_eq!(ZINC_BORDER, 0xFFE4E4E7);
    }

    #[test]
    fn rem_geometry_matches_tailwind() {
        assert_eq!(BTN_WEB, 36.0);
        assert_eq!(BTN_HEIGHT, 48.0);
        assert_eq!(BTN_HEIGHT_SM, 36.0);
        assert_eq!(BTN_HEIGHT_LG, 56.0);
        assert_eq!(DIALOG_MAX_W, 512.0);
        assert_eq!(DIALOG_PAD, 24.0);
        assert_eq!(RADIUS_MD, 6.0);
        assert_eq!(RADIUS_LG, 8.0);
        assert_eq!(TEXT_SM, 14.0);
        assert_eq!(OVERLAY, 0x80000000);
        assert_eq!(SHEET_W, 384.0);
    }
}
