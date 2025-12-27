use log::info;

pub mod risk;
pub mod swap;
pub mod engine;

// 策略模块入口
pub fn init() {
    info!("🧠 策略引擎已初始化");
}
