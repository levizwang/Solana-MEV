use reqwest::Client;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use log::{info, warn, error};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PoolInfo {
    pub address: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
}

pub async fn fetch_raydium_pools() -> Result<Vec<PoolInfo>, Box<dyn std::error::Error + Send + Sync>> {
    info!("🌐 Fetching Raydium pools...");
    let client = Client::new();
    let url = "https://api.raydium.io/v2/main/pairs";
    let cache_file = "raydium_pairs.json";
    
    // Try to fetch from API
    let json_result = client.get(url).send().await;
    
    let json: Value = match json_result {
        Ok(resp) => {
            match resp.json().await {
                Ok(v) => {
                    // Save to cache
                    if let Ok(mut file) = File::create(cache_file) {
                        if let Ok(content) = serde_json::to_string(&v) {
                            let _ = file.write_all(content.as_bytes());
                        }
                    }
                    v
                },
                Err(e) => {
                    warn!("⚠️ Failed to parse Raydium API response: {}", e);
                    load_from_cache(cache_file).await?
                }
            }
        },
        Err(e) => {
            warn!("⚠️ Failed to fetch Raydium pools from API: {}. Trying cache...", e);
            load_from_cache(cache_file).await?
        }
    };
    
    let mut pools = Vec::new();
    
    if let Some(pairs) = json.as_array() {
        for pair in pairs {
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

async fn load_from_cache(path: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    if Path::new(path).exists() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let json: Value = serde_json::from_str(&contents)?;
        info!("📂 Loaded Raydium pools from local cache: {}", path);
        Ok(json)
    } else {
        warn!("⚠️ Cache file not found. Using hardcoded fallback for SOL/USDC.");
        // Hardcoded fallback for SOL/USDC (Mainnet)
        let fallback_json = r#"[
            {
                "ammId": "58oQChx4yWmvKdwLLZzBi4ChoCcTKqdJennsXZGhPG43",
                "baseMint": "So11111111111111111111111111111111111111112",
                "quoteMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
            }
        ]"#;
        let json: Value = serde_json::from_str(fallback_json)?;
        Ok(json)
    }
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
