use std::path::PathBuf;

use clap::Parser;
use shipstern::{config::ShipsternConfig, Pipeline};
use shipstern_jetstream_source::{JetstreamSource, JetstreamSourceConfig};
use shipstern_spl_token_parser::InstructionParser;
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

// Replays the configured slot range, logs each SPL Token instruction, then exits.
// A plain `fn main`, because the environment must be set before Tokio starts.
fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let Opts { config } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let config: ShipsternConfig<JetstreamSourceConfig> =
        toml::from_str(&config).unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    // SAFETY: no other thread exists yet. `run()` creates the Tokio runtime after this.
    unsafe { shipstern_jetstream_source::init_process_env(&config.source) };

    shipstern::Runtime::builder()
        .instruction(Pipeline::new(InstructionParser, [Logger]))
        .build::<JetstreamSource>(config)
        .run();
}
