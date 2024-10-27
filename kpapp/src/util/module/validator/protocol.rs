use validator::ValidationError;

pub fn rtmp_url(url: &str) -> Result<(), ValidationError> {
    if url.starts_with("rtmp://") {
        Ok(())
    } else {
        Err(ValidationError {
            code: "url_not_rtmp".into(),
            message: Some(format!("URL is not an RTMP URL: {}", url).into()),
            params: [("url".into(), url.into())].iter().cloned().collect(),
        })
    }
}

pub fn file_url(url: &str) -> Result<(), ValidationError> {
    if url.starts_with("file://") {
        Ok(())
    } else {
        Err(ValidationError {
            code: "url_not_file".into(),
            message: Some(format!("URL is not a file URL: {}", url).into()),
            params: [("url".into(), url.into())].iter().cloned().collect(),
        })
    }
}


pub fn rtmp_or_file_url(url: &str) -> Result<(), ValidationError> {
    if url.starts_with("rtmp://") || url.starts_with("file://") {
        Ok(())
    } else {
        Err(ValidationError {
            code: "url_not_rtmp_or_file".into(),
            message: Some(format!("URL is not an RTMP or file URL: {}", url).into()),
            params: [("url".into(), url.into())].iter().cloned().collect(),
        })
    }
}