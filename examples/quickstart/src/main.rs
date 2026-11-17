use std::path::PathBuf;

use clap::Parser;
use shipstern::Pipeline;
use shipstern_spl_token_parser::{AccountParser, InstructionParser};
use shipstern_yellowstone_grpc_source::YellowstoneGrpcSource;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(clap::Parser)]
#[command(version, author, about)]
pub struct Opts {
    #[arg(long, short)]
    config: PathBuf,
}

#[derive(Debug)]
pub struct Logger;

impl<V: std::fmt::Debug + Sync, R: Sync> shipstern::Handler<V, R> for Logger {
    async fn handle(&self, value: &V, _raw: &R) -> shipstern::HandlerResult<()> {
        tracing::info!(?value);
        Ok(())
    }
}

fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let Opts { config } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let config = toml::from_str(&config)
        .unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    shipstern::Runtime::builder()
        .account(Pipeline::new(AccountParser, [Logger]))
        .instruction(Pipeline::new(InstructionParser, [Logger]))
        .build::<YellowstoneGrpcSource>(config)
        .run();
}
