use std::path::Path;
use std::path::PathBuf;
use std::fmt;
use bytes::Bytes;
use serde::{Serialize, Deserialize};
use tracing::debug;
use anyhow::{anyhow, Result};

use terminal_keycode::KeyCode;

// Model for commands sent to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    // Command to execute on the server.
    command: ServerCommand,
    // Path of image currently on display.
    // path: PathBuf
}

impl Request {
    pub fn new(code: KeyCode) -> Option<Request> {
	ServerCommand::from_keycode(code).map(|command| Request { command })
    }
    pub fn command(&self) -> ServerCommand {
	self.command
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
	bincode::serialize(&self)
            .map_err(|e| anyhow!("Serialization Error: {}", e))
    }
    pub fn from_bytes(bytes: Bytes) -> Result<Request> {
	bincode::deserialize(&bytes)
            .map_err(|e| anyhow!("Deserialization Error: {}", e))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    // Path of image currently on display. If Request updted image,
    // this will be the image display was updated to.
    path: Option<PathBuf>,
    // Success, Error, etc
    message: String,
    // Image data in the case of request was fetch
    bytes: Option<Vec<u8>>
}

impl Response {
    pub fn new(path: Option<PathBuf>, bytes: Option<Vec<u8>>, message: &str) -> Response {
	let message = message.to_string();
	Response { path, message, bytes }
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
	bincode::serialize(&self)
            .map_err(|e| anyhow!("Serialization Error: {}", e))
    }
    pub fn from_bytes(bytes: Bytes) -> Result<Response> {
	bincode::deserialize(&bytes)
            .map_err(|e| anyhow!("Deserialization Error: {}", e))
    }
    pub fn path(&self) -> Option<&Path> {
    	self.path.as_deref()
    }
    // pub fn image_name(&self) -> &[u8] {
    // 	let path = self.path.unwrap();
    // 	path.file_stem().unwrap().as_bytes()
    // }
}

/// Possible commands to execute on the Server.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ServerCommand {
    // Go back to the last image
    Prev,
    // Advance by one image
    Next,
    // Toggle Fullscreen
    Fullscreen,
    // rotate clockwise by 45%
    Rotate,
    // Update Image ever second
    Pageant,
    // Download image to client
    Fetch
}

impl ServerCommand {
    pub fn from_keycode(code: KeyCode) -> Option<Self> {
	match code {
	    KeyCode::Char('f') => Some(Self::Fullscreen),
	    KeyCode::Char('r') => Some(Self::Rotate),
	    KeyCode::Char('s') => Some(Self::Fetch),
	    KeyCode::Char('p') | KeyCode::Space => Some(Self::Pageant),
	    KeyCode::ArrowRight => Some(Self::Next),
	    KeyCode::ArrowLeft => Some(Self::Prev),
	    _ => {
		debug!("unhandled key");
		debug![
		    "loop code={:?} bytes={:?} printable={:?}\r\n",
		    code,
		    code.bytes(),
		    code.printable()
		];
		None
	    }
	}
    }
}

impl fmt::Display for ServerCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	match self {
	    Self::Fullscreen => write!(f, "Fullscreen"),
	    Self::Rotate => write!(f, "Rotate"),
	    Self::Fetch => write!(f, "Fetch"),
	    Self::Pageant => write!(f, "Pageant"),
	    Self::Next => write!(f, "Next"),
	    Self::Prev => write!(f, "Previous"),
	}
    }
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::*;

    #[test]
    fn test_serialize_response() -> Result<()> {
	let path = Path::new("/foo/bar.jpg").to_path_buf();
	let resp = Response::new(Some(path), None, "Success");
	let bytes = resp.to_bytes()?;
	let decoded = Response::from_bytes(bytes.into())?;
	assert_eq!(resp.path, decoded.path);
	Ok(())
    }

}
