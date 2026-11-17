use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    str::FromStr as _,
    sync::{Arc, Mutex},
    time::Instant,
};

use shipstern::{
    bs58, config::ShipsternConfig, CommitmentLevel, Handler, HandlerResult, Pipeline, Runtime,
};
use shipstern_core::{
    ParseResult, Parser, Prefilter, Pubkey, TransactionPrefilter, TransactionUpdate,
};
use shipstern_preconf_source::{PreconfConfig, PreconfSource};
use shipstern_yellowstone_grpc_source::{YellowstoneGrpcConfig, YellowstoneGrpcSource};
use tracing::Instrument as _;

/// The Orca Whirlpool program, as in the upstream preconf example. The whole SPL Token
/// program reaches the feed's coverage share within a minute.
const PROGRAM: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
/// Confirmed slots to wait for a confirmed copy before a preconf counts as not settled.
const WINDOW_SLOTS: u64 = 150;
/// The most signatures held at once, settled ones included.
const MAX_TRACKED: usize = 100_000;

/// `Shipstern.toml`: one runtime config for each source.
#[derive(serde::Deserialize)]
struct Config {
    preconf: ShipsternConfig<PreconfConfig>,
    confirmed: ShipsternConfig<YellowstoneGrpcConfig>,
}

/// Reads the signature, slot and result of each transaction that names the program.
#[derive(Debug, Clone, Copy)]
struct Signatures {
    id: &'static str,
    program: Pubkey,
    /// `Some(false)` asks for successes only, `None` for both results.
    failed: Option<bool>,
}

#[derive(Debug)]
struct Seen {
    signature: [u8; 64],
    slot: u64,
    failed: bool,
}

impl Parser for Signatures {
    type Input = TransactionUpdate;
    type Output = Seen;

    fn id(&self) -> Cow<'static, str> {
        self.id.into()
    }

    fn prefilter(&self) -> Prefilter {
        let transaction = TransactionPrefilter {
            accounts_include: [self.program].into(),
            failed: self.failed,
            ..TransactionPrefilter::default()
        };
        Prefilter {
            transaction: Some(transaction),
            ..Prefilter::default()
        }
    }

    async fn parse(&self, update: &TransactionUpdate) -> ParseResult<Seen> {
        let info = update.transaction.as_ref().ok_or("no transaction")?;
        let signature = info.signature[..].try_into().map_err(|_| "bad signature")?;
        let failed = info.meta.as_ref().is_some_and(|meta| meta.err.is_some());
        let slot = update.slot;
        Ok(Seen {
            signature,
            slot,
            failed,
        })
    }
}

#[derive(Debug)]
enum Entry {
    /// Preconfirmed, with no confirmed copy yet.
    Pending { at: Instant, slot: u64 },
    /// Settled, or confirmed with no preconf. Later copies change nothing.
    Done,
}

/// One entry per signature, grouped by the slot it was first seen in.
#[derive(Debug, Default)]
struct Tracker {
    entries: HashMap<[u8; 64], Entry>,
    by_slot: BTreeMap<u64, Vec<[u8; 64]>>,
    first_confirmed: Option<u64>,
    confirmed: u64,
}

impl Tracker {
    /// The place to act on the early signal. This program prints it and records it.
    fn on_preconf(&mut self, seen: &Seen) {
        // Only the confirmed stream's slots after it started can settle a preconf.
        let live = self.first_confirmed.is_some_and(|first| seen.slot > first);
        let late = seen.slot.saturating_add(WINDOW_SLOTS) <= self.confirmed;
        if !live || late || self.entries.contains_key(&seen.signature) {
            return;
        }
        println!("preconf   slot={} {}", seen.slot, b58(&seen.signature));
        let (at, slot) = (Instant::now(), seen.slot);
        self.insert(seen, Entry::Pending { at, slot });
    }

    fn on_confirmed(&mut self, seen: &Seen) {
        self.first_confirmed.get_or_insert(seen.slot);
        self.confirmed = self.confirmed.max(seen.slot);
        match self.entries.get(&seen.signature) {
            Some(&Entry::Pending { at, slot }) => {
                let lead = at.elapsed().as_millis();
                let result = if seen.failed { "failed" } else { "ok" };
                let sig = b58(&seen.signature);
                println!(
                    "settled   slot={} preconf_slot={slot} lead_ms={lead} result={result} {sig}",
                    seen.slot
                );
                self.entries.insert(seen.signature, Entry::Done);
            }
            Some(Entry::Done) => {}
            None => self.insert(seen, Entry::Done),
        }
        self.expire();
    }

    fn insert(&mut self, seen: &Seen, entry: Entry) {
        self.entries.insert(seen.signature, entry);
        let group = self.by_slot.entry(seen.slot).or_default();
        group.push(seen.signature);
        self.expire();
    }

    /// Drops the oldest slot group once the confirmed stream is a window past it, or
    /// while over the cap. A pending entry there is reported as unsettled or untracked.
    fn expire(&mut self) {
        while let Some(oldest) = self.by_slot.first_entry() {
            let passed = oldest.key().saturating_add(WINDOW_SLOTS) <= self.confirmed;
            if !passed && self.entries.len() <= MAX_TRACKED {
                break;
            }
            let outcome = if passed { "unsettled" } else { "untracked" };
            for signature in oldest.remove() {
                if let Some(Entry::Pending { slot, .. }) = self.entries.remove(&signature) {
                    println!("{outcome} preconf_slot={slot} {}", b58(&signature));
                }
            }
        }
    }
}

fn b58(signature: &[u8; 64]) -> String {
    bs58::encode(signature).into_string()
}

/// Passes each transaction of one runtime to one `Tracker` method.
#[derive(Debug)]
struct Track(Arc<Mutex<Tracker>>, fn(&mut Tracker, &Seen));

impl Handler<Seen, TransactionUpdate> for Track {
    async fn handle(&self, seen: &Seen, _raw: &TransactionUpdate) -> HandlerResult<()> {
        let mut tracker = self.0.lock().map_err(|_| "tracker lock poisoned")?;
        (self.1)(&mut tracker, seen);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let text = std::fs::read_to_string("Shipstern.toml").expect("Error reading Shipstern.toml");
    // The error's Debug output holds the whole file, tokens included, so print only its message.
    let config: Config =
        toml::from_str(&text).unwrap_or_else(|e| panic!("Error parsing config: {}", e.message()));
    let mut confirmed = config.confirmed;
    confirmed.source.commitment_level = Some(CommitmentLevel::Confirmed);
    let program = Pubkey::from_str(PROGRAM).expect("Error parsing PROGRAM");

    let tracker = Arc::new(Mutex::new(Tracker::default()));
    let signatures = |id, failed| Signatures {
        id,
        program,
        failed,
    };

    let preconf = Runtime::builder()
        .transaction(Pipeline::new(
            signatures("settlement::preconf", Some(false)),
            [Track(Arc::clone(&tracker), Tracker::on_preconf)],
        ))
        .build::<PreconfSource>(config.preconf);
    let confirmed = Runtime::builder()
        .transaction(Pipeline::new(
            signatures("settlement::confirmed", None),
            [Track(tracker, Tracker::on_confirmed)],
        ))
        .build::<YellowstoneGrpcSource>(confirmed);

    // Ctrl+C reaches both runtimes. The first one to stop ends the program; a fatal error
    // exits inside run_async, and the span names the runtime on that log line.
    tokio::select! {
        () = preconf.run_async().instrument(tracing::error_span!("preconf")) => {},
        () = confirmed.run_async().instrument(tracing::error_span!("confirmed")) => {},
    }
}
