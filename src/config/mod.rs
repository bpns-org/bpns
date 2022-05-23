// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

mod model;

pub(crate) use model::Config;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use clap::Parser;
use dirs::home_dir;

#[cfg(feature = "matrix")]
use model::Matrix;
#[cfg(feature = "server")]
use model::Server;
#[cfg(feature = "telegram")]
use model::Telegram;
use model::{Bitcoin, ConfigFile, Core};

fn default_dir() -> PathBuf {
    let home: PathBuf = home_dir().unwrap_or_else(|| {
        log::error!("Unknown home directory");
        std::process::exit(1)
    });
    home.join(".bpns")
}

fn default_config_file() -> PathBuf {
    let mut default = default_dir().join("config");
    default.set_extension("toml");
    default
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long("conf"), name("config_file"), parse(from_os_str))]
    config_file: Option<PathBuf>,
}

impl Config {
    pub fn from_args() -> Self {
        let args: Args = Args::parse();

        let config_file_path: PathBuf = match args.config_file {
            Some(path) => path,
            None => default_config_file(),
        };

        let config_file: ConfigFile = match Self::read_config_file(&config_file_path) {
            Ok(data) => data,
            Err(error) => {
                log::error!("Impossible to read config file at {:?}", config_file_path);
                panic!("{}", error);
            }
        };

        let main_path: PathBuf = config_file.main_path.unwrap_or_else(|| default_dir());

        let config = Self {
            main_path: main_path.clone(),
            core: Core {
                db_path: main_path.join("core/db"),
            },
            #[cfg(feature = "server")]
            server: Server {
                enabled: config_file.server.enabled.unwrap_or(true),
                http_addr: config_file.server.http_addr.unwrap_or_else(|| {
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 50055)
                }),
            },
            bitcoin: Bitcoin {
                rpc_addr: config_file.bitcoin.rpc_addr.unwrap_or_else(|| {
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8332)
                }),
                rpc_username: config_file.bitcoin.rpc_username,
                rpc_password: config_file.bitcoin.rpc_password,
            },
            #[cfg(feature = "matrix")]
            matrix: Matrix {
                enabled: config_file.matrix.enabled.unwrap_or(false),
                db_path: main_path.join("matrix/db"),
                state_path: main_path.join("matrix/state"),
                homeserver_url: config_file.matrix.homeserver_url,
                proxy: config_file.matrix.proxy,
                user_id: config_file.matrix.user_id,
                password: config_file.matrix.password,
            },
            #[cfg(feature = "telegram")]
            telegram: Telegram {
                enabled: config_file.telegram.enabled.unwrap_or(false),
                db_path: main_path.join("telegram/db"),
                bot_token: config_file.telegram.bot_token,
            },
        };

        log::info!("{:?}", config);

        config
    }

    fn read_config_file(path: &Path) -> std::io::Result<ConfigFile> {
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }
}
