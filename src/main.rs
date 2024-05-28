use crate::config::{parse_file, Root};
use crate::factory::{KPGFactory, ThreadResult, ThreadType};
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::*;
use crate::util::service_context::generate_service_context;
use actix_web::web::ServiceConfig;
use fern::colors::{Color, ColoredLevelConfig};
use lazy_static::lazy_static;
use libkplayer::bindings::exception;
use libkplayer::codec::component::media::KPMedia;
use libkplayer::codec::playlist::{KPPlayList, PlayModel};
use libkplayer::util::console::KPConsole;
use libkplayer::util::error::KPError;
use libkplayer::util::kpcodec::kpencode_parameter::KPEncodeParameterItem;
use libkplayer::util::logger::LogLevel;
use log::{error, info, Level, LevelFilter};
use std::collections::HashMap;
use std::fs::metadata;
use std::sync::mpsc::{channel, sync_channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, time::SystemTime};
use tokio::time::sleep;
use uuid::Uuid;

pub mod config;
pub mod factory;
pub mod server;
pub mod util;

#[tokio::main]
async fn main() {
    let svc = generate_service_context();
    setup_log(svc.log_level);

    // initialize
    let mut factory = KPGFactory::default();

    // create factory from config
    factory.create(&svc).await.expect("create factory failed");

    // launch threads
    factory
        .launch_message_bus(&svc)
        .await
        .expect("launch message bus failed");

    factory
        .launch_server(&svc, None)
        .await
        .expect("launch server failed");

    // factory
    //     .launch_output(None)
    //     .await
    //     .expect("launch output failed");

    // factory
    //     .launch_instance(None)
    //     .await
    //     .expect("launch instance failed");

    factory.wait(&svc).await.expect("wait for runtime");

    info!("exit success");
}

fn setup_log(level: LevelFilter) {
    libkplayer::set_log_level(
        match &level {
            LevelFilter::Error => LogLevel::Error,
            LevelFilter::Warn => LogLevel::Warn,
            LevelFilter::Info => LogLevel::Info,
            LevelFilter::Debug => LogLevel::Debug,
            LevelFilter::Trace => LogLevel::Trace,
            _ => LogLevel::Info,
        },
        Some("log/core/".to_string()),
    );

    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);
    let colors_level = colors_line.info(Color::Green);

    // fern
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[{date} {level} {target} {color_line}] \x1B[90m{message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = humantime::format_rfc3339_seconds(SystemTime::now()),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })
        .filter(|metadata| {
            let target_name = "kplayer_go";
            metadata.target().starts_with(target_name) || metadata.target() == target_name
        })
        .level(level)
        .chain(io::stdout())
        .apply()
        .unwrap();
}
