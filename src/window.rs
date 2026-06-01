use std::{fs, path::PathBuf};

use anyhow::Result;
use sdl3::{event::Event, pixels::Color, pixels::PixelFormat, surface::Surface};

use crate::components::{Component, core::body::Body};

pub struct Window {
    icon: PathBuf,
    title: String,
    viewport: (u32, u32),
}

impl Window {
    pub fn new(title: &str, dimensions: (u32, u32), icon: PathBuf) -> Self {
        Self {
            icon,
            title: title.to_string(),
            viewport: dimensions,
        }
    }

    pub fn build(&self, root: Body) -> Result<()> {
        let sdl = sdl3::init()?;
        let video = sdl.video()?;

        let mut window = video
            .window(&self.title, self.viewport.0, self.viewport.1)
            .position_centered()
            .resizable()
            .build()?;

        let icon = fs::read(&self.icon)?;
        let img = image::load_from_memory(&icon)?.into_rgba8();
        let (w, h) = img.dimensions();
        let mut pixels = img.into_raw();
        window.set_icon(&Surface::from_data(
            &mut pixels,
            w,
            h,
            w * 4,
            PixelFormat::ABGR8888,
        )?);

        let mut canvas = window.into_canvas();
        let mut events = sdl.event_pump()?;

        'main: loop {
            for event in events.poll_iter() {
                if let Event::Quit { .. } = event {
                    break 'main;
                }
            }

            canvas.set_draw_color(Color::BLACK);
            canvas.clear();
            root.render(&mut canvas);
            canvas.present();
        }

        Ok(())
    }
}
