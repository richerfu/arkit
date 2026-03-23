pub mod color {
    pub const BACKGROUND: u32 = 0xFFFFFFFF;
    pub const FOREGROUND: u32 = 0xFF0A0A0A;
    pub const CARD: u32 = 0xFFFFFFFF;
    pub const CARD_FOREGROUND: u32 = 0xFF0A0A0A;
    pub const POPOVER: u32 = 0xFFFFFFFF;
    pub const POPOVER_FOREGROUND: u32 = 0xFF0A0A0A;
    pub const PRIMARY: u32 = 0xFF171717;
    pub const PRIMARY_FOREGROUND: u32 = 0xFFFAFAFA;
    pub const PRIMARY_TRACK: u32 = 0x33171717;
    pub const SECONDARY: u32 = 0xFFF5F5F5;
    pub const SECONDARY_FOREGROUND: u32 = 0xFF171717;
    pub const MUTED: u32 = 0xFFF5F5F5;
    pub const MUTED_FOREGROUND: u32 = 0xFF737373;
    pub const ACCENT: u32 = 0xFFF5F5F5;
    pub const ACCENT_FOREGROUND: u32 = 0xFF171717;
    pub const DESTRUCTIVE: u32 = 0xFFEF4444;
    pub const DESTRUCTIVE_FOREGROUND: u32 = 0xFFFAFAFA;
    pub const BORDER: u32 = 0xFFE5E5E5;
    pub const INPUT: u32 = 0xFFE5E5E5;
    pub const RING: u32 = 0xFFA1A1A1;
    pub const SURFACE: u32 = 0xFFFFFFFF;
    pub const CHART_1: u32 = 0xFFE76E50;
    pub const CHART_2: u32 = 0xFF2A9D90;
    pub const CHART_3: u32 = 0xFF274754;
    pub const CHART_4: u32 = 0xFFE8C468;
    pub const CHART_5: u32 = 0xFFF4A462;
}

pub mod radius {
    // Keep these close to the Tailwind radii used by react-native-reusables:
    // `rounded`/small surfaces ~= 4, `rounded-md` = 6, `rounded-lg` = 8.
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 6.0;
    pub const LG: f32 = 8.0;
    pub const XL: f32 = 12.0;
    pub const XXL: f32 = 16.0;
    pub const FULL: f32 = 999.0;
}

pub mod spacing {
    pub const XXS: f32 = 4.0;
    pub const XS: f32 = 6.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 20.0;
    pub const XXL: f32 = 24.0;
}

pub mod typography {
    pub const XS: f32 = 12.0;
    pub const SM: f32 = 14.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 18.0;
    pub const XL: f32 = 20.0;
    pub const XXL: f32 = 24.0;
}
