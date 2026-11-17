use std::path::PathBuf;

use clap::Parser as _;
use shipstern::{shipstern_core::instruction::InstructionUpdate, Pipeline};
use shipstern_proc_macro::include_shipstern_parser;
use shipstern_yellowstone_grpc_source::YellowstoneGrpcSource;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Generates the `pump` module from the Anchor IDL in the crate root.
include_shipstern_parser!("pump.json");

use pump::event::Event;

/// 1 SOL in lamports. Smaller trades are not printed.
const MIN_LAMPORTS: u64 = 1_000_000_000;

#[derive(clap::Parser)]
#[command(version, author, about)]
pub struct Opts {
    #[arg(long, short)]
    config: PathBuf,
}

#[derive(Debug)]
pub struct TradeFeed;

impl shipstern::Handler<pump::ProgramEventOutput, InstructionUpdate> for TradeFeed {
    async fn handle(
        &self,
        value: &pump::ProgramEventOutput,
        _raw: &InstructionUpdate,
    ) -> shipstern::HandlerResult<()> {
        for ev in &value.program_events {
            match &ev.event {
                Event::TradeEvent { args, .. } if args.sol_amount >= MIN_LAMPORTS => {
                    let side = if args.is_buy { "buy " } else { "sell" };
                    let sol = args.sol_amount as f64 / 1e9;

                    println!("{side} {sol:>8.2} SOL  {}", args.mint);
                }
                Event::CreateEvent { args, .. } => {
                    println!("new  {} ({})  {}", args.name, args.symbol, args.mint);
                }
                _ => {}
            }
        }

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
        .instruction(Pipeline::new(pump::InstructionParser, [TradeFeed]))
        .build::<YellowstoneGrpcSource>(config)
        .run();
}
