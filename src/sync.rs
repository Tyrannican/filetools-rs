//! Sync variations of the main [`crate`] functions
//!
//! All operations are identical to those defined in the `async` version.
use crate::util::FtIterItemState;
use crate::{naming::generate_n_digit_name, util::iteritems_sync, FtFilter};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Creates a directory at the given path.
///
/// If the directory already exists, nothing is done
///
/// ## Async
///
/// For the `async` version, see: [`crate::ensure_directory`]
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

/// Creates a range of numeric folders in the given path
///
/// Directories can be padded with X zeros using the `fill` parameter.
///
/// ## Async
///
/// For the `async` version, see: [`crate::create_numeric_directories`]
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
/// ## Async
///
/// For the `async` version, see: [`crate::create_multiple_directories`]
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
/// ## Async
///
/// For the `async` version, see: [`crate::list_files`]
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

    iteritems_sync(path, FtIterItemState::File, None)
}

/// Lists all files in a directory including ALL subdirectories
///
/// Use responsibly.
///
/// ## Async
///
/// For the `async` version, see: [`crate::list_nested_files`]
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

    iteritems_sync(path, FtIterItemState::RFile, None)
}

/// Lists files in a folder (not including subdirectories) matching a filter pattern.
///
/// This pattern can be a `String`, `PathBuf`, or a [`regex::Regex`] pattern.
///
/// ## Async
///
/// For the `async` version, see: [`crate::list_files_with_filter`]
///
/// # Example
///
/// ```rust,no_run
/// use regex::Regex;
/// use std::path::PathBuf;
/// use filetools::{sync::list_files_with_filter, FtFilter};
///
/// let root = "some/path/containing/files";
///
/// // List all files containing the phrase `log`
/// let mut filter = FtFilter::Raw("log".to_string());
/// let mut results = list_files_with_filter(&root, filter).expect("unable to list filtered files");
///
/// // List all files containing the path segment `files/test`
/// filter = FtFilter::Path(PathBuf::from("files/test"));
/// results = list_files_with_filter(&root, filter).expect("unable to list filtered files");
///
/// // List all files ending with `.rs`
/// let re = Regex::new(r"(.*)\.rs").expect("unable to create regex");
/// filter = FtFilter::Regex(re);
/// results = list_files_with_filter(&root, filter).expect("unable to list filtered files");
///
///
/// ```
pub fn list_files_with_filter<P: AsRef<Path>>(path: P, filter: FtFilter) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems_sync(path, FtIterItemState::File, Some(&filter))
}

/// Lists files in a folder (including ALL subdirectories) matching a filter pattern.
///
/// This pattern can be a `String`, `PathBuf`, or a [`regex::Regex`] pattern.
///
/// Use responsibly.
///
/// ## Async
///
/// For the `async` version, see: [`crate::list_nested_files_with_filter`]
///
/// # Example
///
/// ```rust,no_run
/// use regex::Regex;
/// use std::path::PathBuf;
/// use filetools::{sync::list_nested_files_with_filter, FtFilter};
///
/// let root = "some/path/containing/nested/folders/with/files";
///
/// // List all files containing the phrase `log`
/// let mut filter = FtFilter::Raw("log".to_string());
/// let mut results = list_nested_files_with_filter(&root, filter).expect("unable to list nested files with filter");
///
/// // List all files containing the path segment `files/test`
/// filter = FtFilter::Path(PathBuf::from("files/test"));
/// results = list_nested_files_with_filter(&root, filter).expect("unable to list nested files with filter");
///
/// // List all files ending with `.rs`
/// let re = Regex::new(r"(.*)\.rs").expect("unable to create regex");
/// filter = FtFilter::Regex(re);
/// results = list_nested_files_with_filter(&root, filter).expect("unable to list nested files with filter");
/// ```
pub fn list_nested_files_with_filter<P: AsRef<Path>>(
    path: P,
    filter: FtFilter,
) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems_sync(path, FtIterItemState::RFile, Some(&filter))
}

/// Lists all directories in the given directory (not including subdirectories).
///
/// ## Async
///
/// For the `async` version, see: [`crate::list_directories`]
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
    iteritems_sync(path, FtIterItemState::Dir, None)
}

/// Lists all directories in a directory including ALL subdirectories
///
/// Use responsibly.
///
/// ## Async
///
/// For the `async` version, see: [`crate::list_nested_directories`]
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
    iteritems_sync(path, FtIterItemState::RDir, None)
}

/// Lists directories in a given directory (not including subdirectories) matching a filter pattern.
///
/// This pattern can be a `String`, `PathBuf`, or a [`regex::Regex`] pattern.
///
/// ## Async
///
/// For the `async` version, see: [`crate::list_directories_with_filter`]
///
/// # Example
///
/// ```rust,no_run
/// use regex::Regex;
/// use std::path::PathBuf;
/// use filetools::{sync::list_directories_with_filter, FtFilter};
///
/// let root = "some/path/containing/dirs";
///
/// // List all dirs containing the phrase `log`
/// let mut filter = FtFilter::Raw("log".to_string());
/// let mut results = list_directories_with_filter(&root, filter).expect("unable to list dirs with filter");
///
/// // List all dirs containing the path segment `files/test`
/// filter = FtFilter::Path(PathBuf::from("files/test"));
/// results = list_directories_with_filter(&root, filter).expect("unable to list dirs with filter");
///
/// // List all dirs ending with `_test`
/// let re = Regex::new(r"(.*)_test").expect("unable to create regex");
/// filter = FtFilter::Regex(re);
/// results = list_directories_with_filter(&root, filter).expect("unable to list dirs with filter");
/// ```
pub fn list_directories_with_filter<P: AsRef<Path>>(
    path: P,
    filter: FtFilter,
) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );
    iteritems_sync(path, FtIterItemState::Dir, Some(&filter))
}

/// Lists directories in a given directory (including ALL subdirectories) matching a filter pattern.
///
/// Use responsibly.
///
/// This pattern can be a `String`, `PathBuf`, or a [`regex::Regex`] pattern.
///
/// ## Async
///
/// For the `async` version, see: [`crate::list_nested_directories_with_filter`]
///
/// # Example
///
/// ```rust,no_run
/// use regex::Regex;
/// use std::path::PathBuf;
/// use filetools::{sync::list_nested_directories_with_filter, FtFilter};
///
/// let root = "some/path/containing/dirs";
///
/// // List all dirs containing the phrase `log`
/// let mut filter = FtFilter::Raw("log".to_string());
/// let mut results = list_nested_directories_with_filter(&root, filter).expect("unable to list nested dirs with filter");
///
/// // List all dirs containing the path segment `files/test`
/// filter = FtFilter::Path(PathBuf::from("files/test"));
/// results = list_nested_directories_with_filter(&root, filter).expect("unable to list nested dirs with filter");
///
/// // List all dirs ending with `_test`
/// let re = Regex::new(r"(.*)_test").expect("unable to create regex");
/// filter = FtFilter::Regex(re);
/// results = list_nested_directories_with_filter(&root, filter).expect("unable to list nested dirs with filter");
/// ```
pub fn list_nested_directories_with_filter<P: AsRef<Path>>(
    path: P,
    filter: FtFilter,
) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );
    iteritems_sync(path, FtIterItemState::RDir, Some(&filter))
}

// No tests needed cause these are tested in the main crate
