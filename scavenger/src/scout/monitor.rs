use log::{info, error};
// use solana_client::pubsub_client::PubsubClient;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_client::rpc_response::RpcLogsResponse;
// use solana_client::rpc_response::Response;
use solana_sdk::commitment_config::CommitmentConfig;
use futures::StreamExt;
use crate::scout::raydium;
use crate::scout::orca;
use crate::strategy::engine; // 引入引擎
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

// Raydium AMM V4 Program ID
pub const RAYDIUM_AMM_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
// Orca Whirlpool Program ID
pub const ORCA_WHIRLPOOL: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

pub async fn start_monitoring(
    ws_url: String, 
    rpc_client: Arc<RpcClient>,
    keypair: Arc<Keypair>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("🔌 连接 WebSocket: {}", ws_url);
    
    // v1.14 的 nonblocking client
    let pubsub_client = solana_client::nonblocking::pubsub_client::PubsubClient::new(&ws_url).await?;
    
    info!("✅ WebSocket 连接成功，开始多路订阅 (Raydium & Orca)...");

    // 订阅 Raydium 和 Orca 的日志
    // 由于 logs_subscribe 一次只能传一个 Filter，如果需要监听多个 Program，
    // 要么开多个 Subscription，要么监听所有并过滤。
    // RpcTransactionLogsFilter::Mentions 接受 Vec<String>，所以可以一次订阅多个 Program!
    
    let (mut stream, _unsubscribe) = pubsub_client.logs_subscribe(
        RpcTransactionLogsFilter::Mentions(vec![
            RAYDIUM_AMM_V4.to_string(),
            ORCA_WHIRLPOOL.to_string()
        ]),
        RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig::processed()),
        },
    ).await?;

    info!("👀 正在监听 Raydium V4 和 Orca Whirlpool 日志...");

    // 处理日志流
    while let Some(response) = stream.next().await {
        // response 是 Response<RpcLogsResponse>
        let logs_response: RpcLogsResponse = response.value;
        let logs = &logs_response.logs;
        let signature = &logs_response.signature;
        
        // 1. 检查 Raydium
        if let Some(event) = raydium::parse_log_for_new_pool(signature, logs) {
            info!("✨ [Raydium] 发现潜在活动! Tx: https://solscan.io/tx/{}", event.signature);
            
            let client = rpc_client.clone();
            let kp = keypair.clone();
            let sig = event.signature.clone();
            
            tokio::spawn(async move {
                if let Some(full_event) = raydium::fetch_and_parse_tx(client.clone(), &sig).await {
                    info!("🎉 [Raydium] 成功解析池子详情: Pool: {}, TokenA: {}, TokenB: {}", 
                        full_event.pool_id, full_event.token_a, full_event.token_b);
                    
                    engine::process_new_pool(client, kp, full_event).await;
                }
            });
        }

        // 2. 检查 Orca
        if let Some(event) = orca::parse_log_for_event(signature, logs) {
            // Orca 的日志可能非常多，这里可能需要更严格的过滤
            // 暂时只打印 Log
            // info!("🌊 [Orca] 发现潜在活动! Tx: https://solscan.io/tx/{}", event.signature);
            
            // 可以在这里加异步 fetch 逻辑，类似于 Raydium
        }
    }

    Ok(())
}
