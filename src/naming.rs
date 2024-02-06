//! Functions that generate PathBuf filenames
//!
//! # Examples
//!
//! ```
//! use std::path::PathBuf;
//! use filetools::naming;
//!
//! fn main() {
//!     let custom_name = naming::generate_name("test", "pdf");
//!
//!     // Name will be suffixed by the current time it was generated
//!     let timestamped_name = naming::generate_timestamped_name("test", "pdf");
//!
//!     // Random name is a UUIDv4 string suffixed by the extension
//!     let random_name = naming::generate_random_name("pdf");
//!
//!     // N-digit name is a number prefixed by X zeros
//!     let n_digit_name = naming::generate_n_digit_name(5, 4, "pdf");
//! }
//! ```
//!

use chrono::prelude::*;
use std::path::PathBuf;
use uuid::Uuid;

/// Helper for makeing extensions
///
/// Literally just preprends a .
fn make_extension(ext: impl AsRef<str>) -> String {
    if ext.as_ref().is_empty() {
        return String::new();
    }

    format!(".{}", ext.as_ref())
}

/// Generates a `PathBuf` from a given and extension
///
/// Returns a `PathBuf` of the form `name.ext`
pub fn generate_name(name: &str, ext: &str) -> PathBuf {
    PathBuf::from(format!("{}{}", name, make_extension(ext)))
}

/// Generates a `PathBuf` from a name and extention with a default timestamp of "DD_MM_YY_HHMMSS"
/// If `fname` is "", just uses the timestamp and extension
///
/// Returns `PathBuf` in the form `fname_timestamp.ext`
pub fn generate_timestamped_name(fname: &str, ext: &str) -> PathBuf {
    let dt = UTC::now().format("%d_%m_%Y_%Hh%Mm%Ss");

    if fname.is_empty() {
        return PathBuf::from(format!("{}{}", dt, make_extension(ext)));
    }

    PathBuf::from(format!("{}_{}{}", fname, dt, make_extension(ext)))
}

/// Generates a random UUIDv4 `PathBuf`
///
/// Returns `PathBuf` in the form `uuid.ext`
pub fn generate_random_name(ext: &str) -> PathBuf {
    let unique = Uuid::new_v4();

    PathBuf::from(format!("{}{}", unique.to_string(), make_extension(ext)))
}

/// Generates a `PathBuf` from a `number` prefixed by `n_digits` zeros
///
/// Returns `PathBuf` of the form e.g `0005.ext`
pub fn generate_n_digit_name(number: usize, fill: usize, ext: &str) -> PathBuf {
    PathBuf::from(format!(
        "{:0fill$}{}",
        number,
        make_extension(ext),
        fill = fill
    ))
}

#[cfg(test)]
mod naming_tests {
    use super::*;
    use regex::Regex;
    use std::path::PathBuf;

    #[test]
    fn generates_expected_name() {
        assert_eq!(generate_name("test", "pdf"), PathBuf::from("test.pdf"));
        assert_eq!(
            generate_name("another", "txt"),
            PathBuf::from("another.txt")
        );
        assert_eq!(generate_name("main", "c"), PathBuf::from("main.c"));
        assert_eq!(generate_name("app", "js"), PathBuf::from("app.js"));
        assert_eq!(
            generate_name("somephotothing", "H4AC"),
            PathBuf::from("somephotothing.H4AC")
        );
    }

    #[test]
    // Don't judge me on regex...
    fn generates_timestamped_name_ok() {
        let ts_re = Regex::new(r"(.*)_\d{2}_\d{2}_\d{4}_\d{2}h\d{2}m\d{2}s").unwrap();
        let ts_name = generate_timestamped_name("with_filename", "txt");

        // Pathbuf checks need the full path component
        let ts_name = ts_name.to_str().unwrap();
        assert!(ts_name.starts_with("with_filename"));
        assert!(ts_re.is_match(ts_name));
        assert!(ts_name.ends_with(".txt"));

        let no_prefix_re = Regex::new(r"\d{2}_\d{2}_\d{4}_\d{2}h\d{2}m\d{2}s").unwrap();
        let no_prefix = generate_timestamped_name("", "pdf");

        let no_prefix = no_prefix.to_str().unwrap();
        assert!(no_prefix.ends_with("pdf"));
        assert!(no_prefix_re.is_match(no_prefix));
    }

    #[test]
    fn checks_random_names_are_ok() {
        let uuid_re =
            Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap();
        let rn = generate_random_name("json");
        let rn_name = rn.to_str().unwrap();
        assert!(uuid_re.is_match(rn_name));
        assert!(rn_name.ends_with(".json"));
    }
}
