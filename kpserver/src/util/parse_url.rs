use crate::util::*;

pub fn get_url_info(source_url: &String) -> Result<(String, String, String)> {
    let url = Url::parse(source_url.as_str())?;
    if url.scheme() != KPProtocol::Rtmp.to_string() { return Err(anyhow!("can not support forward source protocol. protocol: {}", url.scheme())); };
    let address = {
        let host = match url.host() {
            None => { return Err(anyhow!("source host can not be empty")); }
            Some(h) => h,
        };
        let port = url.port().unwrap_or(1935);
        format!("{}:{}", host, port)
    };
    let (app_name, stream_name) = match url.path_segments() {
        None => { return Err(anyhow!("source app name or stream name can not be empty")); }
        Some(mut paths) => {
            let app_name = match paths.nth(0) {
                None => { return Err(anyhow!("source app name or stream name can not be empty")); }
                Some(s) => s.to_string(),
            };
            let stream_name = match paths.nth(0) {
                None => { return Err(anyhow!("source app name or stream name can not be empty")); }
                Some(s) => s.to_string()
            };
            (app_name, stream_name)
        }
    };
    Ok((address, app_name, stream_name))
}