use anyhow::{anyhow, Result};
use sdl2::EventPump;
use sdl2::event::EventPollIterator;
use sdl2::image::LoadTexture;
use sdl2::{render::WindowCanvas, video::FullscreenType};
use std::path::Path;
use std::ffi::OsString;

pub struct Window {
    rotation: f64,
    fullscreen: FullscreenType,
    pub pageant_mode: bool,
    pub pageant_ready: bool,
    pub canvas: WindowCanvas,
    window_title: OsString,
    event_pump: EventPump
}

impl Window {
    pub fn new(title: OsString) -> Result<Self> {
	let sdl_context = sdl2::init()
            .map_err(|e| anyhow!("Navigator init Error: {}", e))?;
	let event_pump = sdl_context.event_pump()
            .map_err(|e| anyhow!("Navigator init Error: {}", e))?;
        let fullscreen = FullscreenType::Off;
        let rotation: f64 = 0.0;
        let pageant_mode = false;
        let pageant_ready = false;
        let video_subsystem = sdl_context
            .video()
            .map_err(|e| anyhow!("Navigator init Error: {}", e))?;
        let window = video_subsystem
            .window("viewd", 800, 600)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| anyhow!("Navigator init Error: {}", e))?;

        let canvas = window
            .into_canvas()
            .present_vsync()
            .software()
            .target_texture()
            .build()
            .map_err(|e| anyhow!("Navigator init Error: {}", e))?;

        let s = Self {
            fullscreen,
            rotation,
            pageant_mode,
            canvas,
            window_title: title,
            pageant_ready,
	    event_pump
        };

        Ok(s)
    }
    pub fn poll_events(&mut self) -> EventPollIterator {
	self.event_pump.poll_iter()
    }
    /// wraps update methods
    pub fn update(&mut self, image: &Path) -> Result<()> {
        self.update_canvas(image)?;
        self.update_window()?;
        self.update_title(image);
        Ok(())
    }
    pub fn update_title(&mut self, image: &Path) {
        if let Some(name) = image.file_name() {
            self.window_title = name.to_owned();
        }
    }
    pub fn update_canvas(&mut self, image: &Path) -> Result<()> {
        self.canvas.clear();
        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator
            .load_texture(image.clone())
            .map_err(|e| anyhow!("Update Canvas Error: {}", e))?;
        self.canvas
            .copy_ex(
                &texture,
                None,
                None,
                self.rotation * -90_f64,
                None,
                false,
                false,
            )
            .map_err(|e| anyhow!("Update Canvas Error: {}", e))?;
        self.canvas.present();
        Ok(())
    }
    pub fn update_window(&mut self) -> Result<()> {
        let window = self.canvas.window_mut();
        window.set_fullscreen(self.fullscreen).unwrap();
        window
            .set_title(self.window_title.to_str().unwrap())
            .map_err(|e| anyhow!("Update Window Error: {}", e))?;
        Ok(())
    }
    pub fn fullscreen_toggle(&mut self, image: &Path) -> Result<()> {
        match self.fullscreen {
            FullscreenType::Off => self.fullscreen = FullscreenType::Desktop,
            FullscreenType::True => self.fullscreen = FullscreenType::Off,
            FullscreenType::Desktop => self.fullscreen = FullscreenType::Off,
        };

        let window = self.canvas.window_mut();
        window
            .set_fullscreen(self.fullscreen)
            .map_err(|e| anyhow!("FullScreen Toggle Error: {}", e))?;
        self.update_canvas(image)?;
        Ok(())
    }
    pub fn pageant_toggle(&mut self) {
        self.pageant_mode = !self.pageant_mode;
    }
    pub fn rotate(&mut self, f: f64, image: &Path) -> Result<()> {
        self.rotation += f;
        self.update_canvas(image)?;
        Ok(())
    }
    pub fn try_load(&mut self, image: &Path) -> Option<()> {
        let texture_creator = self.canvas.texture_creator();
        texture_creator.load_texture(image).ok().map(|_| ())
    }
}
