use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::KPGJsonRPCFailed;

#[derive(Serialize)]
struct JsonRPCBody<D> where D: Serialize {
    id: i64,
    method: String,
    params: Vec<D>,
}

pub fn jsonrpc_call<T: ToString, D: Serialize>(url: String, method: T, params: Vec<D>) -> Result<String, KPGError> {
    let client = Client::new();
    let body = JsonRPCBody {
        id: 0,
        method: method.to_string(),
        params,
    };

    let body_str = serde_json::to_string(&body).unwrap();

    // request
    let response = match client.post(url.to_string()).body(body_str).send() {
        Ok(res) => res,
        Err(err) => {
            return Err(KPGError::new_with_string(KPGJsonRPCFailed, format!("call json-rpc failed. error: {}", err)));
        }
    };
    if !response.status().is_success() {
        return Err(KPGError::new_with_string(KPGJsonRPCFailed, format!("call json-rpc failed. status: {}", response.status())));
    }
    let response_data = match response.text() {
        Ok(data) => data,
        Err(err) => {
            return Err(KPGError::new_with_string(KPGJsonRPCFailed, format!("call json-rpc failed. error: {}", err)));
        }
    };

    // parse
    let s = serde_json::from_str::<Value>(&response_data).unwrap();
    if let Some(error) = s.get("error") {
        if !error.is_null() {
            return Err(KPGError::new_with_string(KPGJsonRPCFailed, format!("json-rpc return error. error: {}", error)));
        }
    }

    if let Some(result) = s.get("result") {
        return Ok(result.to_string());
    }

    return Err(KPGError::new_with_string(KPGJsonRPCFailed, format!("invalid json-rpc response body. body: {}", response_data)));
}