use crate::ensure_directory;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Determines the type of iteration performed by the `list_directories` and `list_files` functions
/// If the NoRec variation is used, only the current directory is considered
/// If the Rec variation is used, then ALL subdirectores are traversed
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum FtIterItemState {
    /// Iterate files with no recursion
    FileNoRec,

    /// Iterate files with recursion
    FileRec,

    /// Iterate directories with no recursion
    DirNoRec,

    /// Iterate directories with recursion
    DirRec,
}

/// Helper for creating temp directories
///
/// Tempfile _would_ work but I want nested dirs and easy ways to create
/// a series of files / folder quickly without worrying
/// A cheap knock-off of `Tempfile` but meh, this works kinda better for my use case
pub(crate) struct TempPath {
    pub path: PathBuf,
}

// This is only used in the test suite
#[allow(dead_code)]
impl TempPath {
    pub async fn new(p: impl AsRef<Path>) -> Result<Self> {
        let root = std::env::temp_dir();
        let path = if p.as_ref().starts_with(&root) {
            p.as_ref().to_path_buf()
        } else {
            root.join(p)
        };

        ensure_directory(&path).await?;

        Ok(Self { path })
    }

    pub async fn new_file(&self, name: impl AsRef<Path>) -> Result<Self> {
        let p = self.path.join(name);
        tokio::fs::File::create(&p).await?;

        Self::new(p).await
    }

    pub async fn multi_file(&self, names: Vec<impl AsRef<Path>>) -> Result<()> {
        for name in names {
            tokio::fs::File::create(&self.path.join(name)).await?;
        }

        Ok(())
    }

    pub async fn new_folder(&self, name: impl AsRef<Path>) -> Result<Self> {
        let p = self.path.join(name);
        ensure_directory(&p).await?;

        Self::new(p).await
    }

    pub async fn multi_folder(&self, names: Vec<impl AsRef<Path>>) -> Result<()> {
        for name in names {
            ensure_directory(&self.path.join(name)).await?;
        }

        Ok(())
    }

    pub async fn nest_folders(&self, subfolder_chain: Vec<impl AsRef<Path>>) -> Result<Self> {
        let mut dst_path = self.path.clone();
        for sf in subfolder_chain {
            dst_path = dst_path.join(sf.as_ref());
        }

        ensure_directory(&dst_path).await?;
        Self::new(dst_path).await
    }

    pub fn join(&self, path: impl AsRef<Path>) -> impl AsRef<Path> {
        self.path.join(path)
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
