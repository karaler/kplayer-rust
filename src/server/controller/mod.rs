pub mod playlist;
pub mod instance;

use std::collections::HashMap;
use actix_web::{HttpResponse, Responder};
use validator::Validate;

#[macro_export]
macro_rules! validate_and_respond_unprocessable_entity {
    ($body:expr) => {
        match $body.validate() {
            Ok(_) => {}
            Err(err) => {
                return HttpResponse::UnprocessableEntity().json(json!({"error": err.to_string()})).into();
            }
        }
    };
}