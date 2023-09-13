use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use actix_web::{get, post, HttpResponse, web};
use libkplayer::codec::component::media::KPMedia;
use libkplayer::codec::transform::KPTransform;
use libkplayer::get_global_console;
use libkplayer::util::console::{KPConsoleModule, KPConsolePrompt, PromptTransformAddMediaPlayList};
use libkplayer::util::error::KPError;
use serde::{Deserialize, Serialize};
use crate::{GLOBAL_FACTORY};
use crate::config::ResourceType;

#[get("/instance/{name}/playlist")]
pub async fn get_instance_playlist(name: web::Path<String>) -> HttpResponse {
    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(KPConsoleModule::Transform, name.to_string(), KPConsolePrompt::TransformGetPlayList {}) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::Ok().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[get("/instance/{name}/playlist/current")]
pub async fn get_instance_current(name: web::Path<String>) -> HttpResponse {
    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(KPConsoleModule::Transform, name.to_string(), KPConsolePrompt::TransformCurrentMedia {}) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::Ok().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[post("/instance/{name}/playlist/skip")]
pub async fn post_instance_skip(name: web::Path<String>) -> HttpResponse {
    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(KPConsoleModule::Transform, name.to_string(), KPConsolePrompt::TransformSkipPlayList {}) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::Ok().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[post("/instance/{name}/playlist/add")]
pub async fn add_instance_media(name: web::Path<String>, body: web::Json<PromptTransformAddMediaPlayList>) -> HttpResponse {
    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(KPConsoleModule::Transform, name.to_string(), KPConsolePrompt::TransformAddMediaPlayList { media: body.clone() })
    {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::Ok().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}