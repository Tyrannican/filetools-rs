use crate::{ensure_directory, path_contains, FtFilter};
use anyhow::{Context, Result};
use async_recursion::async_recursion;
use std::path::{Path, PathBuf};
use tokio::fs;

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

/// Helper function to determine if an path item is valid based on the supplied filter
fn matches_filter(item: impl AsRef<Path>, filter: &FtFilter) -> bool {
    match filter {
        // I know these are the same for Raw and Path
        // but it complains when you try and use the | with match
        // for this
        FtFilter::Raw(raw) => {
            if path_contains(&item, raw) {
                return true;
            }
        }
        FtFilter::Path(filter_path) => {
            if path_contains(&item, filter_path) {
                return true;
            }
        }
        FtFilter::Regex(re) => {
            if re.is_match(item.as_ref().to_str().unwrap()) {
                return true;
            }
        }
    }

    false
}

/// Helper function to iterate through a directory to find all Files / Directories
/// depending on the `FilterState` passed.
#[async_recursion]
pub(crate) async fn iteritems<P: AsRef<Path> + Send>(
    path: P,
    iterstate: FtIterItemState,
    filter: Option<&'async_recursion FtFilter>,
) -> Result<Vec<PathBuf>> {
    let mut items = vec![];

    let mut entries = fs::read_dir(path.as_ref())
        .await
        .context("list items inner call")?;

    while let Some(entry) = entries.next_entry().await? {
        let e_path = entry.path();

        // If a filter is present, set the value to the result of the filter
        // check, else default to true so always adds the value
        let filter_pass = match filter.as_ref() {
            Some(f) => matches_filter(&e_path, f),
            None => true,
        };

        match iterstate {
            FtIterItemState::FileNoRec => {
                if e_path.is_file() && filter_pass {
                    items.push(e_path);
                }
            }
            FtIterItemState::FileRec => {
                if e_path.is_file() && filter_pass {
                    items.push(e_path)
                } else if e_path.is_dir() {
                    items.extend(iteritems(e_path, iterstate, filter).await?);
                }
            }
            FtIterItemState::DirNoRec => {
                if e_path.is_dir() && filter_pass {
                    items.push(e_path);
                }
            }
            FtIterItemState::DirRec => {
                if e_path.is_dir() {
                    if filter_pass {
                        items.push(e_path.clone());
                    }

                    items.extend(iteritems(e_path, iterstate, filter).await?);
                }
            }
        }
    }

    Ok(items)
}

pub(crate) fn iteritems_sync<P: AsRef<Path>>(
    path: P,
    iterstate: FtIterItemState,
    filter: Option<&FtFilter>,
) -> Result<Vec<PathBuf>> {
    let mut items = vec![];

    let mut entries = std::fs::read_dir(path.as_ref()).context("sync iteritems entry call")?;

    while let Some(Ok(entry)) = entries.next() {
        let e_path = entry.path();

        // If a filter is present, set the value to the result of the filter
        // check, else default to true so always adds the value
        let filter_pass = match filter.as_ref() {
            Some(f) => matches_filter(&e_path, f),
            None => true,
        };
        match iterstate {
            FtIterItemState::FileNoRec => {
                if e_path.is_file() && filter_pass {
                    items.push(e_path);
                }
            }
            FtIterItemState::FileRec => {
                if e_path.is_file() && filter_pass {
                    items.push(e_path)
                } else if e_path.is_dir() {
                    items.extend(iteritems_sync(e_path, iterstate, filter)?);
                }
            }
            FtIterItemState::DirNoRec => {
                if e_path.is_dir() && filter_pass {
                    items.push(e_path);
                }
            }
            FtIterItemState::DirRec => {
                if e_path.is_dir() {
                    if filter_pass {
                        items.push(e_path.clone());
                    }

                    items.extend(iteritems_sync(e_path, iterstate, filter)?);
                }
            }
        }
    }

    Ok(items)
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
