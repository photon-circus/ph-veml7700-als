use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub struct EvidenceWriter {
    path: PathBuf,
    enabled: bool,
}

impl EvidenceWriter {
    pub fn new(enabled: bool, path: PathBuf) -> Result<Self> {
        if enabled {
            if let Some(dir) = path.parent() {
                if dir.exists() {
                    fs::remove_dir_all(dir)
                        .with_context(|| format!("failed to reset {}", dir.display()))?;
                }
                fs::create_dir_all(dir)
                    .with_context(|| format!("failed to create {}", dir.display()))?;
            }
            File::create(&path).with_context(|| format!("failed to create {}", path.display()))?;
        }
        Ok(Self { path, enabled })
    }

    pub fn line(&self, text: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.path)
            .with_context(|| format!("failed to open {}", self.path.display()))?;
        writeln!(file, "{text}")
            .with_context(|| format!("failed to write {}", self.path.display()))?;
        Ok(())
    }

    pub fn lines(&self, lines: &[String]) -> Result<()> {
        for line in lines {
            self.line(line)?;
        }
        Ok(())
    }

    pub fn blank(&self) -> Result<()> {
        self.line("")
    }

    pub fn raw(&self, text: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.path)
            .with_context(|| format!("failed to open {}", self.path.display()))?;
        file.write_all(text.as_bytes())
            .with_context(|| format!("failed to write {}", self.path.display()))?;
        if !text.ends_with('\n') {
            writeln!(file).with_context(|| format!("failed to write {}", self.path.display()))?;
        }
        Ok(())
    }

    pub fn relative_to(&self, repo_root: &Path) -> String {
        self.path
            .strip_prefix(repo_root)
            .unwrap_or(&self.path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}
