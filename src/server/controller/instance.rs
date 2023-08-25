use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use actix_web::{get, post, HttpResponse, web};
use libkplayer::codec::component::media::KPMedia;
use libkplayer::codec::transform::KPTransform;
use libkplayer::util::console::{KPConsoleModule, KPConsolePrompt, PromptInstanceAddMediaPlayList};
use libkplayer::util::error::KPError;
use serde::{Deserialize, Serialize};
use crate::{GLOBAL_FACTORY, GLOVAL_CONSOLE};
use crate::config::ResourceType;

#[get("/instance/{name}/playlist")]
pub async fn get_instance_playlist(name: web::Path<String>) -> HttpResponse {
    let console = GLOVAL_CONSOLE.lock().unwrap();
    let receipt = match console.issue(KPConsoleModule::Instance, name.to_string(), KPConsolePrompt::InstanceGetPlayList {}) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::Ok().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[get("/instance/{name}/playlist/current")]
pub async fn get_instance_current(name: web::Path<String>) -> HttpResponse {
    let console = GLOVAL_CONSOLE.lock().unwrap();
    let receipt = match console.issue(KPConsoleModule::Instance, name.to_string(), KPConsolePrompt::InstanceCurrentMedia {}) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::Ok().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[post("/instance/{name}/playlist/skip")]
pub async fn post_instance_skip(name: web::Path<String>) -> HttpResponse {
    let console = GLOVAL_CONSOLE.lock().unwrap();
    let receipt = match console.issue(KPConsoleModule::Instance, name.to_string(), KPConsolePrompt::InstanceSkipPlayList {}) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::Ok().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[post("/instance/{name}/playlist/add")]
pub async fn add_instance_media(name: web::Path<String>, body: web::Json<PromptInstanceAddMediaPlayList>) -> HttpResponse {
    let console = GLOVAL_CONSOLE.lock().unwrap();
    let receipt = match console.issue(KPConsoleModule::Instance, name.to_string(), KPConsolePrompt::InstanceAddMediaPlayList { media: body.clone() })
    {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::Ok().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}