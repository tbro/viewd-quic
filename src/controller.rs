use std::{
    ffi::OsStr,
    path::{PathBuf, Path},
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Result};
use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use tracing::{debug, info};

use crate::{
    model::{Request, Response, ServerCommand},
    navigator::Navigator,
    window::Window,
};

pub struct Controller {
    /// Navigator holds a cursor for moving through list of Image files
    nav: Navigator,
    /// Window holds Sdl window and update functions
    win: Window,
    tx_res: Sender<Bytes>,
    rx_req: Receiver<Bytes>,
    exiting: Arc<Mutex<bool>>,
}
// if there is a Server struct, this might go there
impl Controller {
    pub fn new(
	path: &Path,
	rx_req: Receiver<Bytes>,
	tx_res: Sender<Bytes>,
	exiting: Arc<Mutex<bool>>,
    ) -> Result<Self> {
	let nav = Navigator::new(path)?;
	let win = Window::new(OsStr::new("viewd").to_owned())?;
	let c = Controller {
	    nav,
	    win,
	    tx_res,
	    rx_req,
	    exiting,
	};
	Ok(c)
    }
    pub fn next(&mut self) -> Result<()> {
	let _result = self.handle_command(ServerCommand::Next);
	Ok(())
    }
    pub fn prev(&mut self) -> Result<()> {
	let _result = self.handle_command(ServerCommand::Prev);
	Ok(())
    }
    /// Handle network request
    pub fn handle_request(&mut self) -> Result<()> {
	while let Ok(bytes) = self.rx_req.try_recv() {
	    let request = Request::from_bytes(bytes)?;
	    debug!("request: {:?}", request);
	    let cmd = request.command();

	    if let Ok((path, data, message)) = self.handle_command(request.command()) {
		let resp = Response::new(path, data, &message);
		debug!("response {:?}", resp);
		self.tx_res.send(resp.to_bytes()?.into())?;
	    }
	}
	Ok(())
    }
    /// Handle client originating commands
    pub fn handle_command(&mut self, command: ServerCommand) -> Result<(Option<PathBuf>, Option<Vec<u8>>, &str)> {
	let result = match command {
	    ServerCommand::Fetch => {
		let data = self.nav.image_data()?;
		(Some(self.nav.image_path()), Some(data), "Success")
	    }
	    ServerCommand::Fullscreen => {
		self.win.fullscreen_toggle(&self.nav.image)?;
		(Some(self.nav.image_path()), None, "Success")

	    },
	    ServerCommand::Rotate => {
		self.win.rotate(1.0, &self.nav.image)?;
		(Some(self.nav.image_path()), None, "Success")
	    },
	    ServerCommand::Pageant => {
		self.win.pageant_toggle();
		(Some(self.nav.image_path()), None, "Success")
	    },
	    ServerCommand::Next => {
		// loop until we get a supported image. Test if image
		// is supported by loading it in the window.
		let image = loop {
		    let image = self.nav.next()
			.ok_or(anyhow!("Controller: image get error"))?;
		    if let Some(_t) = self.win.try_load(&image) {
			break image
		    } else {
			self.nav.delete();
		    }
		};

		// self.update_window(&image)?;
		self.win.update(image)?;
		(Some(image.to_path_buf()), None, "Success")
	    }
	    ServerCommand::Prev => {
		// loop until we get a supported image. Test if image
		// is supported by loading it in the window.
		let image = loop {
		    let image = self.nav.prev()
			.ok_or(anyhow!("Controller: image get error"))?;
		    if let Some(_t) = self.win.try_load(&image) {
			break image
		    } else {
			self.nav.delete();
		    }
		};
		self.win.update(image)?;
		(Some(image.to_path_buf()), None, "Success")
	    }
	    _ => {
		info!["unknown command: {:?}", command];
		(None, None, "Error: Unknown Command")
	    }
	};

	Ok(result)
    }
    // /// update window with new image
    // fn update_window(&mut self, image: &Path) -> Result<()> {
    //	self.win.update(&image)
    // }
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
}
