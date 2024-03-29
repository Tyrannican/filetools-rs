//! Crate to help with simple file / folder operations.
//!
//! Provides helper functions to:
//!
//! * Create directories
//! * Check filepaths contain a pattern
//! * List files / directories both iteratively and recursively
//! * List files / directories both iteratively and recursively with filters
//! * Generate names for files / directories
//!
//! ## Async vs Sync
//!
//! The operations in this crate are designed for async/await, however sync variations
//! of the operations exist in the [`crate::sync`] module.
//!
//! # Example
//!
//! ```rust,no_run
//! use filetools::{FtFilter, list_nested_files_with_filter};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Get all Lua files in the Neovim directory
//!     let root_path = "/home/user/.config/nvim";
//!     let filter = FtFilter::Raw("lua".to_string());
//!     let lua_files = list_nested_files_with_filter(&root_path, filter).await?;
//!
//!     // Delete them all, we hate Lua
//!     for lua_file in lua_files.into_iter() {
//!         tokio::fs::remove_file(lua_file).await?;
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod naming;
pub mod sync;
pub(crate) mod util;

use anyhow::{Context, Result};
use regex::Regex;
use std::path::{Component, Path, PathBuf};
use tokio::fs;

use util::{iteritems, FtIterItemState};

/// Filter types for listing files / directories
///
/// # Example
///
/// ```rust
/// use filetools::FtFilter;
/// use std::path::PathBuf;
/// use regex::Regex;
///
/// // Use a raw String filter to match an item containing ".log"
/// let filter = FtFilter::Raw(".log".to_string());
///
/// // Use the Path filter to match paths that contain `sub/path/to/math`
/// let filter = FtFilter::Path(PathBuf::from("sub/path/to/match"));
///
/// // Use a Regex filter to match all files ending with `.rs`
/// let re = Regex::new(r"(.*)\.rs").expect("unable to create regex");
/// let filter = FtFilter::Regex(re);
/// ```
#[derive(Debug)]
pub enum FtFilter {
    /// Filter based on a raw String pattern
    Raw(String),

    /// Filter based on a PathBuf pattern
    Path(PathBuf),

    /// Filter based on a regex pattern
    Regex(Regex),
}

/// Checks if a given pattern is considered a subdirectory of the given path
///
/// # Example
///
/// ```rust
/// use filetools::is_subdir;
///
/// let path = "directory/to/check/for/sub/directory";
/// let check = "for";
///
/// // As "for" is a subdirectory in this path, this returns true
/// let result = is_subdir(path, check);
/// ```
pub fn is_subdir(path: impl AsRef<Path>, dir: impl AsRef<Path>) -> bool {
    for component in path.as_ref().components() {
        if let Component::Normal(p) = component {
            if p == dir.as_ref().as_os_str() {
                return true;
            }
        }
    }

    false
}

/// Determines if a path contains a given pattern
///
/// Converts both the path and the pattern to a string and performs simple matching
///
/// # Example
///
/// ```rust
/// use filetools::path_contains;
///
/// let path = "This/is/a/path/with/a/file.txt";
/// let pattern = "file.txt";
///
/// // The path contains the pattern file.txt so this returns true
/// let result = path_contains(path, pattern);
/// ```
pub fn path_contains(path: impl AsRef<Path>, pattern: impl AsRef<Path> /* maybe */) -> bool {
    if let Some(p) = path.as_ref().to_str() {
        if let Some(pat) = pattern.as_ref().to_str() {
            return p.contains(pat);
        }
    }

    false
}

/// Creates a directory at the given path.
///
/// If the directory already exists, nothing is done
///
/// ## Sync
///
/// For the `sync` version, see [`crate::sync::ensure_directory`]
///
/// # Example
///
/// ```rust,no_run
/// use filetools::ensure_directory;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let target_path = "directory/to/create";
///     ensure_directory(target_path).await?;
///
///     Ok(())
/// }
/// ```
pub async fn ensure_directory(dir: impl AsRef<Path>) -> Result<()> {
    if !dir.as_ref().exists() {
        fs::create_dir_all(dir)
            .await
            .context("unable to create directory")?;
    }

    Ok(())
}

/// Creates multiple directories inside the target path.
///
/// ## Sync
///
/// For the `sync` version, see [`crate::sync::create_multiple_directories`]
///
/// # Example
///
/// ```rust,no_run
/// use filetools::create_multiple_directories;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let root = "dir/to/populate";
///     let to_create = ["dir1", "dir2", "dir3"];
///
///     // Will create:
///     // `dir/to/populate/dir1`
///     // `dir/to/populate/dir2`
///     // `dir/to/populate/dir3`
///     create_multiple_directories(root, &to_create);
///
///     Ok(())
/// }
/// ```
pub async fn create_multiple_directories(
    path: impl AsRef<Path>,
    directories: &[impl AsRef<Path>],
) -> Result<()> {
    for dir in directories {
        let target = path.as_ref().join(dir);
        ensure_directory(target).await?;
    }

    Ok(())
}

/// Creates a range of numeric folders in the given path
///
/// Directories can be padded with X zeros using the `fill` parameter.
///
/// ## Sync
///
/// For the `sync` version, see [`crate::sync::create_numeric_directories`]
///
/// # Example
///
/// ```rust,no_run
/// use filetools::create_numeric_directories;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let root = "some/root/path";
///     
///     // This will create the following directories:
///     // "some/root/path/0000"
///     // ...
///     // "some/root/path/0099"
///     create_numeric_directories(root, 0, 100, 4).await?;
///     Ok(())
/// }
/// ```
pub async fn create_numeric_directories(
    path: impl AsRef<Path>,
    start: usize,
    end: usize,
    fill: usize,
) -> Result<()> {
    for i in start..end {
        let name = path
            .as_ref()
            .join(naming::generate_n_digit_name(i, fill, ""));
        ensure_directory(name)
            .await
            .context("creating numeric directories")?;
    }

    Ok(())
}

/// Lists all files in the given directory (not including subdirectories).
///
/// ## Sync
///
/// For the `sync` version, see [`crate::sync::list_files`]
///
/// # Errors
///
/// This function will return an error in the following situations:
///
/// * The path given is a file and not a directory
/// * The given path does not exist
///
///
/// # Example
///
/// ```rust,no_run
/// use filetools::list_files;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let target_folder = "folder/containing/files";
///
///     // Will return a Vec containing all files in the folder
///     let files = list_files(target_folder).await?;
///     Ok(())
/// }
/// ```
pub async fn list_files<P: AsRef<Path> + Send>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::File, None).await
}

/// Lists all files in a directory including ALL subdirectories
///
/// Use responsibly.
///
/// ## Sync
///
/// For the `sync` version, see [`crate::sync::list_nested_files`]
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
/// use filetools::list_nested_files;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let target_folder = "directory/containing/nested/files";
///
///     // This will return a Vec of ALL files contained within the directory
///     // (including in all subdirectories)
///     let files = list_nested_files(target_folder).await?;
///     Ok(())
/// }
/// ```
pub async fn list_nested_files<P: AsRef<Path> + Send>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::RFile, None).await
}

/// Lists files in a folder (not including subdirectories) matching a filter pattern.
///
/// This pattern can be a `String`, `PathBuf`, or a [`regex::Regex`] pattern.
///
/// ## Sync
///
/// For the `sync` version, see [`crate::sync::list_files_with_filter`]
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
/// use regex::Regex;
/// use std::path::PathBuf;
/// use filetools::{list_files_with_filter, FtFilter};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let root = "some/path/containing/files";
///
///     // List all files containing the phrase `log`
///     let mut filter = FtFilter::Raw("log".to_string());
///     let mut results = list_files_with_filter(&root, filter).await?;
///
///     // List all files containing the path segment `files/test`
///     filter = FtFilter::Path(PathBuf::from("files/test"));
///     results = list_files_with_filter(&root, filter).await?;
///
///     // List all files ending with `.rs`
///     let re = Regex::new(r"(.*)\.rs").expect("unable to create regex");
///     filter = FtFilter::Regex(re);
///     results = list_files_with_filter(&root, filter).await?;
///
///     Ok(())
/// }
/// ```
pub async fn list_files_with_filter<P: AsRef<Path> + Send>(
    path: P,
    pattern: FtFilter,
) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::File, Some(&pattern)).await
}

/// Lists files in a folder (including ALL subdirectories) matching a filter pattern.
///
/// This pattern can be a `String`, `PathBuf`, or a [`regex::Regex`] pattern.
///
/// Use responsibly.
///
/// ## Sync
///
/// For the `sync` version, see [`crate::sync::list_nested_files_with_filter`]
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
/// use regex::Regex;
/// use std::path::PathBuf;
/// use filetools::{list_nested_files_with_filter, FtFilter};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let root = "some/path/containing/nested/folders/with/files";
///
///     // List all files containing the phrase `log`
///     let mut filter = FtFilter::Raw("log".to_string());
///     let mut results = list_nested_files_with_filter(&root, filter).await?;
///
///     // List all files containing the path segment `files/test`
///     filter = FtFilter::Path(PathBuf::from("files/test"));
///     results = list_nested_files_with_filter(&root, filter).await?;
///
///     // List all files ending with `.rs`
///     let re = Regex::new(r"(.*)\.rs").expect("unable to create regex");
///     filter = FtFilter::Regex(re);
///     results = list_nested_files_with_filter(&root, filter).await?;
///
///     Ok(())
/// }
/// ```
pub async fn list_nested_files_with_filter<P: AsRef<Path> + Send>(
    path: P,
    pattern: FtFilter,
) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::RFile, Some(&pattern)).await
}

/// Lists all directories in the given directory (not including subdirectories).
///
/// ## Sync
///
/// For the `sync` version, see [`crate::sync::list_directories`]
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
/// use filetools::list_directories;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let target_folder = "directory/containing/other/directories";
///
///     // Will return a Vec containing all directories in the folder
///     let directories = list_directories(target_folder).await?;
///     Ok(())
/// }
/// ```
pub async fn list_directories<P: AsRef<Path> + Send>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::Dir, None).await
}

/// Lists all directories in a directory including ALL subdirectories
///
/// Use responsibly.
///
/// ## Sync
///
/// For the `sync` version, see [`crate::sync::list_nested_directories`]
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
/// use filetools::list_nested_directories;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let target_folder = "directory/containing/nested/files";
///
///     // This will return a Vec of ALL directories contained within the directory
///     // (including in all subdirectories)
///     let directories = list_nested_directories(target_folder).await?;
///     Ok(())
/// }
/// ```
pub async fn list_nested_directories<P: AsRef<Path> + Send>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    iteritems(path, FtIterItemState::RDir, None).await
}

/// Lists directories in a given directory (not including subdirectories) matching a filter pattern.
///
/// This pattern can be a `String`, `PathBuf`, or a [`regex::Regex`] pattern.
///
/// ## Sync
///
/// For the `sync` version, see [`crate::sync::list_directories_with_filter`]
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
/// use regex::Regex;
/// use std::path::PathBuf;
/// use filetools::{list_directories_with_filter, FtFilter};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let root = "some/path/containing/dirs";
///
///     // List all dirs containing the phrase `log`
///     let mut filter = FtFilter::Raw("log".to_string());
///     let mut results = list_directories_with_filter(&root, filter).await?;
///
///     // List all dirs containing the path segment `files/test`
///     filter = FtFilter::Path(PathBuf::from("files/test"));
///     results = list_directories_with_filter(&root, filter).await?;
///
///     // List all dirs ending with `_test`
///     let re = Regex::new(r"(.*)_test").expect("unable to create regex");
///     filter = FtFilter::Regex(re);
///     results = list_directories_with_filter(&root, filter).await?;
///
///     Ok(())
/// }
/// ```
pub async fn list_directories_with_filter<P: AsRef<Path> + Send>(
    path: P,
    filter: FtFilter,
) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::Dir, Some(&filter)).await
}

/// Lists directories in a given directory (including ALL subdirectories) matching a filter pattern.
///
/// This pattern can be a `String`, `PathBuf`, or a [`regex::Regex`] pattern.
///
/// Use responsibly.
///
/// ## Sync
///
/// For the `sync` version, see [`crate::sync::list_nested_directories_with_filter`]
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
/// use regex::Regex;
/// use std::path::PathBuf;
/// use filetools::{list_nested_directories_with_filter, FtFilter};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let root = "some/path/containing/dirs";
///
///     // List all dirs containing the phrase `log`
///     let mut filter = FtFilter::Raw("log".to_string());
///     let mut results = list_nested_directories_with_filter(&root, filter).await?;
///
///     // List all dirs containing the path segment `files/test`
///     filter = FtFilter::Path(PathBuf::from("files/test"));
///     results = list_nested_directories_with_filter(&root, filter).await?;
///
///     // List all dirs ending with `_test`
///     let re = Regex::new(r"(.*)_test").expect("unable to create regex");
///     filter = FtFilter::Regex(re);
///     results = list_nested_directories_with_filter(&root, filter).await?;
///
///     Ok(())
/// }
/// ```
pub async fn list_nested_directories_with_filter<P: AsRef<Path> + Send>(
    path: P,
    filter: FtFilter,
) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::RDir, Some(&filter)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use std::path::PathBuf;
    use util::TempPath;

    // This is kind of redundant as it just wraps `tokio::fs::create_dir_all`
    // but yay for test coverage i suppose
    #[tokio::test]
    async fn creates_a_directory() -> Result<()> {
        let tmp = std::env::temp_dir();

        // Creates a single directory
        let single_path = tmp.join("create_dir");
        ensure_directory(&single_path)
            .await
            .context("create directory single")?;

        assert!(single_path.exists());

        // Nested directories
        let nested_path = tmp.join("create_dir/test/this/is/nested");
        ensure_directory(&nested_path)
            .await
            .context("create directory nested")?;

        assert!(nested_path.exists());

        std::fs::remove_dir_all(single_path)?;

        Ok(())
    }

    #[tokio::test]
    async fn checks_if_a_directory_is_a_subdirectory() -> Result<()> {
        let root = TempPath::new("is_subdir").await?;
        let nested = root
            .nest_folders(vec!["this", "is", "a", "nested", "tmp", "dir"])
            .await?;
        let mut result = is_subdir(&nested.path, "nested");

        assert!(result);

        result = is_subdir(&nested.path, "not_valid");

        assert!(!result);
        Ok(())
    }

    #[test]
    fn check_path_contains_subpath() {
        // Basic str
        let main = "I/am/a/path/hello/there";
        assert!(path_contains(main, "a/path"));
        assert!(!path_contains(main, "not"));

        // Check it works for paths
        let main = Path::new(main);
        assert!(path_contains(main, Path::new("a/path")));
        assert!(!path_contains(main, Path::new("not")));

        // Pathbufs?
        let main = PathBuf::from("I/am/a/path/hello/there");
        assert!(path_contains(&main, PathBuf::from("a/path")));
        assert!(!path_contains(main, PathBuf::from("not")));

        // What about strings?
        assert!(path_contains(
            String::from("I/am/a/path/hello/there"),
            String::from("a/path")
        ));
        assert!(!path_contains(
            String::from("I/am/a/path/hello/there"),
            String::from("not")
        ));
    }

    #[tokio::test]
    async fn check_list_files_works() -> Result<()> {
        let root = TempPath::new("lf_test").await?;
        root.multi_file(vec!["first.rs", "second.c", "third.js", "fourth.rb"])
            .await?;

        let res = list_files(root.path.clone()).await?;
        assert_eq!(res.len(), 4);

        assert!(list_files("IDoNotExistAsADirectoryOrShouldntAtLeAst")
            .await
            .is_err());

        Ok(())
    }

    #[tokio::test]
    async fn check_list_nested_files_works() -> Result<()> {
        let root = TempPath::new("lfr_test").await?;
        let ffolder = root.new_folder("ffolder").await?;
        let sfolder = root.new_folder("sfolder").await?;
        let tfolder = root.new_folder("tfolder").await?;

        root.new_file("initial.pdf").await?;
        ffolder.new_file("first.rs").await?;
        sfolder.multi_file(vec!["second.txt", "third.php"]).await?;
        tfolder.new_file("fourth.cpp").await?;

        let res = list_nested_files(&root.path).await?;
        assert_eq!(res.len(), 5);

        assert!(list_files("IDoNotExistAsADirectoryOrShouldntAtLeAst")
            .await
            .is_err());

        Ok(())
    }

    #[tokio::test]
    async fn check_list_directories_works() -> Result<()> {
        let root = TempPath::new("lfolder_test").await?;
        root.multi_folder(vec!["folder1", "folder2", "folder3", "folder4"])
            .await?;

        let res = list_directories(root.path.clone()).await?;
        assert_eq!(res.len(), 4);

        assert!(list_directories("non-existant_path").await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn check_list_nested_directories_works() -> Result<()> {
        let root = TempPath::new("lfolderrec_test").await?;
        root.multi_folder(vec!["folder1", "folder2"]).await?;

        let f1 = TempPath::new(root.join("folder1")).await?;
        f1.multi_folder(vec!["sub1", "sub2", "sub3"]).await?;

        let s2 = TempPath::new(f1.join("sub2")).await?;
        s2.multi_folder(vec!["deep1", "deep2"]).await?;

        let res = list_nested_directories(root.path.clone()).await?;
        assert_eq!(res.len(), 7);

        assert!(list_nested_directories("not-a-valId_pathd").await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn numeric_directories() -> Result<()> {
        let tmp = TempPath::new("numeric_directories").await?;
        create_numeric_directories(&tmp.path, 0, 100, 4).await?;
        let mut folders = list_directories(&tmp.path).await?;
        folders.sort();
        assert_eq!(folders.len(), 100);

        for (i, folder) in folders.into_iter().enumerate() {
            let test = &tmp.path.join(format!("{:0fill$}", i, fill = 4));
            assert_eq!(&folder, test);
        }

        Ok(())
    }

    #[tokio::test]
    async fn multiple_directory_creation() -> Result<()> {
        let tmp = TempPath::new("create_multiple_dirs").await?;
        let dirs = ["config", "src", "tests"];

        create_multiple_directories(&tmp.path, &dirs).await?;
        let folders = list_directories(&tmp.path).await?;
        assert_eq!(folders.len(), 3);

        for check in dirs {
            let target = tmp.path.join(check);
            assert!(folders.contains(&target));
        }

        Ok(())
    }

    #[tokio::test]
    async fn files_filter() -> Result<()> {
        let root = TempPath::new("filter_files").await?;
        root.multi_file(vec!["first.rs", "second.rs", "third.js", "fourth.rb"])
            .await?;

        // Raw string filter
        let mut filter = FtFilter::Raw("fourth".to_string());
        let mut result = list_files_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], root.path.join("fourth.rb"));

        // PathBuf filter
        filter = FtFilter::Path(PathBuf::from("third.js"));
        result = list_files_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], root.path.join("third.js"));

        // Regex filter
        filter = FtFilter::Regex(Regex::new(r"(.*)\.rs").unwrap());
        result = list_files_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 2);
        assert!(result.contains(&root.path.join("first.rs")));
        assert!(result.contains(&root.path.join("second.rs")));

        Ok(())
    }

    #[tokio::test]
    async fn files_filter_is_empty() -> Result<()> {
        let root = TempPath::new("filter_files_empty").await?;

        // Raw string filter (normal + nested)
        let mut filter = FtFilter::Raw("non-existant".to_string());
        let mut result = list_files_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());
        filter = FtFilter::Raw("non-existant".to_string());
        result = list_nested_files_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());

        // PathBuf Filter
        filter = FtFilter::Path(PathBuf::from("another-missing"));
        result = list_files_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());
        filter = FtFilter::Path(PathBuf::from("another-missing"));
        result = list_nested_files_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());

        // Regex filter
        filter = FtFilter::Regex(Regex::new(r"(.*)\.rs").unwrap());
        result = list_files_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());
        filter = FtFilter::Regex(Regex::new(r"(.*)\.rs").unwrap());
        result = list_nested_files_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn list_items_error() -> Result<()> {
        let root = TempPath::new("filter_files_error").await?;
        let test = root.new_file("test.js").await?;

        assert!(list_files(&test.path).await.is_err());
        assert!(list_nested_files(&test.path).await.is_err());
        assert!(
            list_files_with_filter(&test.path, FtFilter::Raw("filter".to_string()))
                .await
                .is_err()
        );
        assert!(
            list_nested_files_with_filter(&test.path, FtFilter::Raw("filter".to_string()))
                .await
                .is_err()
        );
        assert!(list_directories(&test.path).await.is_err());
        assert!(list_nested_directories(&test.path).await.is_err());
        assert!(
            list_directories_with_filter(&test.path, FtFilter::Raw("filter".to_string()))
                .await
                .is_err()
        );
        assert!(list_nested_directories_with_filter(
            &test.path,
            FtFilter::Raw("filter".to_string())
        )
        .await
        .is_err());
        Ok(())
    }

    #[tokio::test]
    async fn nested_files_filter() -> Result<()> {
        let root = TempPath::new("nested_filter_files").await?;
        let ffolder = root.new_folder("ffolder").await?;
        let sfolder = root.new_folder("sfolder").await?;
        let tfolder = root.new_folder("tfolder").await?;

        root.new_file("initial.pdf").await?;
        ffolder.new_file("first.rs").await?;
        sfolder.multi_file(vec!["second.txt", "third.rs"]).await?;
        tfolder.new_file("initial.cpp").await?;

        let mut filter = FtFilter::Raw("initial".to_string());
        let mut result = list_nested_files_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 2);
        assert!(result.contains(&root.path.join("tfolder/initial.cpp")));
        assert!(result.contains(&root.path.join("initial.pdf")));

        filter = FtFilter::Path(PathBuf::from("second.txt"));
        result = list_nested_files_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], root.path.join("sfolder/second.txt"));
        Ok(())
    }

    #[tokio::test]
    async fn directories_filter() -> Result<()> {
        let root = TempPath::new("dir_filter").await?;
        root.multi_folder(vec!["log_var", "store_var", "config", "etc"])
            .await?;

        // Raw string filter
        let mut filter = FtFilter::Raw("config".to_string());
        let mut result = list_directories_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], root.path.join("config"));

        // PathBuf filter
        filter = FtFilter::Path(PathBuf::from("etc"));
        result = list_directories_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], root.path.join("etc"));

        // Regex filter
        filter = FtFilter::Regex(Regex::new(r"(.*)_var").unwrap());
        result = list_directories_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 2);
        assert!(result.contains(&root.path.join("log_var")));
        assert!(result.contains(&root.path.join("store_var")));
        Ok(())
    }

    #[tokio::test]
    async fn nested_directories_filter() -> Result<()> {
        let root = TempPath::new("nested_dir_filter_test").await?;
        root.multi_folder(vec!["folder1", "folder2"]).await?;

        let f1 = TempPath::new(root.join("folder1")).await?;
        f1.multi_folder(vec!["sub1", "sub_2", "sub3"]).await?;

        let s2 = TempPath::new(f1.join("sub_2")).await?;
        s2.multi_folder(vec!["deep_1", "deep2"]).await?;

        // Raw filter
        let mut filter = FtFilter::Raw("deep".to_string());
        let mut result = list_nested_directories_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 2);
        assert!(result.contains(&root.path.join("folder1/sub_2/deep_1")));
        assert!(result.contains(&root.path.join("folder1/sub_2/deep2")));

        // Path filter
        filter = FtFilter::Path(PathBuf::from("folder1"));
        result = list_nested_directories_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 6);

        filter = FtFilter::Regex(Regex::new(r"(.*)_[0-9]{1}").unwrap());
        result = list_nested_directories_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn list_dirs_empty() -> Result<()> {
        let root = TempPath::new("list_dirs_empty").await?;

        // Raw string filter (normal + nested)
        let mut filter = FtFilter::Raw("non-existant".to_string());
        let mut result = list_directories_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());
        filter = FtFilter::Raw("non-existant".to_string());
        result = list_nested_directories_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());

        // PathBuf Filter
        filter = FtFilter::Path(PathBuf::from("another-missing"));
        result = list_directories_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());
        filter = FtFilter::Path(PathBuf::from("another-missing"));
        result = list_nested_directories_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());

        // Regex filter
        filter = FtFilter::Regex(Regex::new(r"(.*)\.rs").unwrap());
        result = list_directories_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());
        filter = FtFilter::Regex(Regex::new(r"(.*)\.rs").unwrap());
        result = list_nested_directories_with_filter(&root.path, filter).await?;
        assert!(result.is_empty());
        Ok(())
    }
}
