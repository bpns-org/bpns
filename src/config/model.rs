// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Core {
    pub db_path: PathBuf,
}

#[cfg(feature = "server")]
#[derive(Deserialize)]
pub struct Server {
    pub enabled: bool,
    pub http_addr: SocketAddr,
}

#[cfg(feature = "server")]
#[derive(Deserialize)]
pub struct ConfigFileServer {
    pub enabled: Option<bool>,
    pub http_addr: Option<SocketAddr>,
}

#[derive(Deserialize)]
pub struct Bitcoin {
    pub rpc_addr: SocketAddr,
    pub rpc_username: String,
    pub rpc_password: String,
}

#[derive(Deserialize)]
pub struct ConfigFileBitcoin {
    pub rpc_addr: Option<SocketAddr>,
    pub rpc_username: String,
    pub rpc_password: String,
}

#[cfg(feature = "matrix")]
#[derive(Deserialize)]
pub struct Matrix {
    pub enabled: bool,
    pub db_path: PathBuf,
    pub state_path: PathBuf,
    pub homeserver_url: String,
    pub proxy: Option<String>,
    pub user_id: String,
    pub password: String,
}

#[cfg(feature = "matrix")]
#[derive(Deserialize)]
pub struct ConfigFileMatrix {
    pub enabled: Option<bool>,
    pub homeserver_url: String,
    pub proxy: Option<String>,
    pub user_id: String,
    pub password: String,
}

#[cfg(feature = "telegram")]
#[derive(Deserialize)]
pub struct Telegram {
    pub enabled: bool,
    pub db_path: PathBuf,
    pub bot_token: String,
}

#[cfg(feature = "telegram")]
#[derive(Deserialize)]
pub struct ConfigFileTelegram {
    pub enabled: Option<bool>,
    pub bot_token: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub main_path: PathBuf,
    pub core: Core,
    #[cfg(feature = "server")]
    pub server: Server,
    pub bitcoin: Bitcoin,
    #[cfg(feature = "matrix")]
    pub matrix: Matrix,
    #[cfg(feature = "telegram")]
    pub telegram: Telegram,
}

#[derive(Deserialize)]
pub struct ConfigFile {
    pub main_path: Option<PathBuf>,
    #[cfg(feature = "server")]
    pub server: ConfigFileServer,
    pub bitcoin: ConfigFileBitcoin,
    #[cfg(feature = "matrix")]
    pub matrix: ConfigFileMatrix,
    #[cfg(feature = "telegram")]
    pub telegram: ConfigFileTelegram,
}

impl fmt::Debug for Core {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ db_path: {:?} }}", self.db_path)
    }
}

#[cfg(feature = "server")]
impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ enabled: {}, http_addr: {} }}",
            self.enabled, self.http_addr
        )
    }
}

impl fmt::Debug for Bitcoin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ rpc_addr: {:?}, rpc_username: {} }}",
            self.rpc_addr, self.rpc_username
        )
    }
}

#[cfg(feature = "matrix")]
impl fmt::Debug for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ enabled: {}, db_path: {:?}, state_path: {:?}, homeserver_url: {}, proxy: {:?}, user_id: {} }}", self.enabled,
            self.db_path, self.state_path, self.homeserver_url, self.proxy, self.user_id
        )
    }
}

#[cfg(feature = "telegram")]
impl fmt::Debug for Telegram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ enabled: {}, db_path: {:?} }}",
            self.enabled, self.db_path
        )
    }
}
