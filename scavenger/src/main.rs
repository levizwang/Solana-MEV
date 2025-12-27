use scavenger_lib::{config, scout, strategy};
use config::AppConfig;
use scout::Scout;
use log::{info, error, warn};
use solana_sdk::signature::{Keypair, Signer};
use solana_client::rpc_client::RpcClient;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. 初始化日志系统
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    
    info!("🚀 Scavenger (拾荒者) MEV Bot 正在启动...");
    
    // 初始化策略模块
    strategy::init();

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
    
    // 3. 初始化 RPC 客户端并检查连接
    info!("正在连接 RPC 节点: {}", config.network.rpc_url);
    let rpc_client = Arc::new(RpcClient::new(config.network.rpc_url.clone()));
    
    match rpc_client.get_version() {
        Ok(v) => info!("✅ RPC 连接成功 (Version: {})", v.solana_core),
        Err(e) => {
            error!("❌ RPC 连接失败: {}", e);
            return Ok(());
        }
    }

    // 4. 加载钱包 (Keypair) 并检查余额
    let wallet_path = &config.strategy.wallet_path;
    let keypair = if Path::new(wallet_path).exists() {
        match read_keypair_from_file(wallet_path) {
            Ok(kp) => {
                info!("✅ 交易钱包已加载: {}", kp.pubkey());
                kp
            },
            Err(e) => {
                error!("❌ 无法读取交易钱包: {}", e);
                return Ok(());
            }
        }
    } else {
        error!("❌ 钱包文件不存在: {}", wallet_path);
        return Ok(());
    };

    // 检查余额
    match rpc_client.get_balance(&keypair.pubkey()) {
        Ok(balance) => {
            let sol_balance = balance as f64 / LAMPORTS_PER_SOL as f64;
            info!("💰 当前余额: {:.4} SOL", sol_balance);
            
            if sol_balance < 0.05 {
                warn!("⚠️  余额过低! 建议至少保留 0.05 SOL 用于 Gas 费。");
            }
        }
        Err(e) => error!("❌ 无法获取余额: {}", e),
    }
    
    // 鉴权钱包 (通常与交易钱包相同，或者是单独的)
    let auth_keypair = Arc::new(read_keypair_from_file(&config.jito.auth_keypair_path)?);

    // 5. 启动 Phase 2: 侦察系统 (Scout)
    info!("正在初始化侦察系统 (Phase 2)...");
    
    let mut scout = Scout::new(&config, &auth_keypair).await?;
    scout.start().await;
    
    Ok(())
}

/// 从文件读取 Keypair (JSON 格式)
fn read_keypair_from_file(path: &str) -> Result<Keypair, Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    let bytes: Vec<u8> = serde_json::from_reader(file)?;
    let keypair = Keypair::from_bytes(&bytes)?;
    Ok(keypair)
}
