use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

// Raydium AMM V4 State Layout (752 bytes)
// 参考: https://github.com/raydium-io/raydium-amm/blob/master/program/src/state.rs
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Copy)]
#[repr(C)]
pub struct AmmState {
    pub status: u64,
    pub nonce: u64,
    pub order_num: u64,
    pub depth: u64,
    pub coin_decimals: u64,
    pub pc_decimals: u64,
    pub state: u64,
    pub reset_flag: u64,
    pub min_size: u64,
    pub vol_max_cut_ratio: u64,
    pub amount_wave_ratio: u64,
    pub coin_lot_size: u64,
    pub pc_lot_size: u64,
    pub min_price_multiplier: u64,
    pub max_price_multiplier: u64,
    pub system_decimal_value: u64,
    
    // Fees
    pub min_separate_numerator: u64,
    pub min_separate_denominator: u64,
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub pnl_numerator: u64,
    pub pnl_denominator: u64,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
    
    // OutPutData
    pub need_take_pnl_coin: u64,
    pub need_take_pnl_pc: u64,
    pub total_pnl_pc: u64,
    pub total_pnl_coin: u64,
    
    pub pool_open_time: u64,
    pub punish_pc_amount: u64,
    pub punish_coin_amount: u64,
    pub order_max_ts: u64,
    pub order_start_ts: u64, // 0x120
    
    pub pool_total_deposit_pc: u64,
    pub pool_total_deposit_coin: u64,
    pub swap_coin_in_amount: u64,
    pub swap_pc_out_amount: u64,
    pub swap_coin_2pc_fee: u64,
    pub swap_pc_in_amount: u64,
    pub swap_coin_out_amount: u64,
    pub swap_pc_2coin_fee: u64,
    
    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub coin_mint_address: Pubkey,
    pub pc_mint_address: Pubkey,
    pub lp_mint_address: Pubkey,
    pub amm_open_orders: Pubkey,
    pub serum_market: Pubkey,
    pub serum_program_id: Pubkey,
    pub amm_target_orders: Pubkey,
    pub pool_withdraw_queue: Pubkey,
    pub pool_temp_lp_token_account: Pubkey,
    pub amm_owner: Pubkey,
    pub pnl_owner: Pubkey,
}

impl AmmState {
    pub const LEN: usize = 752;
}
