use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use shipstern::{
    config::BufferConfig,
    shipstern_core::{AccountUpdate, Pubkey},
    sources::FromConfig,
    CommitmentLevel, Handler, HandlerResult, Pipeline, Runtime,
};
use shipstern_solana_rpc_source::{SolanaAccountsRpcConfig, SolanaAccountsRpcSource};
use shipstern_stake_pool_parser::{
    spl_stake_pool_program_state::State, AccountParser, SplStakePoolProgramState,
};
use shipstern_yellowstone_grpc_source::{YellowstoneGrpcConfig, YellowstoneGrpcSource};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;

/// `Shipstern.toml`: the quickstart's `[source]` for the stream, plus `[rpc]` for the backfill.
#[derive(serde::Deserialize)]
struct Config {
    source: YellowstoneGrpcConfig,
    rpc: SolanaAccountsRpcConfig,
    #[serde(default)]
    buffer: BufferConfig,
}

/// Orders two states of one account: slot, then live over RPC, then write version.
type Version = (u64, bool, u64);

/// The newest state of each Stake Pool account, from either source.
#[derive(Debug, Default)]
struct Store {
    accounts: HashMap<Pubkey, (Version, SplStakePoolProgramState)>,
    /// The lowest slot on an RPC state, checked after the backfill.
    rpc_lowest: Option<u64>,
}

/// Merges the account states of one source into the shared store.
#[derive(Debug)]
struct Merge {
    store: Arc<Mutex<Store>>,
    live: bool,
}

/// One Stake Pool account pipeline per runtime, both writing to `store`.
fn merge(store: &Arc<Mutex<Store>>, live: bool) -> Pipeline<AccountParser, [Merge; 1]> {
    let store = Arc::clone(store);
    Pipeline::new(AccountParser, [Merge { store, live }])
}

impl Handler<SplStakePoolProgramState, AccountUpdate> for Merge {
    async fn handle(
        &self,
        state: &SplStakePoolProgramState,
        raw: &AccountUpdate,
    ) -> HandlerResult<()> {
        let account = raw.account.as_ref().ok_or("update has no account")?;
        let pubkey = Pubkey::try_from(account.pubkey.as_slice())?;
        let version = (raw.slot, self.live, account.write_version);

        {
            let mut store = self.store.lock().map_err(|_| "store lock poisoned")?;
            if !self.live {
                store.rpc_lowest = Some(store.rpc_lowest.map_or(raw.slot, |low| low.min(raw.slot)));
            }
            // Keep the higher version, so arrival order and repeats do not change the result.
            if matches!(store.accounts.get(&pubkey), Some((stored, _)) if *stored >= version) {
                return Ok(());
            }
            store.accounts.insert(pubkey, (version, state.clone()));
        }

        let kind = match state.state {
            Some(State::StakePool(_)) => "stake pool",
            Some(State::ValidatorList(_)) => "validator list",
            None => "no state",
        };
        let from = if self.live { "live" } else { "rpc" };
        println!("{from} {pubkey} slot={} {kind}", raw.slot);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| "a rustls crypto provider is already installed")?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let text = std::fs::read_to_string("Shipstern.toml")?;
    // Only the message: the error's `Debug` form holds the whole file, x-token included.
    let mut config: Config =
        toml::from_str(&text).map_err(|e| format!("Shipstern.toml: {}", e.message()))?;
    // Both sources read confirmed state, so that their slots compare.
    config.source.commitment_level = Some(CommitmentLevel::Confirmed);
    config.rpc.commitment_level = Some(CommitmentLevel::Confirmed);

    // 1. Read a recent confirmed slot from the RPC node. The stream starts at this slot.
    let start = RpcClient::new_with_timeout_and_commitment(
        config.rpc.endpoint.clone(),
        Duration::from_secs(config.rpc.timeout),
        CommitmentConfig::confirmed(),
    )
    .get_slot()
    .await?;
    config.source.from_slot = Some(start);
    println!("stream starts at slot {start}");

    let store = Arc::new(Mutex::new(Store::default()));

    // 2. The server replays every stored slot from `start`, then sends each new slot.
    let live = Runtime::builder()
        .account(merge(&store, true))
        .try_build_with(
            YellowstoneGrpcSource::from_config(config.source),
            config.buffer,
        )?;
    let mut live = tokio::spawn(live.try_run_async());

    // 3. Load every account. A stream that fails ends the program without waiting for the load.
    let load = Runtime::builder()
        .account(merge(&store, false))
        .try_build_with(
            SolanaAccountsRpcSource::from_config(config.rpc),
            config.buffer,
        )?;
    tokio::select! {
        biased;
        ended = &mut live => {
            // `Ok` here means that the stream stopped without an error, as after a signal.
            ended??;
            return Ok(());
        },
        loaded = load.try_run_async() => loaded?,
    }

    // 4. The stream has every write from `start` on, so no RPC state can be older.
    let (count, lowest) = {
        let store = store.lock().map_err(|_| "store lock poisoned")?;
        (store.accounts.len(), store.rpc_lowest)
    };
    match lowest {
        Some(slot) if slot >= start => {
            println!("backfill ended: {count} accounts stored, RPC slot {slot}, start {start}");
        }
        Some(slot) => {
            return Err(format!("RPC slot {slot} is below the start {start}; run again").into());
        }
        None => return Err("the backfill loaded no accounts, so its slot is unchecked".into()),
    }

    // 5. The stream keeps the store current until a signal or a stream error.
    live.await??;
    Ok(())
}
