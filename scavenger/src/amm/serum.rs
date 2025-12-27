use solana_sdk::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct SerumMarketV3 {
    pub blob_header: [u8; 5],
    pub account_flags: [u8; 8],
    pub own_address: Pubkey,
    pub vault_signer_nonce: u64,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub base_deposits_total: u64,
    pub base_fees_accrued: u64,
    pub quote_vault: Pubkey,
    pub quote_deposits_total: u64,
    pub quote_fees_accrued: u64,
    pub quote_dust_threshold: u64,
    pub request_queue: Pubkey,
    pub event_queue: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub fee_rate_bps: u64,
    pub referrer_rebates_accrued: u64,
}

impl SerumMarketV3 {
    // Serum Market V3 usually has a specific padding, but standard Borsh might work if the layout matches.
    // However, Serum uses "safe-transmute" style casting often.
    // The manual layout is:
    // blob_header: 5 bytes ("serum" padding)
    // account_flags: 8 bytes
    // own_address: 32
    // vault_signer_nonce: 8
    // base_mint: 32
    // quote_mint: 32
    // base_vault: 32
    // ...
    // We can try BorshDeserialize. If it fails, we do manual offset reading.
}

pub fn get_vault_signer(market: &Pubkey, market_program_id: &Pubkey, nonce: u64) -> Result<Pubkey, solana_sdk::pubkey::PubkeyError> {
    solana_sdk::pubkey::Pubkey::create_program_address(
        &[&market.to_bytes(), &[nonce as u8]],
        market_program_id,
    )
}
