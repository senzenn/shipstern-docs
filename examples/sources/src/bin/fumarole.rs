use shipstern::Pipeline;
use shipstern_spl_token_parser::{AccountParser, InstructionParser};
use shipstern_yellowstone_fumarole_source::YellowstoneFumaroleSource;
use sources::Logger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = std::fs::read_to_string("Shipstern.toml").expect("Error reading config file");
    let config = toml::from_str(&config)
        .unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    shipstern::Runtime::builder()
        .account(Pipeline::new(AccountParser, [Logger]))
        .instruction(Pipeline::new(InstructionParser, [Logger]))
        .build::<YellowstoneFumaroleSource>(config)
        .run_async()
        .await;
}
