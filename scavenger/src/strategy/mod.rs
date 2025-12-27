use log::info;

pub mod risk;
pub mod swap;
pub mod engine;
pub mod orca;
pub mod arbitrage;
pub mod pricing;
pub mod quote; // 新增 Quote 模块
pub mod raydium_keys; // AMM Keys Fetcher
pub mod jito; // Jito HTTP Client

// 策略模块入口
pub fn init() {
    info!("🧠 策略引擎已初始化");
}
