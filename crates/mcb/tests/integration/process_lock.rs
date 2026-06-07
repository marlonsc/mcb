use std::fs::{File, OpenOptions};

use fs2::FileExt;

pub struct ProcessLock {
    file: File,
}

impl ProcessLock {
    pub fn acquire() -> Result<Self, Box<dyn std::error::Error>> {
        let path = std::env::temp_dir().join("mcb-stdio-integration.lock");
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)?;
        file.lock_exclusive()?;
        Ok(Self { file })
    }
}

impl Drop for ProcessLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
