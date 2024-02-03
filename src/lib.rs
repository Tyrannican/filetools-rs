use anyhow::{Context, Result};
use async_recursion::async_recursion;
use std::path::{Component, Path, PathBuf};
use tokio::fs;

// pub mod filehelpers;
// pub mod filenaming;

// What do we want?
//
// Iterate files in a directory - DONE
// Iterate folders in a directory
//
// Iterate files in a directory + all subdirs - DONE
// Iterate folders in a directory + all subdirs
//
// Create a directory if not exists - DONE
//
// Check if a directory is a subdirectory - DONE
//
// Pattern match on a path - DONE
//
// All of the above but sync (feature)
//
// Naming patterns
//

pub async fn create_directory(dir: impl AsRef<Path>) -> Result<()> {
    if !dir.as_ref().exists() {
        fs::create_dir_all(dir)
            .await
            .context("unable to create directory")?;
    }

    Ok(())
}

pub async fn is_subdir(path: impl AsRef<Path>, dir: impl AsRef<Path>) -> Result<bool> {
    for component in path.as_ref().components() {
        match component {
            Component::Normal(p) => {
                if p == dir.as_ref().as_os_str() {
                    return Ok(true);
                }
            }
            _ => {}
        }
    }

    Ok(false)
}

pub fn path_contains(path: impl AsRef<Path>, pattern: impl AsRef<Path> /* maybe */) -> bool {
    if let Some(p) = path.as_ref().to_str() {
        if let Some(pat) = pattern.as_ref().to_str() {
            return p.contains(&pat);
        }
    }

    false
}

pub async fn list_files(path: impl AsRef<Path>) -> Result<Vec<impl AsRef<Path>>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");

    let mut files = vec![];
    if path.as_ref().is_file() {
        return Ok(files);
    }

    let mut entries = fs::read_dir(path.as_ref())
        .await
        .context("list_files directory read")?;

    while let Some(entry) = entries.next_entry().await? {
        let e_path = entry.path();

        if e_path.is_file() {
            files.push(e_path);
        }
    }

    Ok(files)
}

#[async_recursion]
pub async fn list_files_recursive<P: AsRef<Path> + Send>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");

    let mut files = vec![];
    if path.as_ref().is_file() {
        return Ok(files);
    }

    let mut entries = fs::read_dir(path.as_ref())
        .await
        .context("list_files_recursive directory read")?;

    while let Some(entry) = entries.next_entry().await? {
        let e_path = entry.path();

        if e_path.is_file() {
            files.push(e_path);
        } else if e_path.is_dir() {
            files.extend(
                list_files_recursive(e_path)
                    .await
                    .context("recursive list_files call")?,
            );
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use std::path::PathBuf;

    /// Helper for creating temp directories
    ///
    /// Tempfile _would_ work but I want nested dirs and easy ways to create
    /// a series of files / folder quickly without worrying
    struct TempPath {
        pub path: PathBuf,
    }

    impl TempPath {
        pub async fn new(p: impl AsRef<Path>) -> Result<Self> {
            let root = std::env::temp_dir();
            let path = if p.as_ref().starts_with(&root) {
                p.as_ref().to_path_buf()
            } else {
                root.join(p)
            };

            create_directory(&path).await?;

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
            create_directory(&p).await?;

            Self::new(p).await
        }

        pub async fn multi_folder(&self, names: Vec<impl AsRef<Path>>) -> Result<()> {
            for name in names {
                create_directory(&self.path.join(name)).await?;
            }

            Ok(())
        }

        pub fn join(&self, path: impl AsRef<Path>) -> impl AsRef<Path> {
            self.path.join(path)
        }

        pub async fn cleanup(self) -> Result<()> {
            tokio::fs::remove_dir_all(&self.path).await?;
            drop(self);
            Ok(())
        }
    }

    // This is needed as the `tempfile` lib doesn't like nested temp dirs
    async fn create_tmpdir(path: &str) -> Result<impl AsRef<Path>> {
        let target = std::env::temp_dir().join(path);
        tokio::fs::create_dir_all(&target)
            .await
            .context("creating tempdir")?;

        Ok(target)
    }

    #[tokio::test]
    // This is kind of redundant as it just wraps `tokio::fs::create_dir_all`
    // but yay for test coverage i suppose
    async fn creates_a_directory() -> Result<()> {
        let tmp = std::env::temp_dir();

        // Creates a single directory
        let single_path = tmp.join("create_dir");
        create_directory(&single_path)
            .await
            .context("create directory single")?;

        assert!(single_path.exists());

        // Nested directories
        let nested_path = tmp.join("create_dir/test/this/is/nested");
        create_directory(&nested_path)
            .await
            .context("create directory nested")?;

        assert!(nested_path.exists());

        Ok(())
    }

    #[tokio::test]
    async fn checks_if_a_directory_is_a_subdirectory() -> Result<()> {
        let path = create_tmpdir("is_subdir/this/is/a/nested/tmp/dir")
            .await
            .context("creating nested tempdirs")?;

        let mut result = is_subdir(&path, "nested").await.context("is_subdir test")?;
        assert!(result);

        result = is_subdir(&path, "not_valid")
            .await
            .context("is_subdir test")?;

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

        root.cleanup().await?;

        Ok(())
    }

    #[tokio::test]
    async fn check_list_files_recursive_works() -> Result<()> {
        let root = TempPath::new("lfr_test").await?;
        let ffolder = root.new_folder("ffolder").await?;
        let sfolder = root.new_folder("sfolder").await?;
        let tfolder = root.new_folder("tfolder").await?;

        root.new_file("initial.pdf").await?;
        ffolder.new_file("first.rs").await?;
        sfolder.multi_file(vec!["second.txt", "third.php"]).await?;
        tfolder.new_file("fourth.cpp").await?;

        let res = list_files_recursive(&root.path).await?;
        assert_eq!(res.len(), 5);

        assert!(list_files("IDoNotExistAsADirectoryOrShouldntAtLeAst")
            .await
            .is_err());

        root.cleanup().await?;

        Ok(())
    }
}
