use shipstern::Pipeline;
use shipstern_solana_rpc_source::SolanaAccountsRpcSource;
use sources::Logger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// #region rpc-backfill
// Loads every account the SPL Stake Pool program owns, parses each one, then exits.
fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = std::fs::read_to_string("Shipstern.toml").expect("Error reading config file");
    let config = toml::from_str(&config)
        .unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    shipstern::Runtime::builder()
        .account(Pipeline::new(shipstern_stake_pool_parser::AccountParser, [Logger]))
        .build::<SolanaAccountsRpcSource>(config)
        .run();
}
// #endregion rpc-backfill
