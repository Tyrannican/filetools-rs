//! Simple functions to help with file naming and file iteration
//! Ported from the [filetools](https://github.com/jgrizou/filetools) library written by Jonathan Grizou
//! 
//! # Examples
//! 
//! ```
//! use filetools::filehelpers::FileHelpers;
//! use std::io::Result;
//! use std::path::PathBuf;
//! 
//! fn main() -> Result <()> {
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

pub mod filenaming;
pub mod filehelpers;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Result;
    use filehelpers::FileHelpers;
    use filenaming::FileNaming;
    use std::path::PathBuf;

    #[test]
    fn iterate_files_and_folders() -> Result<()> {
        let files = FileHelpers::list_files(PathBuf::from("src"), true)?;
        let folders = FileHelpers::list_folders(PathBuf::from("."), false)?;

        // filehelpers.rs filenaming.rs lib.rs
        assert_eq!(files.len(), 3);

        // target/ src/ .git/ test/
        assert_eq!(folders.len(), 4);
        Ok(())
    }

    #[test]
    fn folder_creation() {
        let _ = FileHelpers::ensure_dir(PathBuf::from("./test/func"));
    }

    #[test]
    fn subdir_test() -> Result<()> {
        let f = FileHelpers::is_subdir(PathBuf::from("./test/func"), PathBuf::from("./test"))?;
        assert!(f, true);

        Ok(())
    }

    #[test]
    fn generate_filenames() -> Result<()> {
        let name1 = FileNaming::generate_timestamped_name("", ".pdf");
        let name2 = FileNaming::generate_timestamped_name("test_file", ".dxf");
        let name3 = FileNaming::generate_random_name(".docx");
        let name4 = FileNaming::generate_n_digit_name(55, 6, ".pdf");

        println!("Name1: {:?}", name1);
        println!("Name2: {:?}", name2);
        println!("Name3: {:?}", name3);
        println!("Name4: {:?}", name4);

        assert_eq!(name4, PathBuf::from("000055.pdf"));

        Ok(())
    }
}
