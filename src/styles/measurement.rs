use std::ops::Sub;

#[derive(Clone, Copy)]
pub enum Measurement {
    Px(i32),
    Rem(f32),
    Percent(f32),
    Em(f32),
    Calc(fn((u32, u32)) -> u32),
    Vh(f32),
    Vw(f32),
}

impl Measurement {
    pub fn resolve(self, viewport: (u32, u32)) -> u32 {
        match self {
            Measurement::Px(px) => px.max(0) as u32,
            Measurement::Percent(pct) => (viewport.0 as f32 * pct) as u32, // TODO this needs to be relative to the parent
            Measurement::Vw(pct) => (viewport.0 as f32 * pct) as u32,
            Measurement::Vh(pct) => (viewport.1 as f32 * pct) as u32,
            Measurement::Rem(_) | Measurement::Em(_) => 0, // TODO
            Measurement::Calc(f) => (f)(viewport),
        }
    }
}
