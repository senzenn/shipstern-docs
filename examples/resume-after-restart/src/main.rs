use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use clap::Parser as _;
use shipstern::{
    bs58,
    config::ShipsternConfig,
    shipstern_core::{
        instruction::InstructionUpdate, ParseResult, Parser, Prefilter, SlotPrefilter, SlotUpdate,
    },
    CommitmentLevel, Handler, HandlerResult, Pipeline, Runtime,
};
use shipstern_spl_token_parser::{instruction::Instruction, InstructionParser, TokenProgram};
use shipstern_yellowstone_grpc_source::{YellowstoneGrpcConfig, YellowstoneGrpcSource};

/// Holds one slot: the transfers of every confirmed slot up to it are written.
const CHECKPOINT: &str = "checkpoint";
/// One file per slot with transfers, named after the slot.
const OUT_DIR: &str = "transfers";
/// How many written slots can wait for a parent's status before a warning.
const WAITING_WARNING: usize = 32;

#[derive(clap::Parser)]
struct Opts {
    #[arg(long, short)]
    config: PathBuf,
}

#[derive(Debug, Default)]
struct State {
    /// The saved checkpoint. `None` until the first confirmed status of a run from the tip.
    checkpoint: Option<u64>,
    /// Transfer lines of the slots above the checkpoint that have no file yet.
    pending: BTreeMap<u64, BTreeSet<String>>,
    /// Slots above the checkpoint that have their file, with the parent each one named.
    written: BTreeMap<u64, u64>,
}

impl State {
    /// A slot at or below the checkpoint, or written in this run, takes no more updates.
    fn is_done(&self, slot: u64) -> bool {
        self.checkpoint.is_some_and(|c| slot <= c) || self.written.contains_key(&slot)
    }

    /// Record a written slot, then move the checkpoint over each written slot whose parent
    /// is at or below it. A slot whose parent has no file yet waits. Returns a new checkpoint.
    fn complete(&mut self, slot: u64, parent: u64) -> Option<u64> {
        self.written.insert(slot, parent);
        let old = self.checkpoint?;
        let mut new = old;
        while let Some(entry) = self.written.first_entry() {
            if *entry.get() > new {
                break;
            }
            new = entry.remove_entry().0;
        }
        (new != old).then(|| self.advance_to(new))
    }

    fn advance_to(&mut self, slot: u64) -> u64 {
        self.checkpoint = Some(slot);
        self.pending.retain(|s, _| *s > slot);
        slot
    }
}

/// Buffers each SPL Token transfer under its slot. The set drops a repeated line.
#[derive(Debug)]
struct Collect(Arc<Mutex<State>>);

impl Handler<TokenProgram, InstructionUpdate> for Collect {
    async fn handle(&self, value: &TokenProgram, raw: &InstructionUpdate) -> HandlerResult<()> {
        let (source, destination, amount) = match value.instruction.as_ref() {
            Some(Instruction::Transfer(t)) => {
                (t.accounts.source, t.accounts.destination, t.args.amount)
            }
            Some(Instruction::TransferChecked(t)) => {
                (t.accounts.source, t.accounts.destination, t.args.amount)
            }
            _ => return Ok(()),
        };
        let signature = bs58::encode(&raw.shared.signature).into_string();
        let line = format!("{signature} {:?} {source} {destination} {amount}", raw.path);

        let mut state = self.0.lock().map_err(|e| e.to_string())?;
        if !state.is_done(raw.shared.slot) {
            state
                .pending
                .entry(raw.shared.slot)
                .or_default()
                .insert(line);
        }
        Ok(())
    }
}

/// Passes on slot statuses. The default slot prefilter asks only for statuses at the
/// stream's commitment, which `main` sets to confirmed.
#[derive(Debug, Clone, Copy)]
struct ConfirmedSlots;

impl Parser for ConfirmedSlots {
    type Input = SlotUpdate;
    type Output = SlotUpdate;

    fn id(&self) -> Cow<'static, str> {
        "resume_after_restart::ConfirmedSlots".into()
    }

    fn prefilter(&self) -> Prefilter {
        Prefilter {
            slot: Some(SlotPrefilter::default()),
            ..Prefilter::default()
        }
    }

    async fn parse(&self, update: &SlotUpdate) -> ParseResult<SlotUpdate> {
        Ok(update.clone())
    }
}

/// Writes a slot's file when its confirmed status arrives, then saves the checkpoint.
#[derive(Debug)]
struct Commit(Arc<Mutex<State>>);

impl Handler<SlotUpdate, SlotUpdate> for Commit {
    async fn handle(&self, update: &SlotUpdate, _raw: &SlotUpdate) -> HandlerResult<()> {
        let slot = update.slot;
        let mut state = self.0.lock().map_err(|e| e.to_string())?;
        if state.is_done(slot) {
            return Ok(());
        }
        // A run from the tip can join inside this slot, so the slot only marks the start.
        if state.checkpoint.is_none() {
            write_atomic(Path::new(CHECKPOINT), &slot.to_string())?;
            state.advance_to(slot);
            return Ok(());
        }
        let parent = update.parent.ok_or("the status names no parent slot")?;

        if let Some(lines) = state.pending.get(&slot) {
            let text: String = lines.iter().map(|line| format!("{line}\n")).collect();
            write_atomic(&Path::new(OUT_DIR).join(format!("{slot}.txt")), &text)?;
        }
        state.pending.remove(&slot);

        if let Some(checkpoint) = state.complete(slot, parent) {
            write_atomic(Path::new(CHECKPOINT), &checkpoint.to_string())?;
        } else if state.written.len() == WAITING_WARNING {
            tracing::warn!(checkpoint = ?state.checkpoint, "Written slots wait for a parent's status");
        }
        Ok(())
    }
}

/// Replaces `path` in one step, so after a crash it holds the old text or the new text.
fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}

fn read_checkpoint() -> Result<Option<u64>, String> {
    match std::fs::read_to_string(CHECKPOINT) {
        Ok(text) => text
            .trim()
            .parse()
            .map(Some)
            .map_err(|e| format!("Bad {CHECKPOINT}: {e}")),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("Error reading {CHECKPOINT}: {e}")),
    }
}

fn start() -> Result<(), String> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| "Failed to install rustls crypto provider")?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let Opts { config } = Opts::parse();
    let text = std::fs::read_to_string(config).map_err(|e| format!("Error reading config: {e}"))?;
    // Only the message: Display shows the failing line and Debug the whole file, x-token included.
    let mut config: ShipsternConfig<YellowstoneGrpcConfig> = toml::from_str(&text)
        .map_err(|e| format!("Error parsing config: {}", e.message()))?;

    // The checkpoint depends on both: statuses after content, and one update at a time.
    config.source.commitment_level = Some(CommitmentLevel::Confirmed);
    config.buffer.jobs = Some(1);

    let checkpoint = read_checkpoint()?;
    if let Some(slot) = checkpoint {
        config.source.from_slot = Some(slot.saturating_add(1));
    }
    tracing::info!(?checkpoint, from_slot = ?config.source.from_slot, "Starting");

    std::fs::create_dir_all(OUT_DIR).map_err(|e| format!("Error creating {OUT_DIR}: {e}"))?;

    // The slot before the first one to replay counts as done.
    let state = Arc::new(Mutex::new(State {
        checkpoint: config.source.from_slot.map(|slot| slot.saturating_sub(1)),
        ..State::default()
    }));

    Runtime::builder()
        .instruction(Pipeline::new(
            InstructionParser,
            [Collect(Arc::clone(&state))],
        ))
        .slot(Pipeline::new(ConfirmedSlots, [Commit(state)]))
        .build::<YellowstoneGrpcSource>(config)
        .run();

    Ok(())
}

fn main() {
    if let Err(err) = start() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
