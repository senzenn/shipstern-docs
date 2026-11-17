// #region capture-config
use std::path::PathBuf;

use shipstern_block_coordinator::{CoordinatorInput, CoordinatorSourceConfig};
use shipstern_yellowstone_grpc_source::YellowstoneGrpcConfig;
use tokio::sync::mpsc::Sender;

pub fn capture_config(
    source: YellowstoneGrpcConfig,
    coordinator_input_tx: Sender<CoordinatorInput>,
    fixture_path: PathBuf,
    block_meta_limit: usize,
) -> CoordinatorSourceConfig {
    CoordinatorSourceConfig {
        source,
        source_label: None,
        stream_idle_warn_secs: 0,
        coordinator_input_tx: Some(coordinator_input_tx),
        fixture_path: Some(fixture_path),
        fixture_slots: Some(block_meta_limit),
    }
}
// #endregion capture-config

// #region fixture-io
use std::{io, path::Path};

use shipstern_block_coordinator::{FixtureReader, FixtureWriter};
use yellowstone_grpc_proto::geyser::SubscribeUpdate;

pub fn write_fixture(
    path: &Path,
    updates: &[SubscribeUpdate],
    block_meta_limit: usize,
) -> io::Result<()> {
    let mut writer = FixtureWriter::new(path, block_meta_limit)?;
    for update in updates {
        if !writer.write(update)? {
            break;
        }
    }
    writer.finish()
}

pub fn read_fixture(path: &Path) -> io::Result<Vec<SubscribeUpdate>> {
    Ok(FixtureReader::new(path)?.collect())
}
// #endregion fixture-io

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::SystemTime};
    use yellowstone_grpc_proto::{
        geyser::{
            subscribe_update::UpdateOneof, SlotStatus, SubscribeUpdateBlockMeta,
            SubscribeUpdateSlot, SubscribeUpdateTransaction, SubscribeUpdateTransactionInfo,
        },
        solana::storage::confirmed_block::{
            Message, MessageHeader, TokenBalance, Transaction, TransactionStatusMeta,
        },
    };

    fn fixture_path(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "shipstern-{name}-{}-{nonce}.bin",
            std::process::id()
        ))
    }

    fn synthetic_updates() -> Vec<SubscribeUpdate> {
        let slot = SubscribeUpdate {
            filters: vec!["coordinator".into()],
            update_oneof: Some(UpdateOneof::Slot(SubscribeUpdateSlot {
                slot: 42,
                parent: Some(41),
                status: SlotStatus::SlotConfirmed as i32,
                ..Default::default()
            })),
            ..Default::default()
        };
        let transaction = SubscribeUpdate {
            filters: vec!["token-transactions".into()],
            update_oneof: Some(UpdateOneof::Transaction(SubscribeUpdateTransaction {
                slot: 42,
                transaction: Some(SubscribeUpdateTransactionInfo {
                    signature: vec![9; 64],
                    index: 3,
                    transaction: Some(Transaction {
                        signatures: vec![vec![9; 64]],
                        message: Some(Message {
                            header: Some(MessageHeader::default()),
                            account_keys: vec![vec![1; 32]],
                            recent_blockhash: vec![2; 32],
                            ..Default::default()
                        }),
                    }),
                    meta: Some(TransactionStatusMeta {
                        fee: 5_000,
                        pre_balances: vec![100_000],
                        post_balances: vec![95_000],
                        post_token_balances: vec![TokenBalance {
                            account_index: 0,
                            mint: "example-mint".into(),
                            owner: "example-owner".into(),
                            ..Default::default()
                        }],
                        log_messages: vec!["Program log: fixture context".into()],
                        compute_units_consumed: Some(123),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            })),
            ..Default::default()
        };
        let block_meta = SubscribeUpdate {
            update_oneof: Some(UpdateOneof::BlockMeta(SubscribeUpdateBlockMeta {
                slot: 42,
                ..Default::default()
            })),
            ..Default::default()
        };
        vec![
            slot.clone(),
            transaction,
            block_meta.clone(),
            block_meta,
            slot,
        ]
    }

    // #region round-trip
    #[test]
    fn raw_context_round_trip_stops_at_the_block_meta_limit() -> io::Result<()> {
        let path = fixture_path("round-trip");
        let updates = synthetic_updates();
        write_fixture(&path, &updates, 2)?;
        let replay = read_fixture(&path)?;
        assert_eq!(replay, updates[..4]);
        fs::remove_file(path)?;
        Ok(())
    }
    // #endregion round-trip

    #[test]
    fn a_truncated_tail_returns_only_the_decodable_prefix() -> io::Result<()> {
        let path = fixture_path("truncated");
        let update = SubscribeUpdate {
            filters: vec!["example".into()],
            ..Default::default()
        };
        write_fixture(&path, &[update.clone()], usize::MAX)?;
        let complete = fs::read(&path)?;
        for tail in [vec![0, 1], vec![0, 0, 0, 2, 0xff], vec![0, 0, 0, 1, 0xff]] {
            let mut bytes = complete.clone();
            bytes.extend(tail);
            fs::write(&path, bytes)?;
            assert_eq!(read_fixture(&path)?, vec![update.clone()]);
        }
        fs::remove_file(path)?;
        Ok(())
    }
}
