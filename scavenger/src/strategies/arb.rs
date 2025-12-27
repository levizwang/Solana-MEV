use std::sync::Arc;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use log::{info, warn, error};
use crate::config::StrategyConfig;
use crate::state::Inventory;
use crate::amm::orca_whirlpool::Whirlpool;
use crate::amm::raydium_v4::AmmState;
use borsh::BorshDeserialize;
use crate::core::jito::JitoClient;
// use crate::core::swap::{build_orca_swap, swap as build_raydium_swap};
use std::str::FromStr;

// Constants
const ORCA_PROGRAM_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
const JITO_TIP_ACCOUNT: &str = "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5"; // Random Jito Tip Account

/// 处理账户更新 (主要针对 Orca)
pub async fn process_account_update(
    rpc_client: Arc<RpcClient>,
    keypair: Arc<Keypair>,
    pool_address: Pubkey,
    data: Vec<u8>,
    config: Arc<StrategyConfig>,
    inventory: Arc<Inventory>,
) {
    // 1. 识别这属于哪个共有对
    let pair = match inventory.find_pair_by_pool(&pool_address) {
        Some(p) => p,
        None => return, // 不在白名单中，忽略
    };

    // 2. 识别是哪个 DEX
    // 简单判断: 如果 pool_address == pair.orca_pool，则是 Orca
    let is_orca = Some(pool_address) == pair.orca_pool;
    
    if is_orca {
        // 解析 Orca 价格
        if let Some(price_info) = Whirlpool::decode_current_price(&data) {
            let orca_price = price_info.price;
            info!("🐬 [Orca Update] Pool: {} | Price: {:.6}", pool_address, orca_price);
            
            // 3. 获取对手盘 (Raydium) 价格
            let ray_pool_id = pair.raydium_pool;
            let ray_price = fetch_raydium_price(rpc_client.clone(), ray_pool_id).await;
            
            if let Some(ray_p) = ray_price {
                // 4. 计算价差
                check_spread_and_execute(rpc_client, keypair, orca_price, ray_p, "Orca", "Raydium", config).await;
            }
        }
    } else {
        // Raydium Account Update
        // 解析 Raydium 价格
        if let Ok(state) = AmmState::try_from_slice(&data) {
             let coin_decimals = state.coin_decimals;
             let pc_decimals = state.pc_decimals;
             let coin_amount = state.pool_total_deposit_coin;
             let pc_amount = state.pool_total_deposit_pc;
             
             if coin_amount > 0 && pc_amount > 0 {
                 let coin_scalar = 10f64.powi(coin_decimals as i32);
                 let pc_scalar = 10f64.powi(pc_decimals as i32);
                 let ray_price = (pc_amount as f64 / pc_scalar) / (coin_amount as f64 / coin_scalar);
                 
                 info!("🦄 [Raydium Update] Pool: {} | Price: {:.6}", pool_address, ray_price);
                 
                 // 3. 获取对手盘 (Orca) 价格
                 // 尝试 RPC 获取 (Orca 变动少，RPC 获取比较安全)
                 if let Some(orca_pool_id) = pair.orca_pool {
                     let orca_price = fetch_orca_price(rpc_client.clone(), orca_pool_id).await;
                     
                     if let Some(orca_p) = orca_price {
                         // 4. 计算价差
                         check_spread_and_execute(rpc_client, keypair, ray_price, orca_p, "Raydium", "Orca", config).await;
                     }
                 }
             }
        }
    }
}

/// 检查价差并执行
async fn check_spread_and_execute(
    rpc_client: Arc<RpcClient>,
    keypair: Arc<Keypair>,
    price_a: f64,
    price_b: f64,
    label_a: &str,
    label_b: &str,
    config: Arc<StrategyConfig>,
) {
    let spread = (price_a - price_b).abs() / price_a.min(price_b);
    let spread_pct = spread * 100.0;
    
    if spread_pct > 0.5 { // 0.5% 阈值
        info!("🚨 [ARBITRAGE] Opportunity! {} (${:.6}) vs {} (${:.6}) | Spread: {:.2}%", 
            label_a, price_a, label_b, price_b, spread_pct);
        
        let jito_client = JitoClient::new();
        
        // 构建并发送 Bundle
        // 假设我们有一个固定的交易路径: Buy Low -> Sell High
        // 由于这只是一个框架，我们目前只构建一个 Jito Tip 交易来验证流程
        // 真实的 Swap 需要从 Inventory 获取 Token Mint、Vault 等详细 Account Meta
        // 这需要 fetch_and_parse 完整的池子账户信息，或者在 Inventory 中缓存更详细的 PoolInfo
        
        // 1. Tip Instruction
        let tip_account = Pubkey::from_str(JITO_TIP_ACCOUNT).unwrap();
        let tip_lamports = (config.static_tip_sol * 1_000_000_000.0) as u64;
        let tip_instruction = solana_sdk::system_instruction::transfer(
            &keypair.pubkey(),
            &tip_account,
            tip_lamports,
        );
        
        // 2. Build Transaction
        let recent_blockhash = match rpc_client.get_latest_blockhash().await {
            Ok(hash) => hash,
            Err(e) => {
                error!("❌ Failed to get blockhash: {}", e);
                return;
            }
        };
        
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[tip_instruction], // 真实场景这里需要加上 swap_ix_1, swap_ix_2
            Some(&keypair.pubkey()),
            &[&*keypair],
            recent_blockhash,
        );
        
        // 3. Serialize and Send
        let tx_base58 = bs58::encode(bincode::serialize(&tx).unwrap()).into_string();
        
        info!("📦 Sending Bundle to Jito (Simulated Swap)...");
        match jito_client.send_bundle(vec![tx_base58], None).await {
            Ok(bundle_id) => info!("✅ Bundle Sent! ID: {}", bundle_id),
            Err(e) => error!("❌ Bundle Send Failed: {}", e),
        }

    } else {
        // info!("💤 Spread: {:.2}% (No Action)", spread_pct);
    }
}

/// 获取 Raydium 价格 (真实逻辑)
/// 通过 RPC 获取 Pool Account Data，解析 State，计算 Price
async fn fetch_raydium_price(rpc_client: Arc<RpcClient>, pool_id: Pubkey) -> Option<f64> {
    match rpc_client.get_account_data(&pool_id).await {
        Ok(data) => {
            // 1. 反序列化 AmmState
            if let Ok(state) = AmmState::try_from_slice(&data) {
                // 2. 获取精度
                let coin_decimals = state.coin_decimals;
                let pc_decimals = state.pc_decimals;
                
                // 3. 获取储备量 (Reserves)
                let coin_amount = state.pool_total_deposit_coin;
                let pc_amount = state.pool_total_deposit_pc;
                
                if coin_amount == 0 || pc_amount == 0 {
                    return None;
                }

                // 4. 计算价格
                let coin_scalar = 10f64.powi(coin_decimals as i32);
                let pc_scalar = 10f64.powi(pc_decimals as i32);
                
                let price = (pc_amount as f64 / pc_scalar) / (coin_amount as f64 / coin_scalar);
                
                return Some(price);
            } else {
                warn!("❌ Failed to deserialize Raydium AMM State for {}", pool_id);
            }
        },
        Err(e) => {
            error!("❌ Failed to fetch Raydium Pool Account {}: {}", pool_id, e);
        }
    }
    None
}

/// 获取 Orca 价格 (真实逻辑)
async fn fetch_orca_price(rpc_client: Arc<RpcClient>, pool_id: Pubkey) -> Option<f64> {
    match rpc_client.get_account_data(&pool_id).await {
        Ok(data) => {
            if let Some(price_info) = Whirlpool::decode_current_price(&data) {
                return Some(price_info.price);
            }
        },
        Err(e) => error!("❌ Failed to fetch Orca Pool Account {}: {}", pool_id, e),
    }
    None
}
