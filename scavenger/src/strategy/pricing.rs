use log::info;

// 利润计算配置
pub struct ProfitConfig {
    pub min_profit_sol: f64,    // 最小净利润 (SOL)
    pub max_jito_tip_sol: f64,  // 最大小费 (SOL)
    pub gas_cost_sol: f64,      // 预估 Gas 费 (SOL)
}

impl Default for ProfitConfig {
    fn default() -> Self {
        Self {
            min_profit_sol: 0.01, // 0.01 SOL
            max_jito_tip_sol: 0.1, // 0.1 SOL
            gas_cost_sol: 0.000005, // 5000 Lamports
        }
    }
}

pub struct SimulationResult {
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: f64,
}

// 简单的利润计算器
pub fn calculate_profit(
    config: &ProfitConfig,
    input_amount_sol: f64,
    raydium_out_sol: f64,
    orca_out_sol: f64,
) -> Option<f64> {
    // 假设路径: SOL -> Token (Raydium) -> SOL (Orca)
    // 或者反过来
    
    // 这里我们比较两个输出：
    // 如果 Raydium 价格更低 (买入)，Orca 价格更高 (卖出)
    // 那么 input = input_amount_sol
    // intermediate = amount of Token
    // final_output = orca_out_sol
    
    // 简化模型：直接比较两个市场的 SOL 价值
    // 比如 1 SOL 在 Raydium 能换 100 Token
    // 100 Token 在 Orca 能换 1.1 SOL
    // 毛利 = 1.1 - 1.0 = 0.1 SOL
    
    let gross_profit = if raydium_out_sol > input_amount_sol {
        // Raydium 卖出获利? 
        // 这里的逻辑需要明确是 循环套利 (Cycle) 还是 空间套利 (Spatial)
        // 假设是 Spatial Arbitrage:
        // Case 1: Buy Raydium -> Sell Orca
        // output_orca - input_raydium
        orca_out_sol - input_amount_sol
    } else {
        0.0
    };
    
    // 计算净利润
    // Net = Gross - Gas - Tip
    // 这里 Tip 通常是 Gross 的一部分 (比如 50%)
    
    if gross_profit <= 0.0 {
        return None;
    }

    let potential_tip = gross_profit * 0.5; // 给 Jito 50% 利润
    let final_tip = if potential_tip > config.max_jito_tip_sol {
        config.max_jito_tip_sol
    } else {
        potential_tip
    };

    let net_profit = gross_profit - config.gas_cost_sol - final_tip;

    if net_profit > config.min_profit_sol {
        info!("💰 发现套利机会! 毛利: {:.4} SOL, 净利: {:.4} SOL, Tip: {:.4}", 
            gross_profit, net_profit, final_tip);
        Some(final_tip)
    } else {
        // warn!("📉 利润不足: {:.6} SOL", net_profit);
        None
    }
}
