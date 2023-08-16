use std::sync::mpsc::{channel, sync_channel};
use fern::colors::{Color, ColoredLevelConfig};
use libkplayer::bindings::exception;
use libkplayer::codec::component::media::KPMedia;
use libkplayer::codec::playlist::{KPPlayList, PlayModel};
use libkplayer::server::media_server::KPMediaServer;
use libkplayer::util::error::KPError;
use libkplayer::util::kpcodec::kpencode_parameter::{KPEncodeParameterItem};
use libkplayer::util::logger::LogLevel;
use uuid::Uuid;
use crate::config::{parse_file, Root};
use crate::factory::{KPGFactory, ThreadResult};
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::*;
use log::{LevelFilter, info, Level, error};
use std::{io, time::SystemTime};
use std::collections::HashMap;
use std::fs::metadata;
use std::time::Duration;

pub mod config;
pub mod util;
pub mod factory;
pub mod server;

fn main() {
    setup_log(LevelFilter::Info);

    // load config
    let cfg = match parse_file() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("parse config file failed. {}",err);
            return;
        }
    };

    // initialize factory
    let mut factory = KPGFactory::new();

    // connect message bus
    factory.launch_message_bus();

    // create item from config
    factory.create(cfg).expect("create factory failed");

    // launch server items
    factory.launch_server();
    factory.launch_instance();

    // wait quit signal
    loop {
        match factory.wait() {
            Ok(_) => {
                let mut need_break = true;
                for (_, value) in factory.check_instance_survival() {
                    if value {
                        need_break = false;
                    }
                }

                if need_break {
                    break;
                }
            }
            Err(err) => {
                err.expect("thread result receive exception")
            }
        }
    }

    info!("exit success");
}

fn setup_log(level: LevelFilter) {
    libkplayer::set_log_level(match &level {
        LevelFilter::Error => LogLevel::Error,
        LevelFilter::Warn => LogLevel::Warn,
        LevelFilter::Info => LogLevel::Info,
        LevelFilter::Debug => LogLevel::Debug,
        LevelFilter::Trace => LogLevel::Trace,
        _ => LogLevel::Info,
    }, Some("log/core/".to_string()));

    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);
    let colors_level = colors_line.info(Color::Green);

    // fern
    fern::Dispatch::new().
        format(move |out, message, record| {
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
        }).
        filter(|metadata| {
            let target_name = "kplayer_go";
            metadata.target().starts_with(target_name) || metadata.target() == target_name
        }).
        level(level).
        chain(io::stdout()).apply().unwrap();
}