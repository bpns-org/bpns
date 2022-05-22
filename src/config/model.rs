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
    pub http_addr: SocketAddr,
}

#[cfg(feature = "server")]
#[derive(Deserialize)]
pub struct ConfigFileServer {
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
    pub homeserver_url: String,
    pub proxy: Option<String>,
    pub user_id: String,
    pub password: String,
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
}

#[derive(Deserialize)]
pub struct ConfigFile {
    pub main_path: Option<PathBuf>,
    #[cfg(feature = "server")]
    pub server: ConfigFileServer,
    pub bitcoin: ConfigFileBitcoin,
    #[cfg(feature = "matrix")]
    pub matrix: ConfigFileMatrix,
}

impl fmt::Debug for Core {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ db_path: {:?} }}", self.db_path)
    }
}

#[cfg(feature = "server")]
impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ http_addr: {} }}", self.http_addr)
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
            "{{ db_path: {:?}, state_path: {:?}, homeserver_url: {}, proxy: {:?}, user_id: {} }}",
            self.db_path, self.state_path, self.homeserver_url, self.proxy, self.user_id
        )
    }
}
