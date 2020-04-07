use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};

pub struct FileHelpers;

impl FileHelpers {
    pub fn ensure_dir(dir_name: &str) -> Result <()> {
        let path = Path::new(dir_name);
        if !path.exists() {
            fs::create_dir(path)?;
        }

        Ok(())
    }

    pub fn is_subdir(path: &str, directory: &str) -> Result<bool> {
        let directory = fs::canonicalize(Path::new(directory))?;
        let path = fs::canonicalize(Path::new(path))?;

        let mut is_subdir = Ok(false);
        println!("Abs: {:?}", directory);
        
        for ancestor in path.ancestors() {
            println!("Ancestor: {:?}", ancestor);
            if ancestor == directory {
                is_subdir = Ok(true);
                break;
            }
        }

        is_subdir
    }

    pub fn list_files(path: &str) -> Result<Vec<PathBuf>> {
        let mut found_files = Vec::new();
        let search_path = Path::new(path);

        for entry in fs::read_dir(search_path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::metadata(&path)?;

            if metadata.is_file() {
                found_files.push(path);
            } else if metadata.is_dir() {
                if let Some(subpath) = path.to_str() {
                    let subfiles = FileHelpers::list_files(subpath)?;

                    for file in subfiles.iter() {
                        found_files.push(file.to_path_buf());
                    }
                }
            } else {
                continue;
            }
        }

        Ok(found_files)
    }

    pub fn list_folders(path: &str) -> Result<Vec<PathBuf>> {
        let mut found_folders = Vec::new();
        let search_path = Path::new(path);

        for entry in fs::read_dir(search_path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::metadata(&path)?;

            if metadata.is_dir() {
                found_folders.push(path);
            }
        }

        Ok(found_folders)
    }
}