use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;
use clap::{arg, command, crate_version, Arg, Command};

pub const HOMEDIR_FLAG: &str = "home";
pub const CONFIG_PATH_FLAG: &str = "config";
pub const LOGLEVEL_FLAG: &str = "log_level";

pub fn cli() -> Command {
    Command::new("kplayer")
        .version(crate_version!())
        .arg(
            Arg::new(HOMEDIR_FLAG)
                .long("home")
                .value_name("DIR")
                .env("KP_HOMEDIR")
                .help("Set the home directory"),
        )
        .arg(
            Arg::new(CONFIG_PATH_FLAG)
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Set the configuration file path"),
        )
        .arg(
            Arg::new("log_level")
                .long("log_level")
                .value_name("LEVEL")
                .default_value("info")
                .value_parser(["trace", "debug", "info", "warn", "error"])
                .help("Set the level of logging"),
        )
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use log::info;
    use crate::cmd::cli::cli;

    #[test]
    fn test_cli() -> Result<()> {
        let cmd = cli();
        let matches = cmd.get_matches_from(vec!["kplayer"]);
        info!("{:?}", matches);
        Ok(())
    }
}