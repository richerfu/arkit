//! ColorUI design tokens with a 1:1 citation to weilanwl/ColorUI `main.css`.
//!
//! ColorUI sizes are `upx` on a 750-wide design. We convert `vp = upx / 2`
//! (375-wide phone). Hex values are copied verbatim from `bg-*` / `text-*`.

/// 750-design `upx` → ArkUI vp.
pub const fn upx(value: f32) -> f32 {
    value / 2.0
}

// --- palette: `.bg-red` … `.bg-white` ------------------------------------

pub const BG_RED: u32 = 0xFFE54D42;
pub const BG_ORANGE: u32 = 0xFFF37B1D;
pub const BG_YELLOW: u32 = 0xFFFBBD08;
pub const BG_OLIVE: u32 = 0xFF8DC63F;
pub const BG_GREEN: u32 = 0xFF39B54A;
pub const BG_CYAN: u32 = 0xFF1CBBB4;
pub const BG_BLUE: u32 = 0xFF0081FF;
pub const BG_PURPLE: u32 = 0xFF6739B6;
pub const BG_MAUVE: u32 = 0xFF9C26B0;
pub const BG_PINK: u32 = 0xFFE03997;
pub const BG_BROWN: u32 = 0xFFA5673F;
pub const BG_GREY: u32 = 0xFF8799A3;
pub const BG_GRAY: u32 = 0xFFF0F0F0;
pub const BG_BLACK: u32 = 0xFF333333;
pub const BG_WHITE: u32 = 0xFFFFFFFF;
pub const INK_ON_YELLOW: u32 = 0xFF333333;
pub const INK_ON_GRAY: u32 = 0xFF333333;
pub const INK_ON_WHITE: u32 = 0xFF666666;
pub const INK_ON_FILL: u32 = 0xFFFFFFFF;

pub const LIGHT_RED: u32 = 0xFFFADBD9;
pub const LIGHT_GREEN: u32 = 0xFFD7F0DB;
pub const LIGHT_BLUE: u32 = 0xFFCCE6FF;

pub const PAGE_BG: u32 = 0xFFF1F1F1;
pub const TEXT: u32 = 0xFF333333;
pub const TEXT_MUTED: u32 = 0xFF888888;
pub const TEXT_GREY: u32 = 0xFF8799A3;
pub const HAIRLINE: u32 = 0xFFDDDDDD;
pub const FORM_LINE: u32 = 0xFFEEEEEE;
pub const PROGRESS_TRACK: u32 = 0xFFEBEEF5;
pub const DIALOG_FILL: u32 = 0xFFF8F8F8;
pub const OVERLAY: u32 = 0x99000000;
pub const SWITCH_OFF: u32 = 0xFF8799A3;
pub const AVATAR_FALLBACK: u32 = 0xFFCCCCCC;
pub const SEARCH_BG: u32 = 0xFFF5F5F5;
pub const CHAT_INFO: u32 = 0x33000000;

// --- geometry: `.cu-btn` / `.cu-bar` / `.cu-dialog` / `.cu-progress` ------

pub const BTN_HEIGHT: f32 = upx(64.0);
pub const BTN_HEIGHT_SM: f32 = upx(48.0);
pub const BTN_HEIGHT_LG: f32 = upx(80.0);
pub const BTN_PAD: f32 = upx(30.0);
pub const BTN_PAD_SM: f32 = upx(20.0);
pub const BTN_PAD_LG: f32 = upx(40.0);
pub const BTN_FONT: f32 = upx(28.0);
pub const BTN_FONT_SM: f32 = upx(20.0);
pub const BTN_FONT_LG: f32 = upx(32.0);
pub const RADIUS: f32 = upx(6.0);
pub const RADIUS_CARD: f32 = upx(10.0);
pub const BAR_HEIGHT: f32 = upx(100.0);
pub const LIST_ITEM: f32 = upx(100.0);
pub const DIALOG_WIDTH: f32 = upx(680.0);
pub const PADDING: f32 = upx(30.0);
pub const PADDING_XL: f32 = upx(50.0);
pub const TAG_HEIGHT: f32 = upx(48.0);
pub const TAG_FONT: f32 = upx(24.0);
pub const AVATAR: f32 = upx(64.0);
pub const PROGRESS_HEIGHT: f32 = upx(28.0);
pub const SWITCH_W: f32 = 48.0;
pub const SWITCH_H: f32 = 26.0;
pub const CHECK_RADIO: f32 = 24.0;
pub const TEXT_XS: f32 = upx(20.0);
pub const TEXT_SM: f32 = upx(24.0);
pub const TEXT_DF: f32 = upx(28.0);
pub const TEXT_LG: f32 = upx(32.0);
pub const TEXT_XL: f32 = upx(36.0);
pub const TEXT_XXL: f32 = upx(44.0);
pub const NAV_ITEM: f32 = upx(90.0);
pub const SWIPER_DOT: f32 = upx(10.0);
pub const SWIPER_DOT_ACTIVE: f32 = upx(30.0);
/// `.basis-lg` drawer on a phone.
pub const DRAWER_WIDTH: f32 = 280.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_matches_colorui_css() {
        assert_eq!(BG_GREEN, 0xFF39B54A);
        assert_eq!(BG_RED, 0xFFE54D42);
        assert_eq!(BG_BLUE, 0xFF0081FF);
        assert_eq!(PAGE_BG, 0xFFF1F1F1);
        assert_eq!(TEXT, 0xFF333333);
        assert_eq!(PROGRESS_TRACK, 0xFFEBEEF5);
        assert_eq!(DIALOG_FILL, 0xFFF8F8F8);
        assert_eq!(OVERLAY, 0x99000000);
    }

    #[test]
    fn geometry_is_half_of_750_upx() {
        assert_eq!(BTN_HEIGHT, 32.0);
        assert_eq!(BTN_HEIGHT_SM, 24.0);
        assert_eq!(BTN_HEIGHT_LG, 40.0);
        assert_eq!(BAR_HEIGHT, 50.0);
        assert_eq!(DIALOG_WIDTH, 340.0);
        assert_eq!(RADIUS_CARD, 5.0);
        assert_eq!(PADDING, 15.0);
        assert_eq!(TEXT_DF, 14.0);
        assert_eq!(PROGRESS_HEIGHT, 14.0);
    }
}
