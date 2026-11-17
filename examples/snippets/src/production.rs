use shipstern::{builder::BuilderError, config::ShipsternConfig, Pipeline, Runtime};
use shipstern_spl_token_parser::{AccountParser, InstructionParser};
use shipstern_yellowstone_grpc_source::{YellowstoneGrpcConfig, YellowstoneGrpcSource};

use crate::handlers::Logger;

pub fn run() {
    // #region secrets-from-env
    let config = std::fs::read_to_string("Shipstern.toml").expect("Error reading config file");
    let mut config: ShipsternConfig<YellowstoneGrpcConfig> =
        toml::from_str(&config).unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    // Keep the token out of the file; inject it at deploy time.
    config.source.x_token = std::env::var("SHIPSTERN_X_TOKEN").ok();

    // The stored slot provides a restart checkpoint.
    config.source.from_slot = std::env::var("SHIPSTERN_FROM_SLOT")
        .ok()
        .and_then(|slot| slot.parse().ok());
    // #endregion secrets-from-env

    shipstern::Runtime::builder()
        .instruction(Pipeline::new(InstructionParser, [Logger]))
        .build::<YellowstoneGrpcSource>(config)
        .run();
}

pub fn build_runtime(config: ShipsternConfig<YellowstoneGrpcConfig>) {
    // #region build-runtime
    let runtime = Runtime::builder()
        .account(Pipeline::new(AccountParser, [Logger]))
        .instruction(Pipeline::new(InstructionParser, [Logger]))
        .build::<YellowstoneGrpcSource>(config);
    // #endregion build-runtime

    runtime.run();
}

pub fn take_handle(config: ShipsternConfig<YellowstoneGrpcConfig>) -> Result<(), BuilderError> {
    // #region runtime-handle
    let runtime = Runtime::builder()
        .account(Pipeline::new(AccountParser, [Logger]))
        .try_build::<YellowstoneGrpcSource>(config)?;

    // Take the handle first: the run methods consume the runtime.
    let handle = runtime.handle();
    tokio::spawn(runtime.run_async());
    // #endregion runtime-handle

    drop(handle);
    Ok(())
}
