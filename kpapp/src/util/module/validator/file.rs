use crate::util::module::validator::*;


const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "flv"];

pub fn exist_file(file_path: &str) -> Result<(), ValidationError> {
    if !fs::metadata(file_path).is_ok() {
        return Err(ValidationError {
            code: "file_not_exist".into(),
            message: Some(format!("File does not exist: {}", file_path).into()),
            params: [("file_path".into(), file_path.into())].iter().cloned().collect(),
        });
    }
    Ok(())
}

pub fn video_extension(file_path: &str) -> Result<(), ValidationError> {
    if let Some(extension) = std::path::Path::new(file_path).extension().and_then(|e| e.to_str()) {
        if !VIDEO_EXTENSIONS.contains(&extension) {
            return Err(ValidationError {
                code: "file_not_video".into(),
                message: Some(format!("File is not a video: {}", file_path).into()),
                params: [("file_path".into(), file_path.into())].iter().cloned().collect(),
            });
        }
    } else {
        return Err(ValidationError {
            code: "file_no_extension".into(),
            message: Some(format!("File has no extension: {}", file_path).into()),
            params: [("file_path".into(), file_path.into())].iter().cloned().collect(),
        });
    }
    Ok(())
}
