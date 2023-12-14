use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock, TryLockResult};
use actix_web::{get, post, HttpResponse, web};
use actix_web::test::read_body;
use libkplayer::codec::component::media::KPMedia;
use libkplayer::codec::transform::KPTransform;
use libkplayer::get_global_console;
use libkplayer::util::console::*;
use libkplayer::util::console::KPConsolePrompt::{*};
use libkplayer::util::error::KPError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{GLOBAL_FACTORY, validate_and_respond_unprocessable_entity};
use crate::config::ResourceType;
use validator::{Validate, ValidationError};

// Instance
#[derive(Serialize)]
struct GetInstanceInfo {
    play_list: String,
}

#[get("/instance")]
pub async fn get_instance_list() -> HttpResponse {
    let factory = GLOBAL_FACTORY.lock().unwrap();
    let mut instance_list = HashMap::new();

    for get_instance_name in factory.get_instance_list() {
        let get_factory = factory.get_instance(&get_instance_name).unwrap();
        let get_instance = get_factory.lock().unwrap();
        instance_list.insert(get_instance_name, get_instance.clone());
    }

    HttpResponse::Ok().json(instance_list)
}

// PlayList
#[get("/instance/{name}/playlist")]
pub async fn get_instance_playlist(name: web::Path<String>) -> HttpResponse {
    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(KPConsoleModule::Transform, name.to_string(), KPConsolePrompt::TransformGetPlayList {}) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::UnprocessableEntity().json(&err);
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
            return HttpResponse::UnprocessableEntity().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[post("/instance/{name}/playlist/prev")]
pub async fn post_instance_prev(name: web::Path<String>) -> HttpResponse {
    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(KPConsoleModule::Transform, name.to_string(), KPConsolePrompt::TransformPrevPlayList {}) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::UnprocessableEntity().json(&err);
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
            return HttpResponse::UnprocessableEntity().json(&err);
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
            return HttpResponse::UnprocessableEntity().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[derive(Deserialize, Validate)]
pub struct RemoveInstanceMedia {
    #[validate(length(min = 1))]
    name: String,
}

#[post("/instance/{name}/playlist/remove")]
pub async fn remove_instance_media(name: web::Path<String>, body: web::Json<RemoveInstanceMedia>) -> HttpResponse {
    validate_and_respond_unprocessable_entity!(body);

    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(KPConsoleModule::Transform, name.to_string(), KPConsolePrompt::TransformRemoveMediaPlayList { name: body.name.clone() })
    {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::UnprocessableEntity().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[derive(Deserialize, Validate)]
pub struct MoveInstanceMedia {
    #[validate(length(min = 1))]
    name: String,
    index: usize,
}

#[post("/instance/{name}/playlist/move")]
pub async fn move_instance_media(name: web::Path<String>, body: web::Json<MoveInstanceMedia>) -> HttpResponse {
    validate_and_respond_unprocessable_entity!(body);

    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(KPConsoleModule::Transform, name.to_string(), KPConsolePrompt::TransformMoveMediaPlayList {
        name: body.name.clone(),
        index: body.index.clone(),
    }) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::UnprocessableEntity().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[derive(Deserialize, Validate)]
pub struct SeekInstanceMedia {
    seek: Option<i32>,
    end: Option<i32>,
    is_persistence: Option<bool>,
}

#[post("/instance/{name}/playlist/seek")]
pub async fn seek_instance_media(name: web::Path<String>, body: web::Json<SeekInstanceMedia>) -> HttpResponse {
    validate_and_respond_unprocessable_entity!(body);

    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(KPConsoleModule::Transform, name.to_string(), KPConsolePrompt::TransformSeekMediaPlayList {
        seek: body.seek,
        end: body.end,
        is_persistence: body.is_persistence,
    })
    {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::UnprocessableEntity().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[derive(Deserialize, Validate)]
pub struct SelectInstanceMedia {
    #[validate(length(min = 1))]
    name: String,
}

#[post("/instance/{name}/playlist/select")]
pub async fn select_instance_media(name: web::Path<String>, body: web::Json<SelectInstanceMedia>) -> HttpResponse {
    validate_and_respond_unprocessable_entity!(body);

    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(KPConsoleModule::Transform, name.to_string(), KPConsolePrompt::TransformSelectMediaPlayList {
        name: body.name.clone(),
    })
    {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::UnprocessableEntity().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

// Plugin
#[get("/instance/{name}/plugin")]
pub async fn get_instance_plugin(name: web::Path<String>) -> HttpResponse {
    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(KPConsoleModule::Transform, name.to_string(), KPConsolePrompt::TransformGetPluginList {}) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::UnprocessableEntity().json(&err);
        }
    };
    HttpResponse::Ok().json(receipt)
}

#[derive(Deserialize, Validate)]
pub struct UpdateInstancePluginArgumentBody {
    #[validate(length(min = 1))]
    app: String,
    #[validate(length(min = 1))]
    name: String,
    arguments: HashMap<String, String>,
}

#[post("/instance/{instance_name}/plugin/{plugin_name}")]
pub async fn update_instance_plugin_argument(path: web::Path<(String, String)>, body: web::Json<UpdateInstancePluginArgumentBody>) -> HttpResponse {
    validate_and_respond_unprocessable_entity!(body);

    let mut arguments = HashMap::new();
    for (key, value) in body.arguments.iter() {
        if !value.is_empty() {
            arguments.insert(key.clone(), value.clone());
        }
    }

    let (instance_name, plugin_name) = path.into_inner();
    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(
        KPConsoleModule::Transform,
        instance_name.to_string(),
        TransformUpdatePlugin {
            unique_name: Some(plugin_name.to_string()),
            app: Some(body.app.clone()),
            name: body.name.clone(),
            arguments,
        },
    ) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::UnprocessableEntity().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[get("/instance/{instance_name}/info")]
pub async fn get_instance_info(path: web::Path<String>) -> HttpResponse {
    let instance_name = path.to_string();
    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(
        KPConsoleModule::Transform,
        instance_name.to_string(),
        TransformGetBasicInfo {},
    ) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::UnprocessableEntity().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}

#[get("/instance/{instance_name}/encode_parameter")]
pub async fn get_instance_encode_parameter(path: web::Path<String>) -> HttpResponse {
    let instance_name = path.to_string();
    let global_console = get_global_console();
    let console = global_console.lock().await;
    let receipt = match console.issue(
        KPConsoleModule::Transform,
        instance_name.to_string(),
        TransformGetEncodeParameter {},
    ) {
        Ok(receipt) => receipt,
        Err(err) => {
            return HttpResponse::UnprocessableEntity().json(&err);
        }
    };

    HttpResponse::Ok().json(receipt)
}