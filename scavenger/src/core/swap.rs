use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

// Raydium Swap Instruction Data Layout
// discriminator (1 byte) + amount_in (8 bytes) + min_amount_out (8 bytes)
// Swap = 9
#[derive(Clone, Debug, PartialEq)]
pub struct SwapInstructionData {
    pub instruction: u8,
    pub amount_in: u64,
    pub min_amount_out: u64,
}

impl SwapInstructionData {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(17);
        buf.push(self.instruction);
        buf.extend_from_slice(&self.amount_in.to_le_bytes());
        buf.extend_from_slice(&self.min_amount_out.to_le_bytes());
        buf
    }
}

// 构建 Swap 指令
pub fn swap(
    program_id: &Pubkey,
    amm_id: &Pubkey,
    amm_authority: &Pubkey,
    amm_open_orders: &Pubkey,
    amm_target_orders: &Pubkey, // 这里的 target orders 可能不需要，具体看 Raydium 版本，V4 通常需要
    pool_coin_token_account: &Pubkey,
    pool_pc_token_account: &Pubkey,
    serum_program_id: &Pubkey,
    serum_market: &Pubkey,
    serum_bids: &Pubkey,
    serum_asks: &Pubkey,
    serum_event_queue: &Pubkey,
    serum_coin_vault_account: &Pubkey,
    serum_pc_vault_account: &Pubkey,
    serum_vault_signer: &Pubkey,
    user_source_token_account: &Pubkey,
    user_destination_token_account: &Pubkey,
    user_owner: &Pubkey,
    amount_in: u64,
    min_amount_out: u64,
) -> Instruction {
    let data = SwapInstructionData {
        instruction: 9, // Swap Instruction ID
        amount_in,
        min_amount_out,
    };

    let accounts = vec![
        // 1. Token Program
        AccountMeta::new_readonly(spl_token::id(), false),
        // 2. Amm Account
        AccountMeta::new(*amm_id, false),
        // 3. Amm Authority
        AccountMeta::new_readonly(*amm_authority, false),
        // 4. Amm Open Orders
        AccountMeta::new(*amm_open_orders, false),
        // 5. Amm Target Orders (Optional? Raydium V4 uses it)
        AccountMeta::new(*amm_target_orders, false),
        // 6. Pool Coin Vault
        AccountMeta::new(*pool_coin_token_account, false),
        // 7. Pool Pc Vault
        AccountMeta::new(*pool_pc_token_account, false),
        // 8. Serum Program ID
        AccountMeta::new_readonly(*serum_program_id, false),
        // 9. Serum Market
        AccountMeta::new(*serum_market, false),
        // 10. Serum Bids
        AccountMeta::new(*serum_bids, false),
        // 11. Serum Asks
        AccountMeta::new(*serum_asks, false),
        // 12. Serum Event Queue
        AccountMeta::new(*serum_event_queue, false),
        // 13. Serum Coin Vault
        AccountMeta::new(*serum_coin_vault_account, false),
        // 14. Serum Pc Vault
        AccountMeta::new(*serum_pc_vault_account, false),
        // 15. Serum Vault Signer
        AccountMeta::new_readonly(*serum_vault_signer, false),
        // 16. User Source Token Account
        AccountMeta::new(*user_source_token_account, false),
        // 17. User Destination Token Account
        AccountMeta::new(*user_destination_token_account, false),
        // 18. User Owner
        AccountMeta::new_readonly(*user_owner, true),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: data.to_vec(),
    }
}

// Orca Whirlpool Swap Instruction
// Discriminator for "swap": global:swap -> [248, 198, 158, 145, 225, 117, 135, 200]
#[derive(Clone, Debug, PartialEq)]
pub struct OrcaSwapInstructionData {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit: u128,
    pub amount_specified_is_input: bool,
    pub a_to_b: bool, // True if swapping Token A to Token B
}

impl OrcaSwapInstructionData {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + 8 + 8 + 16 + 1 + 1);
        // Discriminator
        buf.extend_from_slice(&[248, 198, 158, 145, 225, 117, 135, 200]);
        buf.extend_from_slice(&self.amount.to_le_bytes());
        buf.extend_from_slice(&self.other_amount_threshold.to_le_bytes());
        buf.extend_from_slice(&self.sqrt_price_limit.to_le_bytes());
        buf.push(self.amount_specified_is_input as u8);
        buf.push(self.a_to_b as u8);
        buf
    }
}

pub fn build_orca_swap(
    program_id: &Pubkey,
    token_program: &Pubkey,
    token_authority: &Pubkey,
    whirlpool: &Pubkey,
    token_owner_account_a: &Pubkey,
    token_vault_a: &Pubkey,
    token_owner_account_b: &Pubkey,
    token_vault_b: &Pubkey,
    tick_array_0: &Pubkey,
    tick_array_1: &Pubkey,
    oracle: &Pubkey,
    amount: u64,
    other_amount_threshold: u64,
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
) -> Instruction {
    let data = OrcaSwapInstructionData {
        amount,
        other_amount_threshold,
        sqrt_price_limit,
        amount_specified_is_input,
        a_to_b,
    };

    let accounts = vec![
        AccountMeta::new_readonly(*token_program, false),
        AccountMeta::new_readonly(*token_authority, true),
        AccountMeta::new(*whirlpool, false),
        AccountMeta::new(*token_owner_account_a, false),
        AccountMeta::new(*token_vault_a, false),
        AccountMeta::new(*token_owner_account_b, false),
        AccountMeta::new(*token_vault_b, false),
        AccountMeta::new(*tick_array_0, false),
        AccountMeta::new(*tick_array_1, false),
        AccountMeta::new_readonly(*oracle, false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: data.to_vec(),
    }
}
