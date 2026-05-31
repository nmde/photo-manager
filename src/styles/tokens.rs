use crate::styles::{color::Color, measurement::Measurement, timing::Timing};

/* Observatory design token layer — canonical OKLCH values */

/* Color primitives */
pub const COLOR_PRIMARY: Color = Color::oklch(0.65, 0.14, 245.0);
pub const COLOR_PRIMARY_HOVER: Color = Color::oklch(0.72, 0.14, 245.0);
pub const COLOR_PRIMARY_ACTIVE: Color = Color::oklch(0.57, 0.14, 245.0);
pub const COLOR_BACKGROUND: Color = Color::oklch(0.09, 0.003, 245.0);
pub const COLOR_SURFACE: Color = Color::oklch(0.14, 0.004, 245.0);
pub const COLOR_SURFACE_RAISED: Color = Color::oklch(0.21, 0.006, 245.0);
pub const COLOR_BORDER: Color = Color::oklch(0.3, 0.008, 245.0);
pub const COLOR_TEXT_PRIMARY: Color = Color::oklch(0.94, 0.006, 245.0);
pub const COLOR_TEXT_SECONDARY: Color = Color::oklch(0.58, 0.006, 245.0);
pub const COLOR_ERROR: Color = Color::oklch(0.58, 0.006, 14.0);
pub const COLOR_WARNING: Color = Color::oklch(0.72, 0.16, 60.0);

/* Motion */
pub const EASE_STANDARD: Timing = Timing::CubicBezier(0.2, 0.0, 0.0, 1.0);
pub const EASE_OUT_EXPO: Timing = Timing::CubicBezier(0.16, 1.0, 0.3, 1.0);
pub const DURATION_FAST: Timing = Timing::Ms(100);
pub const DURATION_STANDARD: Timing = Timing::Ms(150);

/* Typography scale */
pub const TEXT_SIZE_LABEL: Measurement = Measurement::Rem(0.75);
pub const TEXT_SIZE_BODY: Measurement = Measurement::Rem(0.875);
pub const TEXT_SIZE_TITLE: Measurement = Measurement::Rem(1.0);
pub const TEXT_SIZE_HEADLINE: Measurement = Measurement::Rem(1.25);

pub const TEXT_WEIGHT_REGULAR: i32 = 400;
pub const TEXT_WEIGHT_MEDIUM: i32 = 500;
pub const TEXT_WEIGHT_SEMIBOLD: i32 = 600;
pub const TEXT_WEIGHT_BOLD: i32 = 700;

pub const TEXT_LH_TIGHT: Measurement = Measurement::Percent(1.3);
pub const TEXT_LG_SNUG: Measurement = Measurement::Percent(1.4);
pub const TEXT_LG_BODY: Measurement = Measurement::Percent(1.55);

pub const TEXT_TRACKING_TIGHT: Measurement = Measurement::Em(0.01);
pub const TEXT_TRACKING_BODY: Measurement = Measurement::Em(0.015);
pub const TEXT_TRACKING_LABEL: Measurement = Measurement::Em(0.04);

/* Spacing scale */
pub const SPACE_XS: Measurement = Measurement::Px(4);
pub const SPACE_SM: Measurement = Measurement::Px(8);
pub const SPACE_MD: Measurement = Measurement::Px(16);
pub const SPACE_LG: Measurement = Measurement::Px(24);
pub const SPACE_XL: Measurement = Measurement::Px(40);
pub const SPACE_2XL: Measurement = Measurement::Px(64);
