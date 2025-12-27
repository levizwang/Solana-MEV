use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use spl_token::state::Mint;
use solana_sdk::program_pack::Pack;
use log::{info, warn, error};
use std::sync::Arc;

// 风险检查结果
#[derive(Debug)]
pub struct RiskReport {
    pub is_safe: bool,
    pub mint_authority: Option<Pubkey>,
    pub freeze_authority: Option<Pubkey>,
    pub supply: u64,
    pub decimals: u8,
}

pub async fn check_token_risk(rpc_client: &Arc<RpcClient>, mint: &Pubkey) -> Option<RiskReport> {
    // 1. 获取 Mint 账户信息
    match rpc_client.get_account(mint).await {
        Ok(account) => {
            // 2. 解析 Mint 数据
            if let Ok(mint_data) = Mint::unpack(&account.data) {
                let mut is_safe = true;
                
                // 检查 Freeze Authority (必须为 None)
                if mint_data.freeze_authority.is_some() {
                    warn!("⚠️ 风险警告: 代币 {} 存在 Freeze Authority!", mint);
                    is_safe = false;
                }

                // 检查 Mint Authority (最好为 None，但部分新代币可能还没丢弃)
                if mint_data.mint_authority.is_some() {
                    warn!("⚠️ 风险提示: 代币 {} Mint Authority 尚未丢弃!", mint);
                    // 在严格模式下，这可能被视为不安全
                    // is_safe = false; 
                }

                let report = RiskReport {
                    is_safe,
                    mint_authority: mint_data.mint_authority.into(),
                    freeze_authority: mint_data.freeze_authority.into(),
                    supply: mint_data.supply,
                    decimals: mint_data.decimals,
                };
                
                info!("🛡️ 风险检查报告 [{}]: Safe={}, Auth={:?}", mint, is_safe, report.mint_authority);
                return Some(report);
            } else {
                error!("❌ 无法解析 Mint 数据: {}", mint);
            }
        },
        Err(e) => {
            error!("❌ 获取 Mint 账户失败: {} - {}", mint, e);
        }
    }
    None
}
