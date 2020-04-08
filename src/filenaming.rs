//! Functions that generate PathBuf filenames
//! 
//! # Examples
//! 
//! ```
//! use std::path::PathBuf;
//! use filetools::filenaming::FileNaming;
//! 
//! fn main() {
//!     let custom_name = FileNaming::generate_name("test", ".pdf");
//!     assert_eq!(custom_name, PathBuf::from("test.pdf"));
//! 
//!     // Name will be suffixed by the current time it was generated
//!     let timestamped_name = FileNaming::generate_timestamped_name("test", ".pdf");
//! 
//!     // Random name is a UUIDv4 string suffixed by the extension
//!     let random_name = FileNaming::generate_random_name(".pdf");
//! 
//!     // N-digit name is a number prefixed by X zeros
//!     let n_digit_name = FileNaming::generate_n_digit_name(5, 4, ".pdf");
//!     assert_eq!(n_digit_name, PathBuf::from("0005.pdf"));
//! }
//! ```
//! 

use std::path::PathBuf;
use uuid::Uuid;
use chrono::prelude::*;

pub struct FileNaming;

impl FileNaming {
    /// Generates a `PathBuf` from a given and extension
    /// 
    /// Returns a `PathBuf` of the form `name.ext`
    pub fn generate_name(name: &str, ext: &str) -> PathBuf {
        PathBuf::from(format!("{}{}", name, ext))
    }

    /// Generates a `PathBuf` from a name and extention with a timestamp
    /// If `f_name` is "", just uses the timestamp and extension
    /// 
    /// Returns `PathBuf` in the form `f_name_timestamp.ext`
    pub fn generate_timestamped_name(fname: &str, ext: &str) -> PathBuf {
        let dt = UTC::now().format("%d_%m_%Y_%Hh%Mm%Ss");
        
        if fname == "" {
            PathBuf::from(format!("{}{}", dt, ext))
        } else {
            PathBuf::from(format!("{}_{}{}", fname, dt, ext))
        }
    }

    /// Generates a random UUIDv4 `PathBuf`
    /// 
    /// Returns `PathBuf` in the form `uuid.ext`
    pub fn generate_random_name(ext: &str) -> PathBuf {
        let unique = Uuid::new_v4();

        PathBuf::from(format!("{}{}", unique.to_string(), ext))
    }

    /// Generates a `PathBuf` from a `number` prefixed by `n_digits` zeros
    /// 
    /// Returns `PathBuf` of the form e.g `0005.ext`
    pub fn generate_n_digit_name(number: i32, n_digits: usize, ext: &str) -> PathBuf {
        PathBuf::from(format!("{:0fill$}{}", number, ext, fill=n_digits))
    }
}