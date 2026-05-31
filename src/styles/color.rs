use std::f32::consts::{FRAC_PI_2, PI};

// --- const math primitives ---

const fn reduce_angle(x: f32) -> f32 {
    const TAU: f32 = PI * 2.0;
    let mut x = x;
    while x > PI {
        x -= TAU;
    }
    while x < -PI {
        x += TAU;
    }
    x
}

const fn sin_f32(x: f32) -> f32 {
    let x = reduce_angle(x);
    // Exploit symmetry to keep x in [0, π/2] for good Taylor convergence
    let (x, neg) = if x < 0.0 { (-x, true) } else { (x, false) };
    let x = if x > FRAC_PI_2 { PI - x } else { x };
    let x2 = x * x;
    // 7-term Horner form; error < 2.4e-9 for x ∈ [0, π/2]
    let y = x
        * (1.0
            - x2 / 6.0
                * (1.0
                    - x2 / 20.0
                        * (1.0
                            - x2 / 42.0
                                * (1.0
                                    - x2 / 72.0
                                        * (1.0
                                            - x2 / 110.0
                                                * (1.0 - x2 / 156.0 * (1.0 - x2 / 210.0)))))));
    if neg { -y } else { y }
}

const fn cos_f32(x: f32) -> f32 {
    sin_f32(FRAC_PI_2 - x)
}

const fn sqrt_f32(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut y = x;
    let mut i = 0;
    while i < 10 {
        y = 0.5 * (y + x / y);
        i += 1;
    }
    y
}

const fn cbrt_f32(x: f32) -> f32 {
    if x == 0.0 {
        return 0.0;
    }
    let mut y = 1.0_f32; // y = x gives poor convergence for small x
    let mut i = 0;
    while i < 15 {
        y = (2.0 * y + x / (y * y)) / 3.0;
        i += 1;
    }
    y
}

// x^(5/12) = x^(1/4) * x^(1/6) = sqrt(sqrt(x)) * cbrt(sqrt(x))
const fn pow5_12(x: f32) -> f32 {
    let sq = sqrt_f32(x);
    sqrt_f32(sq) * cbrt_f32(sq)
}

// IEC 61966-2-1 sRGB gamma encoding, input clamped to [0, 1]
const fn gamma_encode(x: f32) -> f32 {
    let x = if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    };
    if x <= 0.0031308 {
        12.92 * x
    } else {
        1.055 * pow5_12(x) - 0.055
    }
}

// ---

#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Converts OKLCH to sRGB, matching the CSS `oklch(L C H)` function.
    /// L: lightness 0.0–1.0 (CSS 0%–100%), C: chroma 0.0–~0.4, H: hue in degrees.
    pub const fn oklch(l: f32, c: f32, h: f32) -> Self {
        const DEG_TO_RAD: f32 = PI / 180.0;
        let h_rad = h * DEG_TO_RAD;
        let ok_a = c * cos_f32(h_rad);
        let ok_b = c * sin_f32(h_rad);

        // OKLab → LMS (cube-root space)
        let l_ = l + 0.3963377774 * ok_a + 0.2158037573 * ok_b;
        let m_ = l - 0.1055613458 * ok_a - 0.0638541728 * ok_b;
        let s_ = l - 0.0894841775 * ok_a - 1.2914855480 * ok_b;

        let l3 = l_ * l_ * l_;
        let m3 = m_ * m_ * m_;
        let s3 = s_ * s_ * s_;

        // Linear LMS → linear sRGB
        let r_lin = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
        let g_lin = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
        let b_lin = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;

        Self {
            r: (gamma_encode(r_lin) * 255.0 + 0.5) as u8,
            g: (gamma_encode(g_lin) * 255.0 + 0.5) as u8,
            b: (gamma_encode(b_lin) * 255.0 + 0.5) as u8,
            a: 255,
        }
    }
}

impl From<Color> for sdl3::pixels::Color {
    fn from(c: Color) -> sdl3::pixels::Color {
        sdl3::pixels::Color::RGBA(c.r, c.g, c.b, c.a)
    }
}
