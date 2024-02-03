use anyhow::{Context, Result};
use async_recursion::async_recursion;
use std::path::{Component, Path, PathBuf};
use tokio::fs;

// pub mod filehelpers;
// pub mod filenaming;

// What do we want?
//
// Iterate files in a directory - DONE
// Iterate folders in a directory - DONE
//
// Iterate files in a directory + all subdirs - DONE
// Iterate folders in a directory + all subdirs - DONE
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum FtIterItemState {
    FileNoRec,
    FileRec,
    DirNoRec,
    DirRec,
}

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

pub async fn list_files<P: AsRef<Path> + Send>(path: P) -> Result<Vec<impl AsRef<Path>>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::FileNoRec).await
}

pub async fn list_folders<P: AsRef<Path> + Send>(path: P) -> Result<Vec<impl AsRef<Path>>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    iteritems(path, FtIterItemState::DirNoRec).await
}

#[async_recursion]
pub async fn list_files_recursive<P: AsRef<Path> + Send>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    anyhow::ensure!(
        path.as_ref().is_dir(),
        "path should be a directory, not a file"
    );

    iteritems(path, FtIterItemState::FileRec).await
}

#[async_recursion]
pub async fn list_folders_recursive<P: AsRef<Path> + Send>(path: P) -> Result<Vec<PathBuf>> {
    anyhow::ensure!(path.as_ref().exists(), "path does not exist");
    iteritems(path, FtIterItemState::DirRec).await
}

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

        pub async fn nest_folders(&self, subfolder_chain: Vec<impl AsRef<Path>>) -> Result<Self> {
            let mut dst_path = self.path.clone();
            for sf in subfolder_chain {
                dst_path = dst_path.join(sf.as_ref());
            }

            create_directory(&dst_path).await?;
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

        std::fs::remove_dir_all(single_path)?;

        Ok(())
    }

    #[tokio::test]
    async fn checks_if_a_directory_is_a_subdirectory() -> Result<()> {
        let root = TempPath::new("is_subdir").await?;
        let nested = root
            .nest_folders(vec!["this", "is", "a", "nested", "tmp", "dir"])
            .await?;
        let mut result = is_subdir(&nested.path, "nested")
            .await
            .context("is_subdir test")?;

        assert!(result);

        result = is_subdir(&nested.path, "not_valid")
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

        Ok(())
    }

    #[tokio::test]
    async fn check_list_folders_works() -> Result<()> {
        let root = TempPath::new("lfolder_test").await?;
        root.multi_folder(vec!["folder1", "folder2", "folder3", "folder4"])
            .await?;

        let res = list_folders(root.path.clone()).await?;
        assert_eq!(res.len(), 4);

        assert!(list_folders("non-existant_path").await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn check_list_folders_recursive_works() -> Result<()> {
        let root = TempPath::new("lfolderrec_test").await?;
        root.multi_folder(vec!["folder1", "folder2"]).await?;

        let f1 = TempPath::new(root.join("folder1")).await?;
        f1.multi_folder(vec!["sub1", "sub2", "sub3"]).await?;

        let s2 = TempPath::new(f1.join("sub2")).await?;
        s2.multi_folder(vec!["deep1", "deep2"]).await?;

        let res = list_folders_recursive(root.path.clone()).await?;
        assert_eq!(res.len(), 7);

        assert!(list_folders_recursive("not-a-valId_pathd").await.is_err());

        Ok(())
    }
}
