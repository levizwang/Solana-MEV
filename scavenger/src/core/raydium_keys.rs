use std::sync::Arc;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use borsh::BorshDeserialize;
use crate::amm::raydium_v4::AmmState;
use solana_sdk::account::Account;

#[derive(Debug, Clone)]
pub struct RaydiumPoolKeys {
    pub id: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub open_orders: Pubkey,
    pub target_orders: Pubkey,
    pub market_program_id: Pubkey,
    pub market_id: Pubkey,
    pub amm_authority: Pubkey, // derived
    // ... we can add market keys (bids/asks/event_queue) later if needed for full serum swap
    // Raydium V4 swap needs these.
}

/// Fetch and decode Raydium AMM State
pub async fn fetch_raydium_keys(rpc_client: Arc<RpcClient>, pool_id: &Pubkey) -> Option<RaydiumPoolKeys> {
    match rpc_client.get_account(pool_id).await {
        Ok(account) => {
            decode_raydium_keys(pool_id, &account)
        },
        Err(e) => {
            log::error!("❌ Fetch Raydium AMM Account Error: {}", e);
            None
        }
    }
}

pub fn decode_raydium_keys(pool_id: &Pubkey, account: &Account) -> Option<RaydiumPoolKeys> {
    if account.data.len() != AmmState::LEN {
        log::warn!("⚠️ Raydium AMM State Length Mismatch: Expected {}, Got {}", AmmState::LEN, account.data.len());
        // In reality, it might be safer to check >= LEN
        if account.data.len() < AmmState::LEN { return None; }
    }

    let amm_state = match AmmState::try_from_slice(&account.data[..AmmState::LEN]) {
        Ok(s) => s,
        Err(e) => {
            log::error!("❌ Decode Raydium AMM State Error: {}", e);
            return None;
        }
    };

    // Calculate Authority
    // let (authority, _) = Pubkey::find_program_address(&[b"amm authority"], &program_id);
    // Hardcode or derive? Raydium V4 authority is usually derived.
    // For now we assume we just need the state keys.

    Some(RaydiumPoolKeys {
        id: *pool_id,
        base_mint: amm_state.coin_mint_address,
        quote_mint: amm_state.pc_mint_address,
        lp_mint: amm_state.lp_mint_address,
        base_vault: amm_state.pool_coin_token_account,
        quote_vault: amm_state.pool_pc_token_account,
        open_orders: amm_state.amm_open_orders,
        target_orders: amm_state.amm_target_orders,
        market_program_id: amm_state.serum_program_id,
        market_id: amm_state.serum_market,
        amm_authority: Pubkey::default(), // TODO: Derive properly if needed
    })
}
