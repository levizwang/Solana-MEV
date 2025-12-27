use reqwest::Client;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
// use std::collections::HashMap;
// use log::{info, error};
use log::info;

#[derive(Debug, Clone)]
pub struct PoolInfo {
    pub address: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
}

pub async fn fetch_raydium_pools() -> Result<Vec<PoolInfo>, Box<dyn std::error::Error + Send + Sync>> {
    info!("🌐 Fetching Raydium pools...");
    let client = Client::new();
    // 使用 Raydium API 获取池子列表
    // 注意：Mainnet API 数据量较大，这里为了演示可能只处理部分或需要更高效的流式处理
    // 常用 Endpoint: https://api.raydium.io/v2/main/pairs (精简) 或 https://api.raydium.io/v2/sdk/liquidity/mainnet.json (全量)
    // 这里使用 pairs 接口作为示例
    let url = "https://api.raydium.io/v2/main/pairs";
    
    let resp = client.get(url).send().await?;
    let json: Value = resp.json().await?;
    
    let mut pools = Vec::new();
    
    if let Some(pairs) = json.as_array() {
        for pair in pairs {
            // 解析 ammId, baseMint, quoteMint
            let amm_id_str = pair.get("ammId").and_then(|v| v.as_str());
            let base_mint_str = pair.get("baseMint").and_then(|v| v.as_str());
            let quote_mint_str = pair.get("quoteMint").and_then(|v| v.as_str());
            
            if let (Some(addr), Some(mint_a), Some(mint_b)) = (amm_id_str, base_mint_str, quote_mint_str) {
                if let (Ok(address), Ok(token_a), Ok(token_b)) = (
                    Pubkey::from_str(addr),
                    Pubkey::from_str(mint_a),
                    Pubkey::from_str(mint_b)
                ) {
                    pools.push(PoolInfo {
                        address,
                        token_a,
                        token_b,
                    });
                }
            }
        }
    }
    
    info!("✅ Fetched {} Raydium pools", pools.len());
    Ok(pools)
}

pub async fn fetch_orca_pools() -> Result<Vec<PoolInfo>, Box<dyn std::error::Error + Send + Sync>> {
    info!("🌐 Fetching Orca pools...");
    let client = Client::new();
    let url = "https://api.mainnet.orca.so/v1/whirlpool/list";
    
    let resp = client.get(url).send().await?;
    let json: Value = resp.json().await?;
    
    let mut pools = Vec::new();
    
    if let Some(whirlpools) = json.get("whirlpools").and_then(|v| v.as_array()) {
        for pool in whirlpools {
            let address_str = pool.get("address").and_then(|v| v.as_str());
            let token_a_str = pool.get("tokenA").and_then(|v| v.get("mint")).and_then(|v| v.as_str());
            let token_b_str = pool.get("tokenB").and_then(|v| v.get("mint")).and_then(|v| v.as_str());
            
            if let (Some(addr), Some(mint_a), Some(mint_b)) = (address_str, token_a_str, token_b_str) {
                if let (Ok(address), Ok(token_a), Ok(token_b)) = (
                    Pubkey::from_str(addr),
                    Pubkey::from_str(mint_a),
                    Pubkey::from_str(mint_b)
                ) {
                    pools.push(PoolInfo {
                        address,
                        token_a,
                        token_b,
                    });
                }
            }
        }
    }
    
    info!("✅ Fetched {} Orca pools", pools.len());
    Ok(pools)
}
