//! Crate to help with simple file / folder operations.
//!
//! Provides helper functions to:
//!
//! * Create directories
//! * Check filepaths contain a pattern
//! * List files / directories both iteratively and recursively
//! * Generate names for files / directories
//!
//! TODO: More Docs!
pub mod naming;
pub mod sync;
pub(crate) mod util;

use anyhow::{Context, Result};
use async_recursion::async_recursion;
use regex::Regex;
use std::path::{Component, Path, PathBuf};
use tokio::fs;

use util::FtIterItemState;

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
        match component {
            Component::Normal(p) => {
                if p == dir.as_ref().as_os_str() {
                    return true;
                }
            }
            _ => {}
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
            return p.contains(&pat);
        }
    }

    false
}

/// Creates a directory at the given path.
///
/// If the directory already exists, nothing is done
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

/// Creates a range of numeric folders in the given path starting from `start`
/// up to `end` (non-inclusive).
///
/// Directories can be padded with X zeros using the `fill` parameter.
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

    iteritems(path, FtIterItemState::FileNoRec).await
}

pub async fn list_files_with_filter<P: AsRef<Path> + Send>(
    path: P,
    pattern: FtFilter,
) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    let results = iteritems(path, FtIterItemState::FileNoRec)
        .await?
        .into_iter()
        .filter_map(|item| match &pattern {
            // I know these are the same for Raw and Path
            // but it complains when you try and use the | with match
            // for this
            FtFilter::Raw(raw) => {
                if path_contains(&item, raw) {
                    return Some(item);
                }

                None
            }
            FtFilter::Path(filter_path) => {
                if path_contains(&item, filter_path) {
                    return Some(item);
                }

                None
            }
            FtFilter::Regex(re) => {
                if re.is_match(item.to_str().unwrap()) {
                    return Some(item);
                }

                None
            }
        })
        .collect();

    Ok(results)
}

/// Lists all directories in the given directory (not including subdirectories).
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

    iteritems(path, FtIterItemState::DirNoRec).await
}

/// Lists all files in a directory including ALL subdirectories
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
#[async_recursion]
pub async fn list_nested_files<P: AsRef<Path> + Send>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::FileRec).await
}

/// Lists all directories in a directory including ALL subdirectories
///
/// # Errors
///
/// This function will return an error in the following situations:
///
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
#[async_recursion]
pub async fn list_nested_directories<P: AsRef<Path> + Send>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    iteritems(path, FtIterItemState::DirRec).await
}

/// Helper function to iterate through a directory to find all Files / Directories
/// depending on the `FilterState` passed.
#[async_recursion]
async fn iteritems<P: AsRef<Path> + Send>(
    path: P,
    iterstate: FtIterItemState,
) -> Result<Vec<PathBuf>> {
    let mut items = vec![];

    let mut entries = fs::read_dir(path.as_ref())
        .await
        .context("list items inner call")?;

    while let Some(entry) = entries.next_entry().await? {
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
                    items.extend(iteritems(e_path, iterstate).await?);
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
                    items.extend(iteritems(e_path, iterstate).await?);
                }
            }
        }
    }

    Ok(items)
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

        let mut filter = FtFilter::Raw("fourth".to_string());
        let mut result = list_files_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], root.path.join("fourth.rb"));

        filter = FtFilter::Path(PathBuf::from("third.js"));
        result = list_files_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], root.path.join("third.js"));

        filter = FtFilter::Regex(Regex::new(r"(.*)\.rs").unwrap());
        result = list_files_with_filter(&root.path, filter).await?;
        assert_eq!(result.len(), 2);
        assert!(result.contains(&root.path.join("first.rs")));
        assert!(result.contains(&root.path.join("second.rs")));
        Ok(())
    }
}
