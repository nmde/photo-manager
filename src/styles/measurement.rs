#[derive(Clone, Copy)]
pub enum Measurement {
    Px(i32),
    Rem(f32),
    Percent(f32),
    Em(f32),
}

impl Measurement {
    pub fn resolve(self, viewport: u32) -> u32 {
        match self {
            Measurement::Px(px) => px.max(0) as u32,
            Measurement::Percent(pct) => (viewport as f32 * pct) as u32,
            Measurement::Rem(_) | Measurement::Em(_) => 0,
        }
    }
}
