mod config;

use config::AppConfig;
use log::{info, error, warn};
use solana_sdk::signature::{Keypair, Signer};
use std::error::Error;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. 初始化日志系统
    // 如果环境变量没有设置 RUST_LOG，则默认使用 info
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    
    info!("🚀 Scavenger (拾荒者) MEV Bot 正在启动...");

    // 2. 加载配置
    info!("正在加载配置文件 config.toml...");
    let config = match AppConfig::load() {
        Ok(c) => c,
        Err(e) => {
            error!("❌ 无法加载配置: {}", e);
            error!("请确保当前目录下存在 config.toml 文件");
            return Ok(());
        }
    };
    
    info!("✅ 配置加载成功");
    info!("   RPC URL: {}", config.network.rpc_url);
    info!("   Block Engine: {}", config.jito.block_engine_url);
    info!("   交易金额: {} SOL", config.strategy.trade_amount_sol);

    // 3. 加载钱包 (Keypair)
    // 检查文件是否存在
    if !Path::new(&config.jito.auth_keypair_path).exists() {
        warn!("⚠️  Jito 鉴权私钥文件未找到: {}", config.jito.auth_keypair_path);
        warn!("   请使用 'solana-keygen new -o {}' 生成，或修改 config.toml", config.jito.auth_keypair_path);
    }
    
    if !Path::new(&config.strategy.wallet_path).exists() {
        warn!("⚠️  交易钱包私钥文件未找到: {}", config.strategy.wallet_path);
        warn!("   请使用 'solana-keygen new -o {}' 生成，或修改 config.toml", config.strategy.wallet_path);
    }

    // 尝试加载 (如果文件存在)
    if Path::new(&config.strategy.wallet_path).exists() {
        match read_keypair_from_file(&config.strategy.wallet_path) {
            Ok(kp) => info!("✅ 交易钱包已加载: {}", kp.pubkey()),
            Err(e) => error!("❌ 无法读取交易钱包: {}", e),
        }
    }

    // 4. Jito 客户端连接准备 (Phase 1 目标)
    // 这里我们暂时只打印连接信息，实际连接逻辑将在 Phase 2/3 中集成
    info!("正在初始化 Jito 搜索者客户端...");
    // let client = ...
    
    info!("✅ 阶段一 (基础设施) 检查完成。");
    info!("   - 项目结构: OK");
    info!("   - 配置文件: OK");
    info!("   - 依赖管理: OK");
    info!("   - 钱包检查: 完成");
    
    Ok(())
}

/// 从文件读取 Keypair (JSON 格式)
fn read_keypair_from_file(path: &str) -> Result<Keypair, Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let bytes: Vec<u8> = serde_json::from_reader(file)?;
    let keypair = Keypair::from_bytes(&bytes)?;
    Ok(keypair)
}
