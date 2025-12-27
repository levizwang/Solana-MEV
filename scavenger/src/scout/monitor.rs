use log::{info, error};
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_client::rpc_response::RpcLogsResponse;
use solana_sdk::commitment_config::CommitmentConfig;
use futures::StreamExt;
use crate::scout::raydium;
use crate::scout::orca;
use crate::strategies::arb; // 引入 Arb 策略
use crate::strategies::sniper; // 引入 Sniper 策略
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
// use solana_sdk::pubkey::Pubkey;

// Raydium AMM V4 Program ID
pub const RAYDIUM_AMM_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
// Orca Whirlpool Program ID
pub const ORCA_WHIRLPOOL: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

use crate::config::StrategyConfig;
use crate::state::Inventory;
use crate::amm::orca_whirlpool::Whirlpool;

pub async fn start_monitoring(
    ws_url: String, 
    rpc_client: Arc<RpcClient>,
    keypair: Arc<Keypair>,
    config: Arc<StrategyConfig>,
    inventory: Arc<Inventory>,
    strategy_name: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("🔌 连接 WebSocket: {}", ws_url);
    
    // 我们需要建立两个独立的 PubsubClient，或者在一个 Client 上建立两个 Subscription
    // solana-client 的 PubsubClient 支持多个 subscription
    
    let pubsub_client = solana_client::nonblocking::pubsub_client::PubsubClient::new(&ws_url).await?;
    info!("✅ WebSocket 连接成功");

    if strategy_name == "arb" {
        // === Arb 策略监听逻辑 ===
        info!("🔄 [Arb] 正在从 API 加载白名单 (Common Pairs)...");
        if let Err(e) = inventory.load_from_api().await {
            error!("❌ Failed to load API data: {}", e);
        }
        let watch_list = inventory.get_watch_list();
        let total = watch_list.len();
        
        // 限制订阅数量，防止 RPC 过载 (标准 RPC 限制通常较严)
        // 如果有 Geyser，这里可以订阅全部
        let max_subs = 50.min(total);
        let target_accounts = &watch_list[0..max_subs];
        
        info!("👀 [Arb] 监控名单共 {} 个，当前订阅 Top {} 个账户进行实时监听...", total, max_subs);
        
        // 创建一个多路复用流
        let mut streams = Vec::new();
        
        for pubkey in target_accounts {
            let (stream, _unsub) = pubsub_client.account_subscribe(
                pubkey,
                None
            ).await?;
            // 将流映射为带 Pubkey 的事件，方便识别来源
            streams.push(stream.map(move |data| (*pubkey, data)));
        }
        
        // 合并所有流
        let mut combined_stream = futures::stream::select_all(streams);
        
        info!("🚀 [Arb] 监听已启动，等待价格变动...");
        
        while let Some((pool_address, account_data)) = combined_stream.next().await {
            // 这里收到的 account_data 是 UiAccount 或 EncodedAccount
            // 我们需要解析它
            // info!("🔔 [Update] Pool: {}", pool_address);
            
            // 将处理逻辑抛给 arb 策略模块
            let client = rpc_client.clone();
            let kp = keypair.clone();
            let cfg = config.clone();
            let inventory_clone = inventory.clone();
            let data_vec = account_data.value.data.decode().unwrap_or_default();
            
            tokio::spawn(async move {
                arb::process_account_update(
                    client,
                    kp,
                    pool_address,
                    data_vec,
                    cfg,
                    inventory_clone
                ).await;
            });
        }

    } else {
        // === Sniper 策略监听逻辑 (默认) ===
        info!("🎯 [Sniper] 启动新池狙击模式 (Logs Subscribe)...");

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

        let mut raydium_log_count = 0;

        while let Some((response, source)) = combined_stream.next().await {
            let logs_response: RpcLogsResponse = response.value;
            let logs = &logs_response.logs;
            let signature = &logs_response.signature;

            if source == "Raydium" {
                raydium_log_count += 1;
                
                if let Some(event) = raydium::parse_log_for_new_pool(signature, logs) {
                    // 仅周期性打印日志，减少刷屏
                    if raydium_log_count % 50 == 0 {
                         info!("✨ [Raydium] 监测中... 已扫描 {} 条相关日志. 最新潜在活动 Tx: https://solscan.io/tx/{}", raydium_log_count, event.signature);
                    } else {
                         // 使用 debug 级别记录详细日志
                         log::debug!("✨ [Raydium] 发现潜在活动! Tx: https://solscan.io/tx/{}", event.signature);
                    }
                    
                    let client = rpc_client.clone();
                    let kp = keypair.clone();
                    let cfg = config.clone();
                    let sig = event.signature.clone();
                    let inventory_clone = inventory.clone();
                    
                    tokio::spawn(async move {
                        if let Some(full_event) = raydium::fetch_and_parse_tx(client.clone(), &sig).await {
                            info!("🎉 [Raydium] 成功解析池子详情: Pool: {}, TokenA: {}, TokenB: {}", 
                                full_event.pool_id, full_event.token_a, full_event.token_b);
                            
                            // Sniper Logic
                            sniper::execute(client, kp, cfg, inventory_clone).await;
                        }
                    });
                }
            } else if source == "Orca" {
                // Orca 日志全量打印
                if let Some(event) = orca::parse_log_for_event(signature, logs) {
                    info!("🌊 [Orca] 发现潜在活动! Tx: https://solscan.io/tx/{}", event.signature);
                    
                    let client = rpc_client.clone();
                    let _kp = keypair.clone();
                    let _cfg = config.clone();
                    let sig = event.signature.clone();
                    let inventory_clone = inventory.clone();

                    tokio::spawn(async move {
                        if let Some(full_event) = orca::fetch_and_parse_tx(client.clone(), &sig).await {
                            info!("🌊 [Orca] 成功解析池子详情: Pool: {}, TokenA: {}, TokenB: {}", 
                                full_event.pool_id, full_event.token_a, full_event.token_b);
                            
                            // 实时更新 Inventory
                            inventory_clone.add_pool(full_event.token_a, full_event.token_b, full_event.pool_id);

                            // 尝试获取池子当前价格
                            match client.get_account_data(&full_event.pool_id).await {
                                Ok(data) => {
                                    if let Some(price_info) = Whirlpool::decode_current_price(&data) {
                                         info!("💲 [Orca Pricing] Pool: {} | Price: {:.6} | Tick: {} | Liquidity: {}", 
                                            full_event.pool_id, price_info.price, price_info.tick, price_info.liquidity);
                                    }
                                },
                                Err(e) => {
                                    info!("⚠️ [Orca Pricing] 获取账户数据失败: {}", e);
                                }
                            }
                        }
                    });
                }
            }
        }
    }

    Ok(())
}
