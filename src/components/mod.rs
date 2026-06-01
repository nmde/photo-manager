use sdl3::{rect::Rect, render::Canvas, video::Window};

use crate::styles::{Style, color::Color};

pub mod core;
pub mod md;

pub struct ComponentBase {
    condition: Option<fn() -> bool>,
    style: Style,
}

impl ComponentBase {
    pub fn new() -> Self {
        Self {
            condition: None,
            style: Style::new(),
        }
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) {
        let render_size = canvas
            .output_size()
            .unwrap_or_else(|_| canvas.window().size());
        let color = self.style.background_color.unwrap_or(Color::black());
        let w = self.style.width.map_or(0, |m| m.resolve(render_size));
        let h = self.style.height.map_or(0, |m| m.resolve(render_size));
        canvas.set_draw_color(color);
        canvas.fill_rect(Rect::new(0, 0, w, h)).unwrap();
    }
}

pub trait Component {
    fn base(&self) -> &ComponentBase;
    fn base_mut(&mut self) -> &mut ComponentBase;

    fn cond(&mut self, test: fn() -> bool) -> &mut Self
    where
        Self: Sized,
    {
        self.base_mut().condition = Some(test);
        self
    }

    fn render(&self, canvas: &mut Canvas<Window>) {
        self.base().render(canvas);
    }

    fn style(&mut self, style: &Style) -> &mut Self
    where
        Self: Sized,
    {
        self.base_mut().style = self.base().style.merge(style);
        self
    }
}
