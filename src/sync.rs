//! Sync variations of the main [`crate`] functions
use crate::{naming::generate_n_digit_name, FtIterItemState};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Creates a directory at the given path.
///
/// If the directory already exists, nothing is done
///
/// This is the sync version of [`crate::ensure_directory`]
///
/// # Example
///
/// ```rust,no_run
/// use filetools::sync::ensure_directory;
///
/// let target_path = "directory/to/create";
/// ensure_directory(target_path).expect("unable to create directory");
///
/// ```
pub fn ensure_directory(dir: impl AsRef<Path>) -> Result<()> {
    if !dir.as_ref().exists() {
        fs::create_dir_all(dir).context("unable to create directory")?;
    }

    Ok(())
}

pub fn list_files<P: AsRef<Path>>(path: P) -> Result<Vec<impl AsRef<Path>>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::FileNoRec)
}

pub fn create_numeric_directories(
    path: impl AsRef<Path>,
    start: usize,
    end: usize,
    fill: usize,
) -> Result<()> {
    for i in start..end {
        let name = path.as_ref().join(generate_n_digit_name(i, fill, ""));
        ensure_directory(name).context("creating numeric directories")?;
    }

    Ok(())
}

pub fn list_directories<P: AsRef<Path>>(path: P) -> Result<Vec<impl AsRef<Path>>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    iteritems(path, FtIterItemState::DirNoRec)
}

pub fn list_files_recursive<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::FileRec)
}

pub fn list_directories_recursive<P: AsRef<Path> + Send>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    iteritems(path, FtIterItemState::DirRec)
}

fn iteritems<P: AsRef<Path>>(path: P, iterstate: FtIterItemState) -> Result<Vec<PathBuf>> {
    let mut items = vec![];

    let mut entries = fs::read_dir(path.as_ref()).context("list items inner call")?;

    while let Some(Ok(entry)) = entries.next() {
        let e_path = entry.path();
        match iterstate {
            FtIterItemState::FileNoRec => {
                if e_path.is_file() {
                    items.push(e_path);
                }
            }
            FtIterItemState::FileRec => {
                if e_path.is_file() {
                    items.push(e_path)
                } else if e_path.is_dir() {
                    items.extend(iteritems(e_path, iterstate)?);
                }
            }
            FtIterItemState::DirNoRec => {
                if e_path.is_dir() {
                    items.push(e_path);
                }
            }
            FtIterItemState::DirRec => {
                if e_path.is_dir() {
                    items.push(e_path.clone());
                    items.extend(iteritems(e_path, iterstate)?);
                }
            }
        }
    }

    Ok(items)
}

// Do I write tests for these as they are just the sync version of the Ft functions
// which already pass...?
// TODO: Maybe tests?
