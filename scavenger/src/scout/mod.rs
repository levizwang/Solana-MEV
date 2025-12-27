use std::sync::Arc;
// use tokio::sync::mpsc;
use log::{info, error, warn};
use solana_sdk::signature::Keypair;
// use jito_searcher_client::get_searcher_client_no_auth; 
use jito_protos::searcher::searcher_service_client::SearcherServiceClient;
use tonic::transport::Channel;
use crate::config::AppConfig;
use tonic::transport::Endpoint;
use solana_client::nonblocking::rpc_client::RpcClient;

mod monitor; // 引入监控模块
pub mod raydium; // 引入解析模块 (需要 pub 供 strategy 使用)

use crate::strategy::engine; // 引入策略引擎

pub struct Scout {
    client: SearcherServiceClient<Channel>,
    rpc_client: Arc<RpcClient>, // 添加 RPC Client
    ws_url: String, 
    keypair: Arc<Keypair>, // 保存 Keypair 用于传给 Strategy
}

impl Scout {
    pub async fn new(config: &AppConfig, auth_keypair: &Arc<Keypair>) -> Result<Self, Box<dyn std::error::Error>> {
        info!("🔍 连接 Jito Block Engine: {}", config.jito.block_engine_url);
        
        let endpoint = Endpoint::from_shared(config.jito.block_engine_url.clone())?;
        let channel = endpoint.connect().await?;
        let client = SearcherServiceClient::new(channel);

        info!("✅ Jito Searcher Client 连接成功 (No Auth Mode)");
        
        // 初始化 RPC Client (Non-blocking)
        let rpc_client = Arc::new(RpcClient::new(config.network.rpc_url.clone()));
        
        Ok(Self { 
            client,
            rpc_client,
            ws_url: config.network.ws_url.clone(),
            keypair: auth_keypair.clone(),
        })
    }

    pub async fn start(&mut self) {
        info!("👀 侦察兵已就位，开始监听全网新池子...");
        
        // 启动 WebSocket 监听器 (在后台任务中运行)
        let ws_url = self.ws_url.clone();
        let rpc_client = self.rpc_client.clone();
        let keypair = self.keypair.clone(); // Clone for task
        
        tokio::spawn(async move {
            // 我们需要修改 monitor 以接受 callback 或者 channel
            // 这里为了简单，我们直接在 monitor 内部调用 strategy
            // 但更好的做法是 monitor 只负责产出数据，通过 channel 发送给 engine
            // 暂时保持 monitor 独立，我们在 monitor 内部集成 engine 调用
            
            // 实际上 monitor::start_monitoring 现在只打印日志
            // 我们需要修改它来调用 engine::process_new_pool
            if let Err(e) = monitor::start_monitoring(ws_url, rpc_client, keypair).await {
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
