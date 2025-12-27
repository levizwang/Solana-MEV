use std::sync::Arc;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use log::{info, warn, error};
use std::str::FromStr;

use crate::scout::raydium::NewPoolEvent;
use crate::scout::orca::OrcaPoolEvent;
use crate::strategy::risk;
use crate::config::StrategyConfig;
use crate::state::Inventory;
use crate::amm::orca_whirlpool::Whirlpool;

// Constants for Quote Tokens
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

// 处理 Raydium 新池事件
pub async fn process_new_pool(
    rpc_client: Arc<RpcClient>,
    _keypair: Arc<Keypair>, 
    event: NewPoolEvent,
    config: Arc<StrategyConfig>,
    inventory: Arc<Inventory>,
) {
    info!("⚙️ [Strategy] 收到 Raydium 新池: {} | Token A: {} | Token B: {}", event.pool_id, event.token_a, event.token_b);

    // 1. 风险检查 (Honeypot Check) - 优先检查
    // 识别 Base Token (非 SOL/USDC 的那个)
    let base_token = if is_quote_token(&event.token_a) { event.token_b } else { event.token_a };
    
    // 快速风险过滤 (仅作示例，实际可并行)
    // if let Some(risk_report) = risk::check_token_risk(&rpc_client, &base_token).await {
    //     if !risk_report.is_safe {
    //         warn!("🛑 [Risk] 风险检查未通过: {}, 跳过", base_token);
    //         return;
    //     }
    // }

    // 2. 核心联动: 查全网代币索引 (Inventory)
    // 检查 Base Token 是否在 Orca 上有流动性
    if inventory.has_liquidity(&base_token) {
        info!("🎯 [Match] 命中! Token {} 在 Orca 上存在流动性池", base_token);
        
        // 3. 并行获取价格 (Raydium Initial Price vs Orca Current Price)
        let orca_pools = inventory.get_pools(&base_token).unwrap_or_default();
        if orca_pools.is_empty() {
            warn!("⚠️ [Inventory] 数据不一致: has_liquidity 为 true 但 pool 列表为空");
            return;
        }

        // 简单起见，取第一个 Orca 池子
        let orca_pool_id = orca_pools[0];
        
        // 3.1 获取 Orca 价格
        let orca_price_task = get_orca_price(rpc_client.clone(), orca_pool_id);
        
        // 3.2 获取 Raydium 价格 (这里暂时模拟，因为解析 Raydium AMM State 比较复杂)
        // 实际逻辑: Fetch Raydium Pool Account -> Parse Vaults -> Fetch Vault Balances -> Divide
        let ray_price_task = mock_get_raydium_price(); 

        let (orca_res, ray_res) = tokio::join!(orca_price_task, ray_price_task);

        if let (Some(orca_p), Some(ray_p)) = (orca_res, ray_res) {
            info!("📊 [Price Check] Orca: ${:.6} | Raydium: ${:.6}", orca_p, ray_p);
            
            // 4. 计算价差
            let spread = (orca_p - ray_p) / ray_p;
            info!("📈 [Spread] 价差: {:.2}%", spread * 100.0);

            if spread > 0.05 { // 5% 阈值
                info!("🚀 [EXECUTE] 触发套利! 买入 Raydium -> 卖出 Orca");
                // execute_arbitrage(...)
            } else if spread < -0.05 {
                 info!("🚀 [EXECUTE] 触发套利! 买入 Orca -> 卖出 Raydium");
            } else {
                info!("zzz [Skip] 价差不足，忽略");
            }
        } else {
            warn!("⚠️ 无法获取完整价格数据，跳过比对");
        }

    } else {
        info!("❄️ [No Match] Token {} 在 Orca 无流动性，进入纯狙击模式 (Sniping Mode)", base_token);
        // 执行纯狙击策略 (Buy -> Wait -> Sell)
    }
}

// 处理 Orca 事件 (保留原有逻辑，可扩展)
pub async fn process_orca_event(
    rpc_client: Arc<RpcClient>,
    keypair: Arc<Keypair>,
    event: OrcaPoolEvent,
    config: Arc<StrategyConfig>,
) {
    // 这里的逻辑也可以升级，反向查 Raydium
    info!("⚙️ [Strategy-Orca] Pool Event: {}", event.pool_id);
}

// --- Helpers ---

fn is_quote_token(mint: &Pubkey) -> bool {
    let s = mint.to_string();
    s == SOL_MINT || s == USDC_MINT
}

async fn get_orca_price(rpc_client: Arc<RpcClient>, pool_id: Pubkey) -> Option<f64> {
    match rpc_client.get_account_data(&pool_id).await {
        Ok(data) => {
            if let Some(price_info) = Whirlpool::decode_current_price(&data) {
                return Some(price_info.price);
            }
        },
        Err(e) => error!("❌ Fetch Orca Pool Error: {}", e),
    }
    None
}

async fn mock_get_raydium_price() -> Option<f64> {
    // 模拟一个价格，用于演示流程
    // 实际开发中需要实现真正的 fetch & calc
    Some(0.123456)
}
