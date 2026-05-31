#[derive(Clone, Copy)]
pub enum Timing {
    CubicBezier(f32, f32, f32, f32),
    Ms(i32),
}
