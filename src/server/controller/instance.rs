use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use actix_web::{HttpResponse};
use libkplayer::codec::component::media::KPMedia;
use libkplayer::codec::transform::KPTransform;
use libkplayer::util::console::{KPConsoleModule, KPConsolePrompt};
use libkplayer::util::error::KPError;
use crate::{GLOBAL_FACTORY, GLOVAL_CONSOLE};

pub async fn get_instance_playlist(name: String) -> HttpResponse {
    let console = GLOVAL_CONSOLE.lock().unwrap();
    let receipt = match console.issue(KPConsoleModule::Instance, name, KPConsolePrompt::InstanceGetPlayList {}) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::Ok().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

pub async fn post_instance_skip(name: String) -> HttpResponse {
    let console = GLOVAL_CONSOLE.lock().unwrap();
    let receipt = match console.issue(KPConsoleModule::Instance, name, KPConsolePrompt::InstanceSkipPlayList {}) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::Ok().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}