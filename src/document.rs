use std::fs;
use std::io;
use std::path::PathBuf;

use chrono::Local;

pub struct Document {
    path: PathBuf,
    modified: bool,
}

impl Document {
    pub fn new(documents_dir: PathBuf) -> Self {
        let now = Local::now();
        let date_folder = now.format("%Y-%m-%d").to_string();
        let filename = format!("{}.md", now.format("%Y-%m-%d_%H-%M-%S"));
        let dir = documents_dir.join(&date_folder);
        let _ = fs::create_dir_all(&dir);
        Self {
            path: dir.join(filename),
            modified: false,
        }
    }

    pub fn filename(&self) -> String {
        self.path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled.md")
            .to_string()
    }

    pub fn mark_modified(&mut self) {
        self.modified = true;
    }

    pub fn save(&mut self, content: &str) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, content)?;
        self.modified = false;
        Ok(())
    }

}
