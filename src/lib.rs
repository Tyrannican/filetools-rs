//! Simple functions to help with file naming and file iteration
//! Ported from the [filetools](https://github.com/jgrizou/filetools) library written by Jonathan Grizou
//! 
//! # Examples
//! 
//! ```
//! use filetools::filehelpers;
//! use std::path::PathBuf;
//! 
//! fn main() -> Result <(), Box<dyn std::error::Error>> {
//!     /// Creating a directory
//!     let new_path = PathBuf::from("./test");
//!     let _ = filehelpers::ensure_dir(new_path)?;
//! 
//!     /// Iterating through all files in a directory
//!     let nr_search = PathBuf::from("./test");
//!     let r_search = PathBuf::from("./test");
//! 
//!     // Non-recursive search of directroy, just files in search folder
//!     let non_recursed_files = filehelpers::list_files(nr_search, false);
//! 
//!     // Recursive search of directory, gets all files in directory and all sub-directories
//!     let recursed_files = filehelpers::list_files(r_search, true);
//! 
//!     /// Iterating through all folders in a directory
//!     let nr_search = PathBuf::from("./test");
//!     let r_search = PathBuf::from("./test");
//! 
//!     // Non-recursive search for all folders, just folders in search directory
//!     let non_recursive_folders = filehelpers::list_folders(nr_search, false);
//! 
//!     // Recursive search of all folders, all subfolders in a directory as well
//!     let recursive_folders = filehelpers::list_folders(r_search, true);
//! 
//!     Ok(())
//! } 
//! ```
//! 

pub mod filenaming;
pub mod filehelpers;


#[cfg(test)]
mod tests {
    use crate::{filehelpers, filenaming};
    use std::path::PathBuf;

    #[test]
    fn iterate_files_and_folders() -> Result<(), Box<dyn std::error::Error>> {
        let files = filehelpers::list_files(PathBuf::from("src"), true)?;
        let folders = filehelpers::list_folders(PathBuf::from("."), false)?;

        // filehelpers.rs filenaming.rs lib.rs
        assert_eq!(files.len(), 3);

        // target/ src/ .git/
        assert_eq!(folders.len(), 4);
        Ok(())
    }

    #[test]
    fn folder_creation() {
        let _ = filehelpers::ensure_dir(PathBuf::from("./test/func"));
    }

    #[test]
    fn generate_filenames() -> Result<(), Box<dyn std::error::Error>> {
        let name1 = filenaming::generate_default_timestamped_name("", ".pdf");
        let name2 = filenaming::generate_default_timestamped_name("test_file", ".dxf");
        let name3 = filenaming::generate_random_name(".docx");
        let name4 = filenaming::generate_n_digit_name(55, 6, ".pdf");

        println!("Name1: {:?}", name1);
        println!("Name2: {:?}", name2);
        println!("Name3: {:?}", name3);
        println!("Name4: {:?}", name4);

        assert_eq!(name4, PathBuf::from("000055.pdf"));

        Ok(())
    }

    #[test]
    fn path_does_contains() -> Result<(), Box<dyn std::error::Error>> {
        let path1 = PathBuf::from("./target/doc/cfg_if");
        let path2 = PathBuf::from("./target/chrono/datetime");
        let path3 = PathBuf::from("./target");

        let target_paths: Vec<PathBuf> = filehelpers::list_files(path3, true)?
                .into_iter()
                .filter(|x| filehelpers::path_contains(x.to_path_buf(), "doc"))
                .collect();

        assert_eq!(filehelpers::path_contains(path1, "doc"), true);
        assert_eq!(filehelpers::path_contains(path2, "debug"), false);

        for path in target_paths.iter() {
            assert_eq!(filehelpers::path_contains(path.to_path_buf(), "doc"), true);
        }

        Ok(())
    }
}
