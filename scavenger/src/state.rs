use std::sync::Arc;
use dashmap::DashMap;
use solana_sdk::pubkey::Pubkey;

/// 全网代币索引 (In-Memory Inventory)
/// 核心数据结构: TokenMint -> Vec<PoolAddress>
/// 用于快速查找某个 Token 在 Orca 上是否有流动性池
#[derive(Debug, Clone)]
pub struct Inventory {
    // Key: Token Mint, Value: List of Orca Whirlpool Addresses
    // 一个 Token 可能对应多个池子 (不同的 Fee Tier, 不同的配对如 SOL/USDC)
    pub orca_pools: Arc<DashMap<Pubkey, Vec<Pubkey>>>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            orca_pools: Arc::new(DashMap::new()),
        }
    }

    /// 添加一个新的 Orca 池子到索引中
    /// 通常在启动时全量加载，或监听到 InitializePool 事件时调用
    pub fn add_pool(&self, token_mint_a: Pubkey, token_mint_b: Pubkey, pool_address: Pubkey) {
        // 索引 Token A -> Pool
        let mut pools_a = self.orca_pools.entry(token_mint_a).or_insert(Vec::new());
        if !pools_a.contains(&pool_address) {
            pools_a.push(pool_address);
        }

        // 索引 Token B -> Pool
        let mut pools_b = self.orca_pools.entry(token_mint_b).or_insert(Vec::new());
        if !pools_b.contains(&pool_address) {
            pools_b.push(pool_address);
        }
    }

    /// 获取某个 Token 参与的所有 Orca 池子
    pub fn get_pools(&self, token_mint: &Pubkey) -> Option<Vec<Pubkey>> {
        self.orca_pools.get(token_mint).map(|v| v.clone())
    }
    
    /// 快速检查某个 Token 是否在 Orca 上有流动性
    pub fn has_liquidity(&self, token_mint: &Pubkey) -> bool {
        self.orca_pools.contains_key(token_mint)
    }

    /// 获取当前索引的统计信息
    pub fn stats(&self) -> (usize, usize) {
        // (Token 数量, 总索引条目数)
        let token_count = self.orca_pools.len();
        // 简单的估算，不遍历
        (token_count, 0) 
    }
}
