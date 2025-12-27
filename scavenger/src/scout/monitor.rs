use log::info;
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

use crate::config::StrategyConfig;

pub async fn start_monitoring(
    ws_url: String, 
    rpc_client: Arc<RpcClient>,
    keypair: Arc<Keypair>,
    config: Arc<StrategyConfig>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("🔌 连接 WebSocket: {}", ws_url);
    
    // 我们需要建立两个独立的 PubsubClient，或者在一个 Client 上建立两个 Subscription
    // solana-client 的 PubsubClient 支持多个 subscription
    
    let pubsub_client = solana_client::nonblocking::pubsub_client::PubsubClient::new(&ws_url).await?;
    info!("✅ WebSocket 连接成功");

    // 1. 订阅 Raydium 日志
    let (raydium_stream, _unsub_ray) = pubsub_client.logs_subscribe(
        RpcTransactionLogsFilter::Mentions(vec![RAYDIUM_AMM_V4.to_string()]),
        RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig::processed()),
        },
    ).await?;
    info!("👀 已订阅 Raydium AMM V4 日志");

    // 2. 订阅 Orca 日志
    let (orca_stream, _unsub_orca) = pubsub_client.logs_subscribe(
        RpcTransactionLogsFilter::Mentions(vec![ORCA_WHIRLPOOL.to_string()]),
        RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig::processed()),
        },
    ).await?;
    info!("👀 已订阅 Orca Whirlpool 日志");

    info!("🚀 多路监控系统已启动，等待信号...");

    // 使用 tokio::select! 或者合并流来同时处理
    // 这里简单起见，我们 Spawn 两个独立的循环，或者用 select
    
    // 为了在一个函数里跑，我们可以用 futures::stream::select
    let mut combined_stream = futures::stream::select(
        raydium_stream.map(|log| (log, "Raydium")),
        orca_stream.map(|log| (log, "Orca"))
    );

    while let Some((response, source)) = combined_stream.next().await {
        let logs_response: RpcLogsResponse = response.value;
        let logs = &logs_response.logs;
        let signature = &logs_response.signature;

        if source == "Raydium" {
            if let Some(event) = raydium::parse_log_for_new_pool(signature, logs) {
                info!("✨ [Raydium] 发现潜在活动! Tx: https://solscan.io/tx/{}", event.signature);
                
                let client = rpc_client.clone();
                let kp = keypair.clone();
                let cfg = config.clone();
                let sig = event.signature.clone();
                
                tokio::spawn(async move {
                    if let Some(full_event) = raydium::fetch_and_parse_tx(client.clone(), &sig).await {
                        info!("🎉 [Raydium] 成功解析池子详情: Pool: {}, TokenA: {}, TokenB: {}", 
                            full_event.pool_id, full_event.token_a, full_event.token_b);
                        
                        engine::process_new_pool(client, kp, full_event, cfg).await;
                    }
                });
            }
        } else if source == "Orca" {
            if let Some(event) = orca::parse_log_for_event(signature, logs) {
                // info!("🌊 [Orca] 发现潜在活动! Tx: https://solscan.io/tx/{}", event.signature);
                
                let client = rpc_client.clone();
                let kp = keypair.clone();
                let cfg = config.clone();
                let sig = event.signature.clone();

                tokio::spawn(async move {
                    if let Some(full_event) = orca::fetch_and_parse_tx(client.clone(), &sig).await {
                        info!("🌊 [Orca] 成功解析池子详情: Pool: {}, TokenA: {}, TokenB: {}", 
                            full_event.pool_id, full_event.token_a, full_event.token_b);
                        
                        // 触发策略引擎处理 Orca 事件
                        engine::process_orca_event(client, kp, full_event, cfg).await;
                    }
                });
            }
        }
    }

    Ok(())
}
