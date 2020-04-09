//! Functions that help in iterating files and folders
//! 
//! # Examples
//! 
//! ```
//! use std::path::PathBuf;
//! use std::io::Result;
//! use filetools::filehelpers::FileHelpers;
//! 
//! fn main() -> Result<()> {
//!     /// Creating a directory
//!     let new_path = PathBuf::from("./test");
//!     let _ = FileHelpers::ensure_dir(new_path)?;
//! 
//!     /// Iterating through all files in a directory
//!     let nr_search = PathBuf::from("./test");
//!     let r_search = PathBuf::from("./test");
//! 
//!     // Non-recursive search of directroy, just files in search folder
//!     let non_recursed_files = FileHelpers::list_files(nr_search, false);
//! 
//!     // Recursive search of directory, gets all files in directory and all sub-directories
//!     let recursed_files = FileHelpers::list_files(r_search, true);
//! 
//!     /// Iterating through all folders in a directory
//!     let nr_search = PathBuf::from("./test");
//!     let r_search = PathBuf::from("./test");
//! 
//!     // Non-recursive search for all folders, just folders in search directory
//!     let non_recursive_folders = FileHelpers::list_folders(nr_search, false);
//! 
//!     // Recursive search of all folders, all subfolders in a directory as well
//!     let recursive_folders = FileHelpers::list_folders(r_search, true);
//! 
//!     Ok(())
//! }
//! ```
//! 

use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};

pub struct FileHelpers;

impl FileHelpers {
    /// Ensures a directory is created from a `PathBuf`
    /// Does nothing if the directory already exists
    /// 
    /// Returns `Ok` if successful, `Err` if not
    pub fn ensure_dir(dir_name: PathBuf) -> Result <()> {
        let path = Path::new(&dir_name);
        if !path.exists() {
            fs::create_dir(path)?;
        }

        Ok(())
    }

    /// Determines if a `path` if a subdirectory of the given `directory`
    /// Creates the absolute paths and checks the `ancestors` of `path` to determine if a subdirectory
    /// 
    /// Note::Not entirely sure this works perfectly fine, use at own risk
    /// 
    /// Returns `Ok(true)` if `path` is a subdirectory, `Ok(false)` if not, `Err` if error occured
    pub fn is_subdir(path: PathBuf, directory: PathBuf) -> Result<bool> {
        // Get absolute paths
        let directory = fs::canonicalize(Path::new(&directory))?;
        let path = fs::canonicalize(Path::new(&path))?;

        let mut is_subdir = Ok(false);
        
        // Iterate through all ancestors of the path
        for ancestor in path.ancestors() {
            // Found directory, current path is a subdirectory
            if ancestor == directory {
                is_subdir = Ok(true);
                break;
            }
        }

        is_subdir
    }

    /// Determines if a given `PathBuf` contains a search string
    /// 
    /// Returns `true` if search string present, else `false`
    pub fn path_contains(path: PathBuf, search_str: &str) -> bool {
        // Path successfully converted to str
        if let Some(p) = path.to_str() {
            // Contains string, return true
            if p.contains(search_str) {
                return true
            }
        }

        // Search string not found
        false
    }

    /// Lists all files in a given `path`
    /// If `recursive` is set, iterates through all subfolders recursively to find all files
    /// If `recursive` not set, just finds all files in the current directory
    /// 
    /// Return `Vec<PathBuf>` of all files in a directory and subdirectories
    pub fn list_files(path: PathBuf, recursive: bool) -> Result<Vec<PathBuf>> {
        let mut found_files = Vec::new();
        let search_path = Path::new(&path);

        // Iterate through all entries in the directory
        for entry in fs::read_dir(search_path)? {
            // Get File metadata
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::metadata(&path)?;

            // Entry is a file, add to array
            if metadata.is_file() {
                found_files.push(path);
            } else if metadata.is_dir() && recursive {
                // Found a directory and recursively looking 
                let subfiles = FileHelpers::list_files(path, recursive)?;

                // Add all found subfiles to array
                for file in subfiles.iter() {
                    found_files.push(file.to_path_buf());
                }
            } else {
                continue;
            }
        }

        Ok(found_files)
    }

    /// Lists all folders in a given `path`
    /// If `recursive` is set, iterates through all subfolders recursively to find all folders
    /// If `recursive` not set, just finds all files in the current directory
    /// Mirrors the functionality of `FileHelpers::list_files()`
    /// 
    /// Return `Vec<PathBuf>` of all folders in a directory and subdirectories
    pub fn list_folders(path: PathBuf, recursive: bool) -> Result<Vec<PathBuf>> {
        let mut found_folders = Vec::new();
        let search_path = Path::new(&path);

        // Iterate through all entries in the directory
        for entry in fs::read_dir(search_path)? {
            // Get File metadata
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::metadata(&path)?;

            // Entry is a directory, add to array
            if metadata.is_dir() {
                found_folders.push(path);

                // Recursively looking
                if recursive {
                    // Search recursively
                    let f_path = entry.path();
                    let subfolders = FileHelpers::list_folders(f_path, recursive)?;

                    // Add all subfolders to array
                    for subfolder in subfolders.iter() {
                        found_folders.push(subfolder.to_path_buf());
                    }
                }
            }
        }

        Ok(found_folders)
    }
}