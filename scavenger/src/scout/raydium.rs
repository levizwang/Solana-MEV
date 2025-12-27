use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::str::FromStr;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;
use solana_transaction_status::{EncodedTransaction, UiMessage};

#[derive(Debug)]
pub struct NewPoolEvent {
    pub signature: String,
    pub pool_id: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub open_time: u64,
}

pub fn parse_log_for_new_pool(signature: &str, logs: &[String]) -> Option<NewPoolEvent> {
    // 简单的日志过滤
    let mut is_raydium_init = false;
    for log in logs {
        if log.contains("Initialize2") || log.contains("Initialize") {
            is_raydium_init = true;
            break;
        }
    }
    
    if is_raydium_init {
        Some(NewPoolEvent {
            signature: signature.to_string(),
            pool_id: Pubkey::default(), // 需进一步解析
            token_a: Pubkey::default(),
            token_b: Pubkey::default(),
            open_time: 0,
        })
    } else {
        None
    }
}

// 异步获取并解析交易，提取 Pool 信息
pub async fn fetch_and_parse_tx(rpc_client: Arc<RpcClient>, signature: &str) -> Option<NewPoolEvent> {
    let sig = match Signature::from_str(signature) {
        Ok(s) => s,
        Err(_) => return None,
    };

    // 重试机制
    for _i in 0..5 {
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
                                    // 简化逻辑：通常 Initialize2 指令的 Accounts 中包含了 Token A, Token B, Pool ID
                                    // 具体的 Index 取决于 Raydium Program 的定义
                                    // 这里假设我们能从 Accounts 中找到特定的模式
                                    
                                    // 这是一个 Hacky 的实现，生产环境应该解析 Instruction Data
                                    if account_keys.len() > 10 {
                                        // 假设 Pool Id 是第 2 个 (Index 1) - 仅作示例
                                        // 实际上需要根据 Instruction Discriminator 来精确定位
                                        
                                        // 为了演示 Phase 3，我们返回一个 Mock 的解析结果
                                        // 只要是 Initialize2，我们就认为它是新池子
                                        return Some(NewPoolEvent {
                                            signature: signature.to_string(),
                                            pool_id: Pubkey::from_str(&account_keys[1]).unwrap_or_default(),
                                            token_a: Pubkey::from_str(&account_keys[8]).unwrap_or_default(), // Mock Index
                                            token_b: Pubkey::from_str(&account_keys[9]).unwrap_or_default(), // Mock Index
                                            open_time: 0,
                                        });
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
            Err(_e) => {
                // error!("❌ 获取交易失败: {} (Attempt {})", e, i);
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    }
    None
}
