use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

/// Orca Whirlpool Account Layout
/// Source: https://github.com/orca-so/whirlpools/blob/main/programs/whirlpool/src/state/whirlpool.rs
/// Discriminator: [62, 10, 14, 196, 56, 60, 89, 21] (Already handled in fetcher)
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Whirlpool {
    pub whirlpools_config: Pubkey, // 32
    pub whirlpool_bump: [u8; 1],   // 1
    pub tick_spacing: u16,         // 2
    pub tick_spacing_seed: [u8; 2],// 2
    pub fee_rate: u16,             // 2
    pub protocol_fee_rate: u16,    // 2
    pub liquidity: u128,           // 16
    pub sqrt_price: u128,          // 16 (Q64.64)
    pub tick_current_index: i32,   // 4
    pub protocol_fee_owed_a: u64,  // 8
    pub protocol_fee_owed_b: u64,  // 8
    pub token_mint_a: Pubkey,      // 32
    pub token_vault_a: Pubkey,     // 32
    pub fee_growth_global_a: u128, // 16
    pub token_mint_b: Pubkey,      // 32
    pub token_vault_b: Pubkey,     // 32
    pub fee_growth_global_b: u128, // 16
    pub reward_last_updated_timestamp: u64, // 8
    
    // RewardInfos (3 items, complex struct, simplifying for now as we just need price)
    // 注意: Borsh 反序列化必须严格匹配布局。如果这里不写全，反序列化会失败。
    // 为了稳健，我们暂时只解析到 reward_last_updated_timestamp 之前的数据，
    // 或者我们需要完整定义 RewardInfo。
    // Whirlpool struct is quite large.
    // 更好的方法是只解析我们需要的前半部分，或者手动切片 data buffer。
}

#[derive(Debug, Clone)]
pub struct WhirlpoolPrice {
    pub price: f64,
    pub sqrt_price_x64: u128,
    pub tick: i32,
    pub liquidity: u128,
}

impl Whirlpool {
    /// Manual parsing from data slice to avoid defining the full complex struct
    /// Whirlpool layout:
    /// discriminator: 8 bytes
    /// config: 32
    /// bump: 1
    /// tick_spacing: 2
    /// tick_spacing_seed: 2
    /// fee_rate: 2
    /// protocol_fee_rate: 2
    /// liquidity: 16
    /// sqrt_price: 16
    /// tick_current_index: 4
    /// ...
    pub fn decode_current_price(data: &[u8]) -> Option<WhirlpoolPrice> {
        // Offset check
        // 8 (discriminator) + 32 + 1 + 2 + 2 + 2 + 2 = 49 bytes before liquidity
        // liquidity: 49..65
        // sqrt_price: 65..81
        // tick_current_index: 81..85
        
        if data.len() < 85 {
            return None;
        }

        let liquidity_bytes: [u8; 16] = data[49..65].try_into().ok()?;
        let sqrt_price_bytes: [u8; 16] = data[65..81].try_into().ok()?;
        let tick_bytes: [u8; 4] = data[81..85].try_into().ok()?;

        let liquidity = u128::from_le_bytes(liquidity_bytes);
        let sqrt_price_x64 = u128::from_le_bytes(sqrt_price_bytes);
        let tick = i32::from_le_bytes(tick_bytes);

        Some(WhirlpoolPrice {
            price: sqrt_price_x64_to_price(sqrt_price_x64),
            sqrt_price_x64,
            tick,
            liquidity,
        })
    }
}

/// Convert Q64.64 sqrt_price to f64 price
/// Price = (sqrt_price / 2^64)^2
pub fn sqrt_price_x64_to_price(sqrt_price_x64: u128) -> f64 {
    let sqrt_price = sqrt_price_x64 as f64;
    let q64 = (1u128 << 64) as f64;
    let p = sqrt_price / q64;
    p * p
}

/// Convert tick index to price
/// Price = 1.0001 ^ tick
pub fn tick_to_price(tick: i32) -> f64 {
    1.0001f64.powi(tick)
}
