use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Result};
use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use tracing::debug;

use crate::{
    model::{Request, Response, ServerCommand},
    server::navigator::Navigator,
    server::pageant::PageantMode,
    server::window::Window,
};

/// Issues commands from the network to the Navigator and Window.
pub struct Controller {
    /// Navigator holds a cursor for moving through list of Image files
    nav: Navigator,
    /// Window holds Sdl window and update functions
    win: Window,
    /// Channel to transmit response back to the network service
    tx_res: Sender<Bytes>,
    /// Channel to recieve request from network service
    rx_req: Receiver<Bytes>,
    /// A mutext to exit tasks gracefully
    exiting: Arc<Mutex<bool>>,
    /// If enabled, display will automatically update periodically
    pageant: PageantMode,
}

impl Controller {
    pub fn new(
        path: &Path,
        rx_req: Receiver<Bytes>,
        tx_res: Sender<Bytes>,
        exiting: Arc<Mutex<bool>>,
    ) -> Result<Self> {
        let nav = Navigator::new(path)?;
        let win = Window::new(OsStr::new("viewd").to_owned())?;
        let pageant = PageantMode::new();
        let c = Controller {
            nav,
            win,
            tx_res,
            rx_req,
            exiting,
            pageant,
        };
        Ok(c)
    }
    pub fn next(&mut self) -> Result<()> {
        let _result = self.handle_command(ServerCommand::Next);
        Ok(())
    }
    pub fn _prev(&mut self) -> Result<()> {
        let _result = self.handle_command(ServerCommand::Prev);
        Ok(())
    }
    /// Parse request and send response back to que Quic Service
    pub fn handle_request(&mut self) -> Result<()> {
        while let Ok(bytes) = self.rx_req.try_recv() {
            let request = Request::from_bytes(bytes)?;
            debug!("request: {:?}", request);

            let p = self.nav.image_path();
            let resp = match self.handle_command(request.command()) {
                Ok(maybe_data) => {
                    let message = "Success";
                    Response::new(Some(p), maybe_data, message)
                }
                Err(e) => {
                    let message = format! {"Error: {}", e};
                    Response::new(Some(p), None, &message)
                }
            };
            self.tx_res.send(resp.to_bytes()?.into())?;
        }
        Ok(())
    }
    /// Call navigator and window commands according to network request.
    pub fn handle_command(&mut self, command: ServerCommand) -> Result<Option<Vec<u8>>> {
        match command {
            ServerCommand::Fetch => {
                let data = self.nav.image_data()?;
                Ok(Some(data))
            }
            ServerCommand::Fullscreen => {
                self.win.fullscreen_toggle(&self.nav.image)?;
                Ok(None)
            }
            ServerCommand::Rotate => {
                self.win.rotate(1.0, &self.nav.image)?;
                Ok(None)
            }
            ServerCommand::Pageant => {
                self.pageant.toggle();
                Ok(None)
            }
            ServerCommand::Next => {
                // loop until we get a supported image. Test if image
                // is supported by loading it in the window.
                let image = loop {
                    let image = self
                        .nav
                        .next()
                        .ok_or(anyhow!("Controller: image get error"))?;
                    if let Some(_t) = self.win.try_load(image) {
                        break image;
                    } else {
                        self.nav.delete();
                    }
                };

                self.win.update(image)?;
                Ok(None)
            }
            ServerCommand::Prev => {
                // loop until we get a supported image. Test if image
                // is supported by loading it in the window.
                let image = loop {
                    let image = self
                        .nav
                        .prev()
                        .ok_or(anyhow!("Controller: image get error"))?;
                    if let Some(_t) = self.win.try_load(image) {
                        break image;
                    } else {
                        self.nav.delete();
                    }
                };
                self.win.update(image)?;
                Ok(None)
            }
        }
    }
    /// Handle Window events
    pub fn handle_events(&mut self) {
        for event in self.win.poll_events() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape) | Some(Keycode::Q),
                    ..
                } => *self.exiting.lock().unwrap() = true,
                _ => {}
            };
        }
    }
    /// Update image if we in pageant mode and timeout has elapsed
    pub fn pageant(&mut self) {
        if self.pageant.should_update() {
            self.pageant.set_instant();
            let _ = self.next();
        };
    }
}
