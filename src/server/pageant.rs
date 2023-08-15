use std::time::{Duration, Instant};

/// A type to represent pageant mode
#[derive(Debug, Copy, Clone)]
pub struct PageantMode {
    /// milliseconds each image will remain visible
    timeout: u64,
    /// time of last update
    instant: Option<Instant>,
}

impl PageantMode {
    pub fn new() -> Self {
        Self {
            timeout: 1000,
            instant: None,
        }
    }
    pub fn toggle(&mut self) {
        if let Some(_instant) = self.instant {
            self.instant = None;
        } else {
            self.instant = Some(Instant::now());
        }
    }
    pub fn set_instant(&mut self) {
        self.instant = Some(Instant::now());
    }
    pub fn should_update(&self) -> bool {
        if let Some(instant) = self.instant {
            Instant::now() - instant >= Duration::from_millis(self.timeout)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    struct MockController {
        pageant: PageantMode,
    }

    impl MockController {
        pub fn new() -> Self {
            let pageant = PageantMode::new();
            Self { pageant }
        }
    }

    #[test]
    fn toggle_pageant() -> Result<()> {
        let mut pageant = PageantMode::new();
        assert!(pageant.instant.is_none());
        pageant.toggle();
        assert!(pageant.instant.is_some());
        Ok(())
    }

    #[test]
    fn nested_update_pageant() -> Result<()> {
        let mut control = MockController::new();
        assert!(control.pageant.instant.is_none());
        control.pageant.toggle();
        assert!(control.pageant.instant.is_some());
        Ok(())
    }
}
