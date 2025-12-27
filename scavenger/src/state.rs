use std::sync::Arc;
use dashmap::DashMap;
use solana_sdk::pubkey::Pubkey;
use crate::scout::api::{fetch_raydium_pools, fetch_orca_pools};
use log::info;
// use std::collections::HashSet;

/// 套利对结构体
#[derive(Debug, Clone)]
pub struct ArbitragePair {
    pub token_mint: Pubkey,
    pub raydium_pool: Pubkey,
    pub orca_pool: Option<Pubkey>,
    pub meteora_pool: Option<Pubkey>,
}

/// 全网代币索引 (In-Memory Inventory)
/// 核心数据结构: TokenMint -> Vec<PoolAddress>
/// 用于快速查找某个 Token 在 Orca 上是否有流动性池
#[derive(Debug, Clone)]
pub struct Inventory {
    // Key: Token Mint, Value: List of Orca Whirlpool Addresses
    // 一个 Token 可能对应多个池子 (不同的 Fee Tier, 不同的配对如 SOL/USDC)
    pub orca_pools: Arc<DashMap<Pubkey, Vec<Pubkey>>>,
    
    // Key: Token Mint, Value: ArbitragePair
    pub common_pairs: Arc<DashMap<Pubkey, ArbitragePair>>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            orca_pools: Arc::new(DashMap::new()),
            common_pairs: Arc::new(DashMap::new()),
        }
    }

    /// 从 API 加载并构建共有白名单
    pub async fn load_from_api(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 1. 并发获取两个 DEX 的池子
        let (ray_pools, orca_pools) = tokio::join!(
            fetch_raydium_pools(),
            fetch_orca_pools()
        );

        let ray_pools = ray_pools?;
        let orca_pools = orca_pools?;

        // 2. 构建映射以便查找
        // TokenMint -> RaydiumPool (假设每个 Token 只有一个主要池子，或者取第一个)
        let mut ray_map = std::collections::HashMap::new();
        for p in ray_pools {
            // 我们关注的是非 Quote Token (即 Base Token)
            // 简单逻辑：如果 A 是 SOL/USDC，则 B 是 Base；反之亦然。
            // 这里为了简化，我们把 TokenA 和 TokenB 都作为 Key 存一下
            ray_map.insert(p.token_a, p.address);
            // ray_map.insert(p.token_b, p.address); 
            // 注意：这样可能会覆盖。更好的做法是识别 Base Token。
            // 但 API 返回的通常 TokenA 是 Base。
        }

        // 3. 遍历 Orca 池子，寻找交集
        let mut count = 0;
        for p in orca_pools {
            // 检查 Token A 是否在 Raydium 有池子
            if let Some(ray_addr) = ray_map.get(&p.token_a) {
                self.common_pairs.insert(p.token_a, ArbitragePair {
                    token_mint: p.token_a,
                    raydium_pool: *ray_addr,
                    orca_pool: Some(p.address),
                    meteora_pool: None,
                });
                count += 1;
            }
            
            // 同时也添加单纯的 Orca 池子到 orca_pools 索引中 (为了 Sniper 策略兼容)
            self.add_pool(p.token_a, p.token_b, p.address);
        }

        info!("✅ Loaded {} common arbitrage pairs from Raydium/Orca", count);
        Ok(())
    }

    /// 获取需要监听的 Pool Address 列表 (用于 Geyser/WebSocket)
    pub fn get_watch_list(&self) -> Vec<Pubkey> {
        let mut list = Vec::new();
        for entry in self.common_pairs.iter() {
            let pair = entry.value();
            list.push(pair.raydium_pool);
            if let Some(orca_pool) = pair.orca_pool {
                list.push(orca_pool);
            }
        }
        list
    }
    
    /// 根据 Pool Address 查找所属的 ArbitragePair
    pub fn find_pair_by_pool(&self, pool_address: &Pubkey) -> Option<ArbitragePair> {
        for entry in self.common_pairs.iter() {
            let pair = entry.value();
            if pair.raydium_pool == *pool_address || pair.orca_pool == Some(*pool_address) {
                return Some(pair.clone());
            }
        }
        None
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
