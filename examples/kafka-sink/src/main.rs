use std::sync::Arc;

use shipstern::{
    config::{BufferConfig, ShipsternConfig},
    Pipeline, Runtime,
};
use shipstern_block_coordinator::{
    AccountCommitAt, BlockMachineCoordinator, CoordinatorHandle, CoordinatorSource,
    CoordinatorSourceConfig, DEFAULT_MAX_PENDING_BANKS,
};
use shipstern_kafka_sink::{
    create_producer, initialize_transactional_producer, read_last_committed_transaction_block,
    BufferingHandler, KafkaSinkBuilder, KafkaSinkConfig, TransactionSlotSink,
    TransactionSubscription,
};
use shipstern_spl_token_parser::InstructionParser;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// `Shipstern.toml`: a `[kafka]` section plus the usual `[source]` and `[buffer]`.
#[derive(serde::Deserialize)]
struct AppConfig {
    kafka: KafkaSinkConfig,
    source: CoordinatorSourceConfig,
    #[serde(default)]
    buffer: BufferConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = std::fs::read_to_string("Shipstern.toml").expect("Error reading config file");
    let AppConfig {
        kafka,
        mut source,
        buffer,
    } = toml::from_str(&config).unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));

    // Fence the previous writer before reading its visible checkpoint.
    let producer = Arc::new(create_producer(&kafka));
    initialize_transactional_producer(&kafka, &producer).map_err(std::io::Error::other)?;

    if let Some(last) =
        read_last_committed_transaction_block(&kafka).map_err(std::io::Error::other)?
    {
        let from = last
            .slot
            .checked_add(1)
            .ok_or_else(|| std::io::Error::other("Last committed slot cannot be incremented"))?;
        source.source.from_slot = Some(from);
        tracing::info!(
            last_committed_slot = last.slot,
            from,
            "Resuming Kafka pipeline"
        );
    }

    // 1. Channels between the stages.
    let (input_tx, input_rx) = mpsc::channel(256); // source -> coordinator: slot lifecycle
    let (parsed_tx, parsed_rx) = mpsc::channel(256); // handler -> coordinator: parsed records
    let (slots_tx, slots_rx) = mpsc::channel(64); // coordinator -> Kafka: complete slots

    // 2. The coordinator: orders records by slot and releases a slot once it is complete.
    let coordinator = tokio::spawn(BlockMachineCoordinator::run(
        input_rx,
        parsed_rx,
        Some(slots_tx),
        None, // no account output in this example
        AccountCommitAt::Confirmed,
        true, // wait until every transaction in the slot has been parsed
        DEFAULT_MAX_PENDING_BANKS, // stop if more than 64 banks wait to flush
    ));

    // 3. The Kafka writer: one transaction per slot, then a marker on `transaction.slots`.
    let writer =
        tokio::spawn(TransactionSlotSink::new_preinitialized(kafka, producer).run(slots_rx));

    // 4. What to decode and where each program's records go.
    let sink = KafkaSinkBuilder::new()
        .instruction_parser(InstructionParser, "spl-token", "spl-token.instructions")
        .build();

    // 5. The Shipstern runtime, reading through the coordinator's source.
    source.coordinator_input_tx = Some(input_tx);
    let cancel = CancellationToken::new();
    let handler = BufferingHandler::new(sink, CoordinatorHandle::new(parsed_tx), cancel.clone());

    let runtime = Runtime::builder()
        .transaction(Pipeline::new(TransactionSubscription, [handler]))
        .build::<CoordinatorSource>(ShipsternConfig { source, buffer });

    // Only the runtime can stop cleanly (on a signal); any other stage ending is a failure.
    tokio::select! {
        result = runtime.try_run_async() => result.map_err(Into::into),
        result = coordinator => Err(format!("coordinator stopped: {result:?}").into()),
        result = writer => Err(format!("Kafka writer stopped: {result:?}").into()),
        () = cancel.cancelled() => Err("pipeline cancelled".into()),
    }
}
