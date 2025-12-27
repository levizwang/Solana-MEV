use log::{info, error};
// use solana_client::pubsub_client::PubsubClient;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_client::rpc_response::RpcLogsResponse;
// use solana_client::rpc_response::Response;
use solana_sdk::commitment_config::CommitmentConfig;
use futures::StreamExt;
use crate::scout::raydium;
use crate::strategy::engine; // 引入引擎
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

// Raydium AMM V4 Program ID
pub const RAYDIUM_AMM_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

pub async fn start_monitoring(
    ws_url: String, 
    rpc_client: Arc<RpcClient>,
    keypair: Arc<Keypair>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("🔌 连接 WebSocket: {}", ws_url);
    
    // v1.14 的 nonblocking client
    let pubsub_client = solana_client::nonblocking::pubsub_client::PubsubClient::new(&ws_url).await?;
    
    info!("✅ WebSocket 连接成功，开始订阅日志...");

    // 订阅 Raydium AMM Program 的日志
    let (mut stream, _unsubscribe) = pubsub_client.logs_subscribe(
        RpcTransactionLogsFilter::Mentions(vec![RAYDIUM_AMM_V4.to_string()]),
        RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig::processed()),
        },
    ).await?;

    info!("👀 正在监听 Raydium AMM v4 日志...");

    // 处理日志流
    while let Some(response) = stream.next().await {
        // response 是 Response<RpcLogsResponse>
        let logs_response: RpcLogsResponse = response.value;
        let logs = &logs_response.logs;
        let signature = &logs_response.signature;
        
        // 使用 raydium 模块解析日志
        if let Some(event) = raydium::parse_log_for_new_pool(signature, logs) {
            info!("✨ 发现潜在新池子! Tx: https://solscan.io/tx/{}", event.signature);
            
            // 异步获取完整交易数据 (Spawn Task 以避免阻塞 WebSocket 流)
            let client = rpc_client.clone();
            let kp = keypair.clone();
            let sig = event.signature.clone();
            
            tokio::spawn(async move {
                if let Some(full_event) = raydium::fetch_and_parse_tx(client.clone(), &sig).await {
                    info!("🎉 成功解析池子详情: Pool: {}, TokenA: {}, TokenB: {}", 
                        full_event.pool_id, full_event.token_a, full_event.token_b);
                    
                    // 触发策略引擎
                    engine::process_new_pool(client, kp, full_event).await;
                }
            });
        }
    }

    Ok(())
}
