use std::path::{Path, PathBuf};

pub fn get_homedir() -> PathBuf {
    std::env::current_dir().unwrap()
}