use std::fs;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{self, Read, Write};
use md5;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::KPGUtilReadDirectoryFailed;

pub fn read_directory_file(path: String, ext_filters: Vec<String>) -> Result<Vec<String>, KPGError> {
    let entries = fs::read_dir(path.clone()).map_err(|err| {
        KPGError::new_with_string(KPGUtilReadDirectoryFailed, format!("read directory failed. path: {:?}, error: {}", path, err))
    })?;

    let mut files = Vec::new();
    for dir_entry in entries {
        if let Ok(entry) = dir_entry {
            let file_path = entry.path();
            if let Some(ext) = file_path.extension() {
                let ext_str = ext.to_string_lossy();
                if ext_filters.iter().any(|filter| filter.clone() == ext_str) {
                    if let Some(file_name) = file_path.file_name() {
                        files.push(file_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Ok(files)
}

pub fn find_existed_file<T: ToString>(files: Vec<T>) -> Option<String> {
    let path_lists: Vec<_> = files.iter().map(|item| { item.to_string() }).collect();
    for file in &path_lists {
        if let Ok(metadata) = fs::metadata(file) {
            if metadata.is_file() {
                return Some(file.clone());
            }
        }
    }
    None
}

pub fn compare_md5(file_path: &PathBuf, expected_md5: &String) -> bool {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let mut hasher = md5::Context::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(bytes_read) => bytes_read,
            Err(e) => return false,
        };
        hasher.consume(&buffer[..bytes_read]);
    }

    let result = hasher.compute();
    let calculated_md5 = format!("{:x}", result);

    calculated_md5.eq(expected_md5)
}

pub fn download_file(url: &String, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut response = reqwest::blocking::get(url)?;

    if !response.status().is_success() {
        return Err(format!("Request failed with status code: {}", response.status()).into());
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(file_path)?;

    let mut buffer = Vec::new();
    response.read_to_end(&mut buffer)?;

    file.write_all(&buffer)?;

    Ok(())
}