use solana_sdk::{
    instruction::Instruction,
    transaction::Transaction,
    signature::{Keypair, Signer},
    pubkey::Pubkey,
    hash::Hash,
};
use log::info;

// 原子交易构建器
pub struct AtomicTransactionBuilder {
    instructions: Vec<Instruction>,
    payer: Pubkey,
}

impl AtomicTransactionBuilder {
    pub fn new(payer: Pubkey) -> Self {
        Self {
            instructions: Vec::new(),
            payer,
        }
    }

    // 添加 Swap 指令
    pub fn add_swap_ix(&mut self, ix: Instruction) -> &mut Self {
        self.instructions.push(ix);
        self
    }

    // 添加 Jito Tip 指令
    pub fn add_tip_ix(&mut self, tip_account: Pubkey, amount: u64) -> &mut Self {
        // Jito Tip 本质上是一个 System Transfer
        let transfer_ix = solana_sdk::system_instruction::transfer(
            &self.payer,
            &tip_account,
            amount,
        );
        self.instructions.push(transfer_ix);
        self
    }

    // 构建并签名交易
    pub fn build(&self, recent_blockhash: Hash, keypair: &Keypair) -> Transaction {
        let tx = Transaction::new_signed_with_payer(
            &self.instructions,
            Some(&self.payer),
            &[keypair],
            recent_blockhash,
        );
        info!("⚛️ 原子交易构建完成，包含 {} 条指令", self.instructions.len());
        tx
    }
}

// 套利路径枚举
pub enum ArbitragePath {
    RaydiumToOrca,
    OrcaToRaydium,
}

// 示例：构建套利交易
pub fn build_arbitrage_tx(
    payer: &Keypair,
    path: ArbitragePath,
    raydium_ix: Instruction,
    orca_ix: Instruction,
    tip_account: Pubkey,
    tip_amount: u64,
    recent_blockhash: Hash,
) -> Transaction {
    let mut builder = AtomicTransactionBuilder::new(payer.pubkey());

    match path {
        ArbitragePath::RaydiumToOrca => {
            builder
                .add_swap_ix(raydium_ix)
                .add_swap_ix(orca_ix);
        }
        ArbitragePath::OrcaToRaydium => {
            builder
                .add_swap_ix(orca_ix)
                .add_swap_ix(raydium_ix);
        }
    }

    builder
        .add_tip_ix(tip_account, tip_amount)
        .build(recent_blockhash, payer)
}
