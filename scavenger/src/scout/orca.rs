use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::str::FromStr;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;
use solana_transaction_status::{EncodedTransaction, UiMessage};
use crate::state::Inventory;
// use solana_client::rpc_config::{RpcProgramAccountsConfig, RpcAccountInfoConfig};
// use solana_client::rpc_filter::{RpcFilterType, Memcmp};
// use solana_sdk::commitment_config::CommitmentConfig;
// use solana_account_decoder::UiAccountEncoding;
use log::{info, error};

// Orca Whirlpool Program ID
pub const ORCA_WHIRLPOOL_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

#[derive(Debug)]
pub struct OrcaPoolEvent {
    pub signature: String,
    pub pool_id: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
}

pub fn parse_log_for_event(signature: &str, logs: &[String]) -> Option<OrcaPoolEvent> {
    // 简单的日志过滤
    // Orca 的 InitializePool 日志通常不如 Raydium 明显
    // 我们主要关注 "Program log: Instruction: InitializePool"
    
    let mut is_orca_init = false;
    for log in logs {
        if log.contains(ORCA_WHIRLPOOL_ID) && log.contains("InitializePool") {
            is_orca_init = true;
            break;
        }
    }
    
    if is_orca_init {
        Some(OrcaPoolEvent {
            signature: signature.to_string(),
            pool_id: Pubkey::default(), // 需进一步解析
            token_a: Pubkey::default(),
            token_b: Pubkey::default(),
        })
    } else {
        None
    }
}

// 异步获取并解析交易，提取 Pool 信息
pub async fn fetch_and_parse_tx(rpc_client: Arc<RpcClient>, signature: &str) -> Option<OrcaPoolEvent> {
    let sig = match Signature::from_str(signature) {
        Ok(s) => s,
        Err(_) => return None,
    };

    // 重试机制
    for _ in 0..3 {
        match rpc_client.get_transaction_with_config(&sig, solana_client::rpc_config::RpcTransactionConfig {
            encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
            commitment: Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        }).await {
            Ok(tx) => {
                if let Some(_transaction) = tx.transaction.transaction.decode() {
                    match tx.transaction.transaction {
                        EncodedTransaction::Json(ui_tx) => {
                            let message = ui_tx.message;
                            match message {
                                UiMessage::Raw(msg) => {
                                    let account_keys = msg.account_keys;
                                    for ix in msg.instructions {
                                        let program_id_index = ix.program_id_index as usize;
                                        if program_id_index < account_keys.len() {
                                            let program_id = &account_keys[program_id_index];
                                            if program_id == ORCA_WHIRLPOOL_ID {
                                                // InitializePool Instruction Accounts (Simplified):
                                                // 0. Configs
                                                // 1. TokenMintA
                                                // 2. TokenMintB
                                                // 3. Funder
                                                // 4. Whirlpool (Pool ID)
                                                
                                                if ix.accounts.len() >= 5 {
                                                    let token_a_idx = ix.accounts[1] as usize;
                                                    let token_b_idx = ix.accounts[2] as usize;
                                                    let pool_idx = ix.accounts[4] as usize;
                                                    
                                                    if pool_idx < account_keys.len() {
                                                        return Some(OrcaPoolEvent {
                                                            signature: signature.to_string(),
                                                            pool_id: Pubkey::from_str(&account_keys[pool_idx]).unwrap_or_default(),
                                                            token_a: Pubkey::from_str(&account_keys[token_a_idx]).unwrap_or_default(),
                                                            token_b: Pubkey::from_str(&account_keys[token_b_idx]).unwrap_or_default(),
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                }
                return None;
            },
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    }
    None
}

use reqwest;
use serde_json::Value;

/// Cold Start: 全量加载 Orca Whirlpool 账户到内存 (Via REST API)
pub async fn load_all_whirlpools(_rpc_client: Arc<RpcClient>, inventory: Arc<Inventory>) {
    info!("🔄 开始全量加载 Orca Whirlpool 账户 (Via Orca API)...");
    
    let url = "https://api.mainnet.orca.so/v1/whirlpool/list";
    
    match reqwest::get(url).await {
        Ok(resp) => {
            match resp.json::<Value>().await {
                Ok(json) => {
                    if let Some(whirlpools) = json.get("whirlpools").and_then(|v| v.as_array()) {
                        info!("✅ 成功获取 {} 个 Orca Whirlpool 信息", whirlpools.len());
                        let mut count = 0;
                        for pool in whirlpools {
                            let address_str = pool.get("address").and_then(|v| v.as_str());
                            let token_a_str = pool.get("tokenA").and_then(|v| v.get("mint")).and_then(|v| v.as_str());
                            let token_b_str = pool.get("tokenB").and_then(|v| v.get("mint")).and_then(|v| v.as_str());
                            
                            if let (Some(addr), Some(mint_a), Some(mint_b)) = (address_str, token_a_str, token_b_str) {
                                if let (Ok(pool_pk), Ok(token_a), Ok(token_b)) = (
                                    Pubkey::from_str(addr),
                                    Pubkey::from_str(mint_a),
                                    Pubkey::from_str(mint_b)
                                ) {
                                    inventory.add_pool(token_a, token_b, pool_pk);
                                    count += 1;
                                }
                            }
                        }
                        info!("📥 已索引 {} 个 Orca 池子到内存数据库", count);
                    } else {
                        error!("❌ Orca API 返回格式错误: 找不到 'whirlpools' 数组");
                    }
                },
                Err(e) => error!("❌ 解析 Orca API JSON 失败: {}", e),
            }
        },
        Err(e) => error!("❌ 请求 Orca API 失败: {}", e),
    }
}

