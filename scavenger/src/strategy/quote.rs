use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use log::{info, error};
use borsh::BorshDeserialize;
use crate::amm::raydium_v4::AmmState;
use crate::amm::math;

// 模拟获取 Raydium 报价 (Quote)
// 实际实现需要：
// 1. 获取 Pool Account Data
// 2. 解析 Reserve A 和 Reserve B
// 3. 计算 Constant Product (x * y = k)
pub async fn get_raydium_quote(
    rpc_client: Arc<RpcClient>,
    pool_id: &Pubkey,
    amount_in: u64,
    input_mint: &Pubkey,
) -> Option<u64> {
    // 1. 获取 Pool Account State
    let account = match rpc_client.get_account(pool_id).await {
        Ok(acc) => acc,
        Err(e) => {
            error!("❌ 无法获取 Pool 账户: {} - {}", pool_id, e);
            return None;
        }
    };

    // 2. 反序列化 AMM State
    let amm_state = match AmmState::try_from_slice(&account.data) {
        Ok(state) => state,
        Err(e) => {
            error!("❌ 解析 AMM State 失败: {} - {}", pool_id, e);
            return None;
        }
    };

    // 3. 获取 Vault 余额 (Reserves)
    // 实际的 Reserve 应该查 Vault Account 的 Balance，而不是 AmmState 中的缓存值 (如果不可信)
    // Raydium State 中没有直接存储实时 reserve，而是存储了 need_take_pnl 等
    // 我们必须查 Vault Token Account
    
    let coin_vault = amm_state.pool_coin_token_account;
    let pc_vault = amm_state.pool_pc_token_account;
    
    let reserve_coin = get_token_balance(&rpc_client, &coin_vault).await?;
    let reserve_pc = get_token_balance(&rpc_client, &pc_vault).await?;
    
    // 4. 确定方向
    // 如果 input_mint == coin_mint, 则是 Coin -> PC
    // 如果 input_mint == pc_mint, 则是 PC -> Coin
    
    let (reserve_in, reserve_out) = if *input_mint == amm_state.coin_mint_address {
        (reserve_coin, reserve_pc)
    } else if *input_mint == amm_state.pc_mint_address {
        (reserve_pc, reserve_coin)
    } else {
        error!("❌ 输入代币 {} 不属于该 Pool {}", input_mint, pool_id);
        return None;
    };

    // 5. 计算 Output (Constant Product Formula)
    let amount_out = math::get_amount_out(
        amount_in,
        reserve_in,
        reserve_out,
        amm_state.swap_fee_numerator,
        amm_state.swap_fee_denominator,
    )?;
    
    info!("🧮 链上计算: Pool={}, In={}, ReserveIn={}, ReserveOut={}, Out={}", 
        pool_id, amount_in, reserve_in, reserve_out, amount_out);

    Some(amount_out)
}

// 辅助：获取 Token 余额
pub async fn get_token_balance(rpc_client: &RpcClient, vault: &Pubkey) -> Option<u64> {
    match rpc_client.get_token_account_balance(vault).await {
        Ok(ui_amount) => {
            ui_amount.amount.parse::<u64>().ok()
        },
        Err(_) => None
    }
}
