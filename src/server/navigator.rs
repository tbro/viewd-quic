use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tracing::debug;

use crate::server::cursor::PathCursor;

/// Navigator holds the list of images and
/// methods to move through them.
pub struct Navigator {
    pub cursor: PathCursor,
    pub image: PathBuf,
}

impl Navigator {
    pub fn new(path: &Path) -> Result<Self> {
	let mut cursor = PathCursor::import_files(path)?;
	let image = if let Some(path) = cursor.next() {
	    path.to_path_buf()
	} else {
	    panic!("no images found");
	};
	let n = Self {
	    cursor,
	    image,
	};
	Ok(n)
    }
    /// advance the cursor and return current
    pub fn next(&mut self) -> Option<&Path> {
	let path  = self.cursor.next()?;
	self.image = path.to_path_buf();
	Some(path)
    }
    /// opposite of next
    pub fn prev(&mut self) -> Option<&Path> {
	let path  = self.cursor.prev()?;
	self.image = path.to_path_buf();
	Some(path)
    }
    /// remove current cursor path from list
    pub fn delete(&mut self) {
	self.cursor.remove();
    }
    pub fn image_path(&self) -> PathBuf {
	let path = &self.image;
	path.to_path_buf()
    }
    pub fn image_data(&self) -> Result<Vec<u8>> {
	let path = &self.image;
	debug!("opening file at {:?}", path);
	fs::read(path).map_err(|e| anyhow!("Image Data Error: {}", e))
    }
}
