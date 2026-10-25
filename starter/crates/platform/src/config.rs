use std::env;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub namespace: String,
    pub collection: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let endpoint = env::var("LOAMS_ENDPOINT").unwrap_or_default();
        if endpoint.trim().is_empty() {
            return Err(ConfigError::Missing("LOAMS_ENDPOINT"));
        }
        let bind_addr = env::var("BIND_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3000".to_owned())
            .parse()?;
        let namespace = env::var("LOAMS_NAMESPACE").unwrap_or_else(|_| "company".to_owned());
        let collection = env::var("LOAMS_COLLECTION").unwrap_or_else(|_| "notes".to_owned());
        Ok(Self {
            bind_addr,
            namespace,
            collection,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing environment variable {0}")]
    Missing(&'static str),
    #[error("invalid BIND_ADDR: {0}")]
    BindAddr(#[from] std::net::AddrParseError),
}
