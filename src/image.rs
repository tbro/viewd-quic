use std::path::Path;
use std::{path::PathBuf, ffi::OsString};
use std::fs;
use anyhow::{anyhow, Result};
use tracing::debug;

// TODO use this!

/// Representation of an image
pub struct Image {
    path: PathBuf,
    name: OsString,
    file_name: OsString
}

impl Image {
    pub fn new(path: &Path) -> Option<Image> {
	let name = path.file_stem()?.to_owned();
	let file_name = path.file_name()?.to_owned();
	let path = path.to_path_buf();
	let i = Image { path, name, file_name };
	Some(i)
    } 
    pub fn path(&self) -> PathBuf {
        let path = &self.path;
	path.to_path_buf()
    }
    pub fn name(&self) -> OsString {
        self.name.clone()
    }
    pub fn file_name(&self) -> OsString {
	self.file_name.clone()
    }
    pub fn data(&self) -> Result<Vec<u8>> {
        let path = &self.path;
	debug!("opening file at {:?}", path);
	fs::read(path).map_err(|e|anyhow!("Image Data Error: {}", e))
    }
}
