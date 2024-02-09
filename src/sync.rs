//! Sync variations of the main [`crate`] functions
use crate::util::FtIterItemState;
use crate::{naming::generate_n_digit_name, util::iteritems_sync, FtFilter};
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

/// Creates a range of numeric folders in the given path starting from `start`
/// up to `end` (non-inclusive).
///
/// Directories can be padded with X zeros using the `fill` parameter.
///
/// This is the sync version of [`crate::create_numeric_directories`]
///
/// # Example
///
/// ```rust,no_run
/// use filetools::sync::create_numeric_directories;
///
/// let root = "some/root/path";
///
/// // This will create the following directories:
/// // "some/root/path/0000"
/// // ...
/// // "some/root/path/0099"
/// create_numeric_directories(root, 0, 100, 4).expect("unable to create numeric directories");
/// ```
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

/// Creates multiple directories inside the target path.
///
/// This is the sync version of [`crate::create_multiple_directories`]
///
/// # Example
///
/// ```rust,no_run
/// use filetools::sync::create_multiple_directories;
///
/// let root = "dir/to/populate";
/// let to_create = ["dir1", "dir2", "dir3"];
///
/// // Will create:
/// // `dir/to/populate/dir1`
/// // `dir/to/populate/dir2`
/// // `dir/to/populate/dir3`
/// create_multiple_directories(root, &to_create).expect("unable to create multiple directories");
/// ```
pub fn create_multiple_directories(
    path: impl AsRef<Path>,
    directories: &[impl AsRef<Path>],
) -> Result<()> {
    for dir in directories {
        let target = path.as_ref().join(dir);
        ensure_directory(target)?;
    }

    Ok(())
}

/// Lists all files in the given directory (not including subdirectories).
///
/// This is the sync version of [`crate::list_files`]
///
/// # Errors
///
/// This function will return an error in the following situations:
///
/// * The path given is a file and not a directory
/// * The given path does not exist
///
/// # Example
///
/// ```rust,no_run
/// use filetools::sync::list_files;
///
/// let target_dir = "some/dir/containing/files";
///
/// // Will return a Vec containing paths to all files in the directory
/// let files = list_files(target_dir).expect("unable to list files");
/// ```
pub fn list_files<P: AsRef<Path>>(path: P) -> Result<Vec<impl AsRef<Path>>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems_sync(path, FtIterItemState::FileNoRec, None)
}

/// Lists all directories in the given directory (not including subdirectories).
///
/// This is the sync version of [`crate::list_directories`]
///
/// # Errors
///
/// This function will return an error in the following situations:
///
/// * The path given is a file and not a directory
/// * The given path does not exist
///
/// # Example
///
/// ```rust,no_run
/// use filetools::sync::list_directories;
///
/// let target_dir = "some/dir/containing/files";
///
/// // Will return a Vec containing paths to all directories in the directory
/// let dirs = list_directories(target_dir).expect("unable to list directories");
/// ```
pub fn list_directories<P: AsRef<Path>>(path: P) -> Result<Vec<impl AsRef<Path>>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );
    iteritems_sync(path, FtIterItemState::DirNoRec, None)
}

/// Lists all files in a directory including ALL subdirectories
///
/// This is the sync version of [`crate::list_nested_files`]
///
/// # Errors
///
/// This function will return an error in the following situations:
///
/// * The given path is a file and not a directory
/// * The given path does not exist
///
/// # Example
///
/// ```rust,no_run
/// use filetools::sync::list_nested_files;
///
/// let target_dir = "some/dir/containing/nested/files";
///
/// // Will return a Vec containing all files in the directory (including all subdirectories)
/// let files = list_nested_files(target_dir).expect("unable to list files recursively");
/// ```
pub fn list_nested_files<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems_sync(path, FtIterItemState::FileRec, None)
}

/// Lists all directories in a directory including ALL subdirectories
///
/// This is the sync version of [`crate::list_nested_directories`]
///
/// # Errors
///
/// This function will return an error in the following situations:
///
/// * The given path is a file and not a directory
/// * The given path does not exist
///
/// # Example
///
/// ```rust,no_run
/// use filetools::sync::list_nested_directories;
///
/// let target_dir = "some/dir/containing/nested/files";
///
/// // Will return a Vec containing all directories in the directory (including all subdirectories)
/// let dirs = list_nested_directories(target_dir).expect("unable to list directories recursively");
/// ```
pub fn list_nested_directories<P: AsRef<Path> + Send>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );
    iteritems_sync(path, FtIterItemState::DirRec, None)
}

// Do I write tests for these as they are just the sync version of the Ft functions
// which already pass...?
// TODO: Maybe tests?
