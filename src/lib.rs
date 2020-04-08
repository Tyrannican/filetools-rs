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
    fn iterate_files_and_folders() -> Result<()>{
        let files = FileHelpers::list_files(PathBuf::from("src"))?;
        let folders = FileHelpers::list_folders(PathBuf::from("."))?;

        // filehelpers.rs filenaming.rs lib.rs
        assert_eq!(files.len(), 3);

        // target/ src/ .git/
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
