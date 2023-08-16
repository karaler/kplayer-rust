use actix_web::{HttpResponse, Responder};

pub async fn get_playlist_list() -> HttpResponse {
    HttpResponse::Ok().body("hello")
}