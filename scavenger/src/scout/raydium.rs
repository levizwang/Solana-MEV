use solana_sdk::pubkey::Pubkey;
// use solana_sdk::instruction::Instruction;
use std::str::FromStr;
// use base64::{Engine as _, engine::general_purpose};
use log::{info, warn, error};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use std::sync::Arc;
use solana_transaction_status::{EncodedTransaction, UiMessage, UiInstruction};

// Raydium AMM V4 Program ID
pub const RAYDIUM_AMM_V4_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

#[derive(Debug)]
pub struct NewPoolEvent {
    pub signature: String,
    pub pool_id: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub open_time: u64,
}

pub fn parse_log_for_new_pool(signature: &str, _logs: &[String]) -> Option<NewPoolEvent> {
    // 简单的日志解析策略
    Some(NewPoolEvent {
        signature: signature.to_string(),
        pool_id: Pubkey::default(),
        token_a: Pubkey::default(),
        token_b: Pubkey::default(),
        open_time: 0,
    })
}

// 辅助函数：解析 Transaction Data (Phase 3 核心)
pub async fn fetch_and_parse_tx(rpc_client: Arc<RpcClient>, signature: &str) -> Option<NewPoolEvent> {
    let sig = match Signature::from_str(signature) {
        Ok(s) => s,
        Err(_) => return None,
    };

    // 重试机制：尝试 5 次，每次间隔 500ms
    for i in 0..5 {
        // info!("🔄 尝试获取交易数据 ({}/5): {}", i + 1, signature);
        match rpc_client.get_transaction_with_config(&sig, solana_client::rpc_config::RpcTransactionConfig {
            encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
            commitment: Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()), // 回退到 confirmed 试试，或者 processed
            max_supported_transaction_version: Some(0),
        }).await {
            Ok(tx) => {
                if let Some(transaction) = tx.transaction.transaction.decode() {
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
                                             if program_id == RAYDIUM_AMM_V4_ID {
                                                 // info!("🔎 找到 Raydium 指令, Accounts: {}", ix.accounts.len());
                                                 if ix.accounts.len() >= 10 {
                                                     let pool_id_idx = ix.accounts[4] as usize;
                                                     let token_a_idx = ix.accounts[8] as usize;
                                                     let token_b_idx = ix.accounts[9] as usize;
                                                     
                                                     if pool_id_idx < account_keys.len() && token_a_idx < account_keys.len() && token_b_idx < account_keys.len() {
                                                         return Some(NewPoolEvent {
                                                             signature: signature.to_string(),
                                                             pool_id: Pubkey::from_str(&account_keys[pool_id_idx]).unwrap_or_default(),
                                                             token_a: Pubkey::from_str(&account_keys[token_a_idx]).unwrap_or_default(),
                                                             token_b: Pubkey::from_str(&account_keys[token_b_idx]).unwrap_or_default(),
                                                             open_time: 0,
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
                // 如果解析失败但获取成功，可能不是目标指令，但也无需重试
                return None;
            },
            Err(e) => {
                // 如果是 "Transaction not found"，等待并重试
                // info!("⏳ 交易尚未索引，等待重试... Error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    }
    
    // warn!("❌ 最终获取交易失败: {}", signature);
    None
}
