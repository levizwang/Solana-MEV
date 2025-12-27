use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use borsh::{BorshSerialize, BorshDeserialize};

// Orca Whirlpool Swap Instruction Data
// Discriminator: [248, 198, 158, 145, 225, 117, 135, 200] (Swap)
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct WhirlpoolSwapData {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit: u128,
    pub amount_specified_is_input: bool,
    pub a_to_b: bool,
}

// 构建 Orca Whirlpool Swap 指令
pub fn swap(
    whirlpool_program_id: &Pubkey,
    token_program: &Pubkey,
    token_authority: &Pubkey,
    whirlpool: &Pubkey,
    token_owner_account_a: &Pubkey,
    token_vault_a: &Pubkey,
    token_owner_account_b: &Pubkey,
    token_vault_b: &Pubkey,
    tick_array_0: &Pubkey,
    tick_array_1: &Pubkey,
    tick_array_2: &Pubkey,
    oracle: &Pubkey,
    amount: u64,
    other_amount_threshold: u64,
    sqrt_price_limit: u128,
    is_input: bool,
    a_to_b: bool,
) -> Instruction {
    // Discriminator for "swap"
    let discriminator: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
    
    let data = WhirlpoolSwapData {
        amount,
        other_amount_threshold,
        sqrt_price_limit,
        amount_specified_is_input: is_input,
        a_to_b,
    };
    
    let mut data_vec = Vec::new();
    data_vec.extend_from_slice(&discriminator);
    data_vec.extend_from_slice(&data.try_to_vec().unwrap());

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
        AccountMeta::new(*tick_array_2, false),
        AccountMeta::new_readonly(*oracle, false),
    ];

    Instruction {
        program_id: *whirlpool_program_id,
        accounts,
        data: data_vec,
    }
}
