use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::str::FromStr;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;
use solana_transaction_status::{EncodedTransaction, UiMessage};
use crate::state::Inventory;
use solana_client::rpc_config::{RpcProgramAccountsConfig, RpcAccountInfoConfig};
use solana_client::rpc_filter::{RpcFilterType, Memcmp};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_account_decoder::UiAccountEncoding;
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

/// Cold Start: 全量加载 Orca Whirlpool 账户到内存
pub async fn load_all_whirlpools(rpc_client: Arc<RpcClient>, inventory: Arc<Inventory>) {
    info!("🔄 开始全量加载 Orca Whirlpool 账户 (Cold Start)...");
    
    let program_id = Pubkey::from_str(ORCA_WHIRLPOOL_ID).unwrap();
    
    // Whirlpool Discriminator: 62, 10, 14, 196, 56, 60, 89, 21
    // derived from sha256("account:Whirlpool")[..8]
    let discriminator: Vec<u8> = vec![62, 10, 14, 196, 56, 60, 89, 21];
    let _discriminator_base58 = bs58::encode(&discriminator).into_string();

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![
            RpcFilterType::Memcmp(Memcmp::new_base58_encoded(0, &discriminator.clone())),
        ]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            commitment: Some(CommitmentConfig::processed()),
            ..RpcAccountInfoConfig::default()
        },
        with_context: Some(false),
    };

    match rpc_client.get_program_accounts_with_config(&program_id, config).await {
        Ok(accounts) => {
            info!("✅ 成功获取 {} 个 Orca Whirlpool 账户", accounts.len());
            let mut count = 0;
            for (pubkey, account) in accounts {
                let data = account.data;
                // Offset 101 for Token A, 181 for Token B
                // 确保数据长度足够
                if data.len() >= 213 { 
                     let token_a_bytes = &data[101..133];
                     let token_b_bytes = &data[181..213];
                     
                     if let (Ok(token_a), Ok(token_b)) = (
                         Pubkey::try_from(token_a_bytes), 
                         Pubkey::try_from(token_b_bytes)
                     ) {
                         inventory.add_pool(token_a, token_b, pubkey);
                         count += 1;
                     }
                }
            }
            info!("📥 已索引 {} 个 Orca 池子到内存数据库", count);
        },
        Err(e) => {
            error!("❌ 加载 Orca 池子失败: {}", e);
        }
    }
}
