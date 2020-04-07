use std::path::PathBuf;
use uuid::Uuid;
use chrono::prelude::*;

pub struct FileNaming;

impl FileNaming {
    pub fn generate_timestamped_name(fname: &str, ext: &str) -> PathBuf {
        let dt = UTC::now().format("%d_%m_%Y_%Hh%Mm%Ss");
        
        if fname == "" {
            PathBuf::from(format!("{}{}", dt, ext))
        } else {
            PathBuf::from(format!("{}_{}{}", fname, dt, ext))
        }
    }

    pub fn generate_random_name(ext: &str) -> PathBuf {
        let unique = Uuid::new_v4();

        PathBuf::from(format!("{}{}", unique.to_string(), ext))
    }

    pub fn generate_n_digit_name(number: i32, n_digits: usize, ext: &str) -> PathBuf {
        PathBuf::from(format!("{:0fill$}{}", number, ext, fill=n_digits))
    }
}