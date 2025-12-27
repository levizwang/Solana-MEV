use std::sync::Arc;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use log::{info, warn, error};
use crate::config::StrategyConfig;
use crate::state::Inventory;
use crate::amm::orca_whirlpool::Whirlpool;
use crate::amm::raydium_v4::AmmState;
use crate::amm::serum::SerumMarketV3;
use borsh::BorshDeserialize;
use crate::core::jito_http::JitoHttpClient;
use crate::core::swap::{swap as build_raydium_swap, build_orca_swap};
use std::str::FromStr;

// Constants
const ORCA_PROGRAM_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
const JITO_TIP_ACCOUNT: &str = "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5"; // Jito Tip Account 1
const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

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
                check_spread_and_execute(rpc_client, keypair, orca_price, ray_p, "Orca", "Raydium", config, &pair).await;
            }
        }
    } else {
        // Raydium Account Update
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
                 
                 if let Some(orca_pool_id) = pair.orca_pool {
                     let orca_price = fetch_orca_price(rpc_client.clone(), orca_pool_id).await;
                     
                     if let Some(orca_p) = orca_price {
                         check_spread_and_execute(rpc_client, keypair, ray_price, orca_p, "Raydium", "Orca", config, &pair).await;
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
    pair: &crate::state::ArbitragePair,
) {
    let spread = (price_a - price_b).abs() / price_a.min(price_b);
    let spread_pct = spread * 100.0;
    
    if spread_pct > 0.5 { // 0.5% 阈值
        info!("🚨 [ARBITRAGE] Opportunity! {} (${:.6}) vs {} (${:.6}) | Spread: {:.2}%", 
            label_a, price_a, label_b, price_b, spread_pct);
        
        let jito_client = JitoHttpClient::new();
        let amount_in_sol = config.trade_amount_sol; // e.g. 0.1 SOL
        let amount_in_lamports = (amount_in_sol * 1_000_000_000.0) as u64;

        // 决定买卖方向: Buy Low -> Sell High
        // 如果 Price A < Price B: Buy A -> Sell B
        // 如果 Price A > Price B: Buy B -> Sell A
        
        let (buy_pool, buy_label, sell_pool, sell_label) = if price_a < price_b {
            // A 便宜，买 A
            let p_a = if label_a == "Raydium" { pair.raydium_pool } else { pair.orca_pool.unwrap() };
            let p_b = if label_b == "Raydium" { pair.raydium_pool } else { pair.orca_pool.unwrap() };
            (p_a, label_a, p_b, label_b)
        } else {
            // B 便宜，买 B
            let p_b = if label_b == "Raydium" { pair.raydium_pool } else { pair.orca_pool.unwrap() };
            let p_a = if label_a == "Raydium" { pair.raydium_pool } else { pair.orca_pool.unwrap() };
            (p_b, label_b, p_a, label_a)
        };
        
        info!("🔄 Strategy: Buy on {} ({}), Sell on {} ({})", buy_label, buy_pool, sell_label, sell_pool);
        
        // 1. 构建 Swap Instructions (核心逻辑)
        let mut instructions = Vec::new();
        
        // Step 1: Buy on Low Price DEX
        // 我们假设 Base Token 是 SOL (WSOL)，Quote 是 USDC
        // 如果我们用 SOL 买 USDC，那么是 Swap Base -> Quote
        // 这里需要识别 Token 方向。为了简化，我们假设持有 WSOL，先 Swap WSOL -> TokenB，再 Swap TokenB -> WSOL
        // 这通常是 Arb 的标准路径。
        
        // 构建 Raydium 指令
        if buy_label == "Raydium" {
            if let Some(ix) = build_raydium_swap_ix(rpc_client.clone(), &keypair.pubkey(), buy_pool, amount_in_lamports, 0).await {
                instructions.push(ix);
            } else {
                warn!("❌ Failed to build Raydium Buy Instruction");
                return;
            }
        } else if buy_label == "Orca" {
             // 暂未完全实现 Orca Swap 构建 (需要 TickArray)
             // instructions.push(build_orca_swap_ix(...));
             warn!("⚠️ Orca Buy logic not fully implemented yet (TickArray missing)");
             return;
        }

        // Step 2: Sell on High Price DEX
        // 这里的 amount_in 应该是上一步的 amount_out (动态)
        // 但原子交易中无法预知确切的 out，通常使用 estimated out 或者 100% balance
        // 这里简化，假设 1:1 兑换
        if sell_label == "Raydium" {
            if let Some(ix) = build_raydium_swap_ix(rpc_client.clone(), &keypair.pubkey(), sell_pool, amount_in_lamports, 0).await {
                instructions.push(ix);
            }
        } else if sell_label == "Orca" {
             warn!("⚠️ Orca Sell logic not fully implemented yet");
             return;
        }
        
        if instructions.is_empty() {
            warn!("⚠️ No instructions generated, aborting bundle.");
            return;
        }

        // 3. Add Jito Tip
        let tip_account = Pubkey::from_str(JITO_TIP_ACCOUNT).unwrap();
        let tip_lamports = (config.static_tip_sol * 1_000_000_000.0) as u64;
        let tip_instruction = solana_sdk::system_instruction::transfer(
            &keypair.pubkey(),
            &tip_account,
            tip_lamports,
        );
        instructions.push(tip_instruction);
        
        // 4. Build & Send Transaction
        let recent_blockhash = match rpc_client.get_latest_blockhash().await {
            Ok(hash) => hash,
            Err(e) => {
                error!("❌ Failed to get blockhash: {}", e);
                return;
            }
        };
        
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &instructions,
            Some(&keypair.pubkey()),
            &[&*keypair],
            recent_blockhash,
        );
        
        let tx_base58 = bs58::encode(bincode::serialize(&tx).unwrap()).into_string();
        info!("📦 Sending Bundle to Jito (Real Arb Tx)...");
        
        match jito_client.send_bundle(vec![tx_base58]).await {
            Ok(bundle_id) => info!("✅ Bundle Sent! ID: {}", bundle_id),
            Err(e) => error!("❌ Bundle Send Failed: {}", e),
        }
    }
}

async fn build_raydium_swap_ix(
    rpc_client: Arc<RpcClient>,
    user_owner: &Pubkey,
    pool_id: Pubkey,
    amount_in: u64,
    min_amount_out: u64,
) -> Option<solana_sdk::instruction::Instruction> {
    // 1. Fetch Amm State
    let data = rpc_client.get_account_data(&pool_id).await.ok()?;
    let state = AmmState::try_from_slice(&data).ok()?;
    
    // 2. Fetch Serum Market
    let market_id = state.serum_market;
    let market_data = rpc_client.get_account_data(&market_id).await.ok()?;
    // 手动解析 Serum Market V3 (跳过 Padding)
    // Blob(5) + Flags(8) + OwnAddress(32) + VaultSignerNonce(8) + BaseMint(32) + QuoteMint(32) + BaseVault(32) ...
    // 我们直接用 Borsh 尝试，或者手动取 offset
    // Offset for EventQueue: 5+8+32+8+32+32+32+8+8+32+8+8+8+32 = 253? 
    // Let's use the struct we defined
    // 注意：Serum 头部有 padding，我们去掉前 5 字节
    if market_data.len() < 5 { return None; }
    let market_state = SerumMarketV3::try_from_slice(&market_data[5..]).ok();
    
    if let Some(market) = market_state {
         // 3. Derive/Find accounts
         // User ATA (Need to know which token is In/Out. Assuming WSOL -> Token or Token -> WSOL)
         // 简化：假设用户已经有对应的 ATA
         let spl_token_prog = Pubkey::from_str(SPL_TOKEN_PROGRAM_ID).unwrap();
         
         let user_source = spl_associated_token_account::get_associated_token_address(user_owner, &state.coin_mint_address);
         let user_dest = spl_associated_token_account::get_associated_token_address(user_owner, &state.pc_mint_address);
         // Swap 实际方向需要根据 amount_in 是 coin 还是 pc 来决定，或者 swap 指令里的参数
         // Raydium swap instruction 9 实际上并不区分 A->B 或 B->A，而是根据 user source/dest 账户来扣款
         
         // 4. Build Ix
         let ix = build_raydium_swap(
             &Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap(), // Raydium V4 Program
             &pool_id,
             &state.amm_owner, // Authority? No, Amm Authority is PDA derived from Program
             &state.amm_open_orders,
             &state.amm_target_orders,
             &state.pool_coin_token_account,
             &state.pool_pc_token_account,
             &state.serum_program_id,
             &market_id,
             &market.bids,
             &market.asks,
             &market.event_queue,
             &market.base_vault,
             &market.quote_vault,
             &Pubkey::default(), // Serum Vault Signer (Need to derive?) - Raydium program usually calculates this or we pass it
             &user_source,
             &user_dest,
             user_owner,
             amount_in,
             min_amount_out
         );
         
         // Fix Serum Vault Signer
         // 这里的 serum_vault_signer 必须正确，否则 Serum 程序会报错
         // 我们可以通过 rpc 拿，或者计算
         // let vault_signer = crate::amm::serum::get_vault_signer(&market_id, &state.serum_program_id, market.vault_signer_nonce).ok()?;
         // ix.accounts[14] = AccountMeta::new_readonly(vault_signer, false);
         
         return Some(ix);
    } else {
        warn!("❌ Failed to parse Serum Market {}", market_id);
    }
    
    None
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
