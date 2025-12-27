use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub grpc_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JitoConfig {
    pub block_engine_url: String,
    pub auth_keypair_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StrategyConfig {
    pub wallet_path: String,
    pub trade_amount_sol: f64,
    pub static_tip_sol: f64,
    pub dynamic_tip_ratio: f64,
    pub max_tip_sol: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub network: NetworkConfig,
    pub jito: JitoConfig,
    pub strategy: StrategyConfig,
    pub log: LogConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let s = Config::builder()
            // 加载根目录下的 config.toml
            .add_source(File::with_name("config"))
            .build()?;

        s.try_deserialize()
    }

    pub fn load_from_path(path: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(path))
            .build()?;

        s.try_deserialize()
    }
}
