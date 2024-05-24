pub mod api;
pub mod instance;
pub mod playlist;

use actix_web::Responder;
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
