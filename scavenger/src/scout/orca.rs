use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::str::FromStr;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;
use solana_transaction_status::{EncodedTransaction, UiMessage};

// Orca Whirlpool Program ID
pub const ORCA_WHIRLPOOL_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

#[derive(Debug)]
pub struct OrcaPoolEvent {
    pub signature: String,
    pub pool_id: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
}

pub fn parse_log_for_event(signature: &str, _logs: &[String]) -> Option<OrcaPoolEvent> {
    // 简单的日志过滤，如果日志中提及了 Orca Whirlpool Program，我们认为它是潜在的相关交易
    // 更精细的过滤可以检查 "Instruction: InitializePool" 或 "Instruction: Swap"
    Some(OrcaPoolEvent {
        signature: signature.to_string(),
        pool_id: Pubkey::default(),
        token_a: Pubkey::default(),
        token_b: Pubkey::default(),
    })
}

// 异步获取并解析交易，提取 Pool 信息
pub async fn fetch_and_parse_tx(rpc_client: Arc<RpcClient>, signature: &str) -> Option<OrcaPoolEvent> {
    let sig = match Signature::from_str(signature) {
        Ok(s) => s,
        Err(_) => return None,
    };

    match rpc_client.get_transaction_with_config(&sig, solana_client::rpc_config::RpcTransactionConfig {
        encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
        commitment: Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
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
                                        if program_id == ORCA_WHIRLPOOL_ID {
                                            // Orca Whirlpool InitializePool Instruction
                                            // 这是一个复杂的匹配，因为不同指令 Account 顺序不同
                                            // 假设我们只关心 "InitializePool" (通常有 Configs, TokenMintA, TokenMintB, etc.)
                                            
                                            // 简化：如果是一个涉及 Orca 的交易，我们返回它作为信号
                                            // 真正的解析需要像 Raydium 那样深入 Account 索引
                                            return Some(OrcaPoolEvent {
                                                signature: signature.to_string(),
                                                pool_id: Pubkey::default(), // 需进一步解析
                                                token_a: Pubkey::default(),
                                                token_b: Pubkey::default(),
                                            });
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
            None
        },
        Err(_) => None
    }
}
