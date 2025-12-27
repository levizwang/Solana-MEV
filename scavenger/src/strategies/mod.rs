pub mod arb;
pub mod sniper;

// use solana_client::nonblocking::rpc_client::RpcClient;
// use solana_sdk::signature::Keypair;
// use std::sync::Arc;
// use crate::config::StrategyConfig;
// use crate::state::Inventory;

// Removed async_trait for now as we use static dispatch in monitor.rs
// #[async_trait::async_trait]
// pub trait Strategy {
//     async fn execute(
//         &self,
//         rpc_client: Arc<RpcClient>,
//         keypair: Arc<Keypair>,
//         config: Arc<StrategyConfig>,
//         inventory: Arc<Inventory>,
//     );
// }
