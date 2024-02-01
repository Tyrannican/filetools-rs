use anyhow::{Context, Result};
use std::path::{Component, Path};
use tokio::fs;

// pub mod filehelpers;
// pub mod filenaming;

// What do we want?
//
// Iterate files in a directory
// Iterate folders in a directory
//
// Iterate files in a directory + all subdirs
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
    fs::create_dir_all(dir)
        .await
        .context("unable to create directory")?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};

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
        let main = "I/am/a/path/hello/there";
        assert!(path_contains(main, "a/path"));
        assert!(!path_contains(main, "not"));

        // Check it works for paths
        let main = Path::new(main);
        assert!(path_contains(main, Path::new("a/path")));
        assert!(!path_contains(main, Path::new("not")));
    }
}
