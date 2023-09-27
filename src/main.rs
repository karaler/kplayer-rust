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
use crate::factory::{KPGFactory, ThreadResult, ThreadType};
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::*;
use log::{LevelFilter, info, Level, error};
use std::{io, time::SystemTime};
use std::collections::HashMap;
use std::fs::metadata;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use lazy_static::lazy_static;
use libkplayer::util::console::KPConsole;

pub mod config;
pub mod util;
pub mod factory;
pub mod server;

lazy_static! {
    static ref GLOBAL_FACTORY: Arc<Mutex<KPGFactory>> = Arc::new(Mutex::new(KPGFactory::new()));
}


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

    // initialize
    // thread_result_map(receive thread result, is_return)
    let mut thread_result_map: HashMap<ThreadResult, bool> = HashMap::new();
    let exit_receiver = {
        let mut factory = GLOBAL_FACTORY.lock().unwrap();

        // connect message bus
        factory.launch_message_bus();

        // create item from config
        factory.create(cfg).expect("create factory failed");

        // launch threads
        {
            for name in factory.get_server_list().iter() {
                let thread_result = factory.launch_server(name).expect("launch server failed");
                thread_result_map.insert(thread_result, false);
            }
            for name in factory.get_instance_list().iter() {
                let thread_result = factory.launch_instance(name).expect("launch instance failed");
                thread_result_map.insert(thread_result, false);
            }
            for name in factory.get_output_list().iter() {
                let thread_result = factory.launch_output(name).expect("launch output failed");
                thread_result_map.insert(thread_result, false);
            }
        }

        factory.get_exit_receiver()
    };

    // wait quit signal
    loop {
        match exit_receiver.lock().unwrap().recv() {
            Ok((thread_result, result)) => {
                info!("thread exit: {:?}, result: {:?}",thread_result, result);
                if result.is_err() {
                    panic!("thread exit exception. err: {:?}", result);
                }
                match thread_result_map.get_mut(&thread_result) {
                    None => {
                        panic!("thread not register. thread_result: {:?}", thread_result);
                    }
                    Some(status) => {
                        if *status {
                            panic!("thread duplicate notice. thread_result: {:?}", thread_result);
                        }
                        *status = true;
                    }
                };

                // exit condition
                {
                    let viable_instance: Vec<bool> = thread_result_map.iter().map(|(filter_thread_result, filter_result)| {
                        filter_thread_result.thread_type == ThreadType::Instance && !*filter_result
                    }).filter(|item| {
                        *item
                    }).collect();
                    if viable_instance.len() == 0 {
                        break;
                    }

                    // api thread exit
                    if thread_result.thread_type == ThreadType::Server || thread_result.thread_type == ThreadType::Output {
                        break;
                    }
                }
            }
            Err(err) => {
                panic!("exit receiver failed. err: {}", err);
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