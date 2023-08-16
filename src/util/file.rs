use std::fs;
use std::path::Path;
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