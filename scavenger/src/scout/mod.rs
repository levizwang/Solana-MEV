use std::sync::Arc;
// use tokio::sync::mpsc;
use log::{info, error};
use solana_sdk::signature::Keypair;
// use jito_searcher_client::get_searcher_client_no_auth; 
// use jito_protos::searcher::searcher_service_client::SearcherServiceClient;
// use tonic::transport::Channel;
use crate::config::AppConfig;
// use tonic::transport::Endpoint;
use solana_client::nonblocking::rpc_client::RpcClient;

mod monitor; // 引入监控模块
pub mod raydium; // 引入 Raydium 解析模块
pub mod orca; // 引入 Orca 解析模块
pub mod api; // 引入 API 模块

// use crate::strategy::engine; // 引入策略引擎 (removed unused import)

use crate::config::StrategyConfig;
use crate::state::Inventory;

pub struct Scout {
    // client: SearcherServiceClient<Channel>,
    rpc_client: Arc<RpcClient>, // 添加 RPC Client
    ws_url: String, 
    keypair: Arc<Keypair>, // 保存 Keypair 用于传给 Strategy
    strategy_config: StrategyConfig, // 保存策略配置
    inventory: Arc<Inventory>, // 全网代币索引
    strategy_name: String, // 策略名称
}

impl Scout {
    pub async fn new(config: &AppConfig, auth_keypair: &Arc<Keypair>, inventory: Arc<Inventory>, strategy_name: String) -> Result<Self, Box<dyn std::error::Error>> {
        // info!("🔍 连接 Jito Block Engine: {}", config.jito.block_engine_url);
        
        // let endpoint = Endpoint::from_shared(config.jito.block_engine_url.clone())?;
        // let channel = endpoint.connect().await?;
        // let client = SearcherServiceClient::new(channel);

        // info!("✅ Jito Searcher Client 连接成功 (No Auth Mode)");
        info!("🚧 Jito Client 暂时禁用 (SDK Version Mismatch)，仅使用 RPC");
        
        // 初始化 RPC Client (Non-blocking)
        let rpc_client = Arc::new(RpcClient::new(config.network.rpc_url.clone()));
        
        // Clone strategy config
        let strategy_config = StrategyConfig {
            wallet_path: config.strategy.wallet_path.clone(),
            trade_amount_sol: config.strategy.trade_amount_sol,
            static_tip_sol: config.strategy.static_tip_sol,
            dynamic_tip_ratio: config.strategy.dynamic_tip_ratio,
            max_tip_sol: config.strategy.max_tip_sol,
        };
        
        Ok(Self { 
            // client,
            rpc_client,
            ws_url: config.network.ws_url.clone(),
            keypair: auth_keypair.clone(),
            strategy_config,
            inventory,
            strategy_name,
        })
    }

    pub async fn start(&mut self) {
        info!("👀 侦察兵已就位，开始监听全网新池子... [Mode: {}]", self.strategy_name);
        
        // 启动 WebSocket 监听器 (在后台任务中运行)
        let ws_url = self.ws_url.clone();
        let rpc_client = self.rpc_client.clone();
        let keypair = self.keypair.clone(); // Clone for task
        let strategy_config = Arc::new(self.strategy_config.clone()); // Wrap in Arc
        let inventory = self.inventory.clone();
        let strategy_name = self.strategy_name.clone();

        tokio::spawn(async move {
            // 我们需要修改 monitor 以接受 callback 或者 channel
            // 这里为了简单，我们直接在 monitor 内部调用 strategy
            // 但更好的做法是 monitor 只负责产出数据，通过 channel 发送给 engine
            // 暂时保持 monitor 独立，我们在 monitor 内部集成 engine 调用
            
            // 实际上 monitor::start_monitoring 现在只打印日志
            // 我们需要修改它来调用 engine::process_new_pool
            if let Err(e) = monitor::start_monitoring(ws_url, rpc_client, keypair, strategy_config, inventory, strategy_name).await {
                error!("❌ WebSocket 监听器异常退出: {}", e);
            }
        });
        
        // Jito 相关逻辑 (如果有)
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            // info!("... Scout Heartbeat ...");
        }
    }
}
