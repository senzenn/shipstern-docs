use std::{path::PathBuf, time::Duration};

use clap::Parser;
use shipstern::Pipeline;
use shipstern_spl_token_parser::{AccountParser, InstructionParser};
use shipstern_yellowstone_grpc_source::YellowstoneGrpcSource;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const PUSHGATEWAY: &str = "http://localhost:9091";

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
        println!("{value:?}");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Without a subscriber, push failures and fatal runtime errors print nothing.
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let Opts { config } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let config = toml::from_str(&config)
        .unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    // One copy goes to the runtime, the other to the push loop.
    let registry = prometheus::Registry::new();
    let push_registry = registry.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;
            let metrics = push_registry.gather();

            // push_metrics blocks, so keep it off the async workers.
            let _ = tokio::task::spawn_blocking(move || {
                if let Err(e) = prometheus::push_metrics(
                    "shipstern",
                    prometheus::labels! {},
                    PUSHGATEWAY,
                    metrics,
                    None,
                ) {
                    tracing::error!("Failed to push metrics: {e:?}");
                }
            })
            .await;
        }
    });

    shipstern::Runtime::builder()
        .instruction(Pipeline::new(InstructionParser, [Logger]))
        .account(Pipeline::new(AccountParser, [Logger]))
        .metrics(registry)
        .build::<YellowstoneGrpcSource>(config)
        .run_async()
        .await;
}
