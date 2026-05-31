use sdl3::{rect::Rect, render::Canvas, video::Window};

use crate::styles::Style;

pub mod core;

pub struct ComponentBase {
    style: Style,
}

impl ComponentBase {
    pub fn new() -> Self {
        Self {
            style: Style::new(),
        }
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) {
        let window_size = canvas.window().size();
        if let Some(color) = self.style.background_color {
            let w = self.style.width.map_or(0, |m| m.resolve(window_size.0));
            let h = self.style.height.map_or(0, |m| m.resolve(window_size.1));
            canvas.set_draw_color(color);
            canvas.fill_rect(Rect::new(0, 0, w, h)).unwrap();
        }
    }
}

pub trait Component {
    fn base(&self) -> &ComponentBase;
    fn base_mut(&mut self) -> &mut ComponentBase;

    fn render(&self, canvas: &mut Canvas<Window>) {
        self.base().render(canvas);
    }

    fn style(&mut self, style: &Style) {
        self.base_mut().style = self.base().style.merge(style);
    }
}
