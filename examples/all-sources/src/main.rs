use std::path::PathBuf;

use clap::Parser as _;
use shipstern::{
    config::BufferConfig,
    sources::{FromConfig, SourceTrait},
    Pipeline,
};
use shipstern_jetstream_source::{JetstreamSource, JetstreamSourceConfig};
use shipstern_solana_rpc_source::{SolanaAccountsRpcConfig, SolanaAccountsRpcSource};
use shipstern_solana_snapshot_source::{SolanaSnapshotConfig, SolanaSnapshotSource};
use shipstern_spl_token_parser::InstructionParser as TokenInstructions;
use shipstern_stake_pool_parser::AccountParser as StakePoolParser;
use shipstern_yellowstone_fumarole_source::{FumaroleConfig, YellowstoneFumaroleSource};
use shipstern_yellowstone_grpc_source::{YellowstoneGrpcConfig, YellowstoneGrpcSource};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    SolanaSnapshot(SolanaSnapshotConfig),
    Jetstreamer(JetstreamSourceConfig),
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
    let Config { source, buffer } = toml::from_str(&config)
        .unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    let source: Box<dyn SourceTrait> = match source {
        SourceConfig::YellowstoneGrpc(config) => {
            Box::new(YellowstoneGrpcSource::from_config(config))
        },
        SourceConfig::Fumarole(config) => Box::new(YellowstoneFumaroleSource::from_config(config)),
        SourceConfig::SolanaRpc(config) => Box::new(SolanaAccountsRpcSource::from_config(config)),
        SourceConfig::SolanaSnapshot(config) => Box::new(SolanaSnapshotSource::from_config(config)),
        SourceConfig::Jetstreamer(config) => {
            // SAFETY: no other thread exists yet. `run()` creates the Tokio runtime after this.
            unsafe { shipstern_jetstream_source::init_process_env(&config) };
            Box::new(JetstreamSource::from_config(config))
        },
    };

    // Stake pool accounts: the RPC and snapshot sources can load them. Token
    // instructions: the streaming sources and Jetstreamer send them.
    shipstern::Runtime::builder()
        .account(Pipeline::new(StakePoolParser, [Logger]))
        .instruction(Pipeline::new(TokenInstructions, [Logger]))
        .build_with(source, buffer)
        .run();
}
