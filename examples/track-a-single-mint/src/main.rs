use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use shipstern::{
    filter_pipeline::FilterPipeline,
    shipstern_core::{instruction::InstructionUpdate, KeyBytes, Prefilter},
};
use shipstern_spl_token_parser::InstructionParser;
use shipstern_yellowstone_grpc_source::YellowstoneGrpcSource;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// A mint key narrows the server's transaction subscription.
const MINT: &str = "So11111111111111111111111111111111111111112";

#[derive(clap::Parser)]
#[command(version, author, about)]
pub struct Opts {
    #[arg(long, short)]
    config: PathBuf,
}

#[derive(Debug)]
pub struct Logger;

impl<V: std::fmt::Debug + Sync> shipstern::Handler<V, InstructionUpdate> for Logger {
    async fn handle(&self, value: &V, input: &InstructionUpdate) -> shipstern::HandlerResult<()> {
        println!("ix {:?} - {value:?}", input.path);
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

    let mint = KeyBytes::<32>::from_str(MINT).expect("invalid mint address");

    shipstern::Runtime::builder()
        .instruction(FilterPipeline::new(
            InstructionParser,
            [Logger],
            Prefilter::builder().transaction_accounts_include([mint]),
        ))
        .build::<YellowstoneGrpcSource>(config)
        .run();
}
