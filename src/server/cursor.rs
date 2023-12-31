use anyhow::{anyhow, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

/// Some methods to move back and forth in a vec of Paths
pub struct PathCursor {
    paths: Vec<PathBuf>,
    /// current index position. it will be None until next() is called
    index: Option<usize>,
    len: usize,
}

impl PathCursor {
    /// initialized a PathCursor from a vec of PathBufs
    pub fn new(items: Vec<PathBuf>) -> Self {
        let index = None;
        let len = items.len();
        Self {
            paths: items,
            index,
            len,
        }
    }
    /// get current index then advance
    pub fn next(&mut self) -> Option<&PathBuf> {
        // if not None use the index, else set it to 0
        let index = self.index.map_or_else(|| 0, |i| i);

        // if get returns an item, it must be a valid index. Otherwise cycle
        // get the first image and set the index for the next call to 1
        let path = if let Some(path) = self.paths.get(index) {
            self.index = Some(index + 1);
            Some(path)
        } else {
            self.index = Some(1);
            self.paths.first()
        };
        path
    }
    /// get previous
    pub fn prev(&mut self) -> Option<&PathBuf> {
        // if not None use the index, else set it to 0
        let mut index = self.index.map_or_else(|| 0, |i| i);
        let path = if index == 0 {
            index = self.len - 1;
            self.paths.last()
        } else {
            index -= 1;
            self.paths.get(index)
        };
        self.index = Some(index);
        path
    }
    /// remove
    pub fn remove(&mut self) -> Option<PathBuf> {
        if let Some(index) = self.index {
            self.len -= 1;
            let p = self.paths.remove(index);
            Some(p)
        } else {
            None
        }
    }

    /// Import all the files under given dir path, performing some sanity checks.
    pub fn import_files(path: &Path) -> Result<Self> {
        let read_dir = std::fs::read_dir(path).map_err(|e| anyhow!("Get Path Error {}", e))?;
        let mut files = read_dir
            .into_iter()
            .par_bridge()
            // filter out i/o errors
            .filter_map(|x| x.ok())
            .map(|x| x.path())
            // filter out directories
            .filter(|x| x.file_name().is_some())
            .collect::<Vec<PathBuf>>();

        if files.is_empty() {
            return Err(anyhow!("no files found in image directory"));
        }
        files.par_sort_unstable_by(|a, b| a.file_name().cmp(&b.file_name()));

        Ok(Self::new(files))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    fn get_paths() -> Vec<PathBuf> {
        let v = vec![
            Path::new("./foo/bar.txt"),
            Path::new("./bim/bam.txt"),
            Path::new("./bar/foo.txt"),
        ];
        v.iter().map(|i| i.to_path_buf()).collect()
    }

    #[test]
    fn test_cursor_next() -> Result<()> {
        let p = get_paths();
        let mut v = PathCursor::new(p);
        assert_eq!(v.next(), Some(&Path::new("./foo/bar.txt").to_path_buf()));
        assert_eq!(v.next(), Some(&Path::new("./bim/bam.txt").to_path_buf()));
        assert_eq!(v.next(), Some(&Path::new("./bar/foo.txt").to_path_buf()));
        assert_eq!(v.next(), Some(&Path::new("./foo/bar.txt").to_path_buf()));
        assert_eq!(v.next(), Some(&Path::new("./bim/bam.txt").to_path_buf()));
        Ok(())
    }
    #[test]
    fn test_cursor_prev() -> Result<()> {
        let p = get_paths();
        let mut v = PathCursor::new(p);
        assert_eq!(v.prev(), Some(&Path::new("./bar/foo.txt").to_path_buf()));
        assert_eq!(v.prev(), Some(&Path::new("./bim/bam.txt").to_path_buf()));
        assert_eq!(v.prev(), Some(&Path::new("./foo/bar.txt").to_path_buf()));
        assert_eq!(v.prev(), Some(&Path::new("./bar/foo.txt").to_path_buf()));
        Ok(())
    }
}
