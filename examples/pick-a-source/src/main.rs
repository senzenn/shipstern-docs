use std::path::PathBuf;

use clap::Parser as _;
use shipstern::{
    config::BufferConfig,
    shipstern_core::{instruction::InstructionUpdate, AccountUpdate},
    sources::{FromConfig, SourceTrait},
    Pipeline,
};
use shipstern_solana_rpc_source::{SolanaAccountsRpcConfig, SolanaAccountsRpcSource};
use shipstern_spl_token_parser::InstructionParser as TokenInstructions;
use shipstern_stake_pool_parser::AccountParser as StakePoolParser;
use shipstern_yellowstone_fumarole_source::{FumaroleConfig, YellowstoneFumaroleSource};
use shipstern_yellowstone_grpc_source::{YellowstoneGrpcConfig, YellowstoneGrpcSource};

#[derive(clap::Parser)]
#[command(version, author, about)]
pub struct Opts {
    #[arg(long, short)]
    config: PathBuf,
}

/// The config file: the same `[source]` and `[buffer]` sections as `ShipsternConfig`.
#[derive(serde::Deserialize)]
struct Config {
    source: SourceConfig,
    #[serde(default)]
    buffer: BufferConfig,
}

/// `kind` in `[source]` picks the variant. The other keys are that source's config.
#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum SourceConfig {
    YellowstoneGrpc(YellowstoneGrpcConfig),
    Fumarole(FumaroleConfig),
    SolanaRpc(SolanaAccountsRpcConfig),
}

#[derive(Debug)]
pub struct Logger;

impl<V: Sync> shipstern::Handler<V, AccountUpdate> for Logger {
    async fn handle(&self, _value: &V, input: &AccountUpdate) -> shipstern::HandlerResult<()> {
        println!("account slot={} startup={}", input.slot, input.is_startup);
        Ok(())
    }
}

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

    // Errors only, unless RUST_LOG says otherwise.
    tracing_subscriber::fmt::init();

    let Opts { config } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let Config { source, buffer } = toml::from_str(&config)
        .unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    let (kind, source): (&str, Box<dyn SourceTrait>) = match source {
        SourceConfig::YellowstoneGrpc(config) => (
            "yellowstone-grpc",
            Box::new(YellowstoneGrpcSource::from_config(config)),
        ),
        SourceConfig::Fumarole(config) => (
            "fumarole",
            Box::new(YellowstoneFumaroleSource::from_config(config)),
        ),
        SourceConfig::SolanaRpc(config) => (
            "solana-rpc",
            Box::new(SolanaAccountsRpcSource::from_config(config)),
        ),
    };

    println!("source: {kind}");

    shipstern::Runtime::builder()
        .account(Pipeline::new(StakePoolParser, [Logger]))
        .instruction(Pipeline::new(TokenInstructions, [Logger]))
        .build_with(source, buffer)
        .run();
}
