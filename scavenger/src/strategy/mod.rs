use log::info;

pub mod risk;
pub mod swap;
pub mod engine;
pub mod orca;
pub mod arbitrage;
pub mod pricing;

// 策略模块入口
pub fn init() {
    info!("🧠 策略引擎已初始化");
}
