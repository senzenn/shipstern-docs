use shipstern::{filter_pipeline::FilterPipeline, instruction::InstructionPipeline, Pipeline, Runtime};
use shipstern_core::{
    AccountFilter, CuckooAccounts, Parser, Prefilter, PrefilterError, Pubkey, TokenAccounts,
};
use shipstern_spl_token_parser::{AccountParser, InstructionParser};
use shipstern_yellowstone_grpc_source::{YellowstoneGrpcConfig, YellowstoneGrpcSource};

use crate::handlers::Logger;

pub fn token_accounts_only() -> Result<Prefilter, PrefilterError> {
    // #region build-prefilter
    let token_program: Pubkey = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse().unwrap();

    // Token accounts are exactly 165 bytes; mints are 82.
    let prefilter = Prefilter::builder()
        .account_owners([token_program])
        .account_filters([AccountFilter::DataSize(165)])
        .build()?;
    // #endregion build-prefilter

    Ok(prefilter)
}

pub fn combined_accounts(
    token_program: Pubkey,
    first: Pubkey,
    second: Pubkey,
) -> Result<Prefilter, PrefilterError> {
    // #region combine-account-filters
    let prefilter = Prefilter::builder()
        .account_owners([token_program])
        .accounts([first, second])
        .account_filters([AccountFilter::DataSize(165)])
        .build()?;
    // #endregion combine-account-filters

    Ok(prefilter)
}

pub fn combined_transactions(
    token_program: Pubkey,
    first: Pubkey,
    second: Pubkey,
    excluded: Pubkey,
) -> Result<Prefilter, PrefilterError> {
    // #region combine-transaction-filters
    let prefilter = Prefilter::builder()
        .transaction_accounts([token_program])
        .transaction_accounts_include([first, second])
        .transaction_accounts_exclude([excluded])
        .transaction_vote(false)
        .build()?;
    // #endregion combine-transaction-filters

    Ok(prefilter)
}

pub fn runtime_filters(
    config: shipstern::config::ShipsternConfig<YellowstoneGrpcConfig>,
    account_keys: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    // #region runtime-filters
    let keys = account_keys
        .iter()
        .map(|key| key.parse::<Pubkey>())
        .collect::<Result<Vec<_>, _>>()?;

    // An empty include set would leave all Token Program transactions eligible.
    if keys.is_empty() {
        return Err("Provide at least one account key".into());
    }

    let pipeline = FilterPipeline::new(
        InstructionParser,
        [Logger],
        Prefilter::builder().transaction_accounts_include(keys),
    );

    Runtime::builder()
        .instruction(pipeline)
        .try_build::<YellowstoneGrpcSource>(config)?
        .run();
    // #endregion runtime-filters

    Ok(())
}

pub fn large_account_set(
    pools: Vec<Pubkey>,
    key: Pubkey,
) -> Result<(Prefilter, bool), PrefilterError> {
    // #region cuckoo
    // Thousands of keys go out as one compact filter instead of a key list.
    let prefilter = Prefilter::builder()
        .account_cuckoo(pools.iter().copied())
        .build()?;

    // False matches require a local membership check.
    let pools: CuckooAccounts = pools.into_iter().collect();
    let wanted = pools.contains(&key);
    // #endregion cuckoo

    Ok((prefilter, wanted))
}

pub fn wallet_transfers(wallet: Pubkey) -> Result<Prefilter, PrefilterError> {
    // #region token-accounts
    // Also match a transaction when a token account the wallet owns changes balance or closes.
    let prefilter = Prefilter::builder()
        .transaction_accounts_include([wallet])
        .transaction_token_accounts(TokenAccounts::BalanceChanged)
        .build()?;
    // #endregion token-accounts

    Ok(prefilter)
}

pub async fn live_updates(
    config: shipstern::config::ShipsternConfig<YellowstoneGrpcConfig>,
    new_owner: Pubkey,
    extra_for_instructions: Prefilter,
) -> Result<(), Box<dyn std::error::Error>> {
    // #region live-updates
    let runtime = Runtime::builder()
        .account(Pipeline::new(AccountParser, [Logger]))
        .instruction(Pipeline::new(InstructionParser, [Logger]))
        .try_build::<YellowstoneGrpcSource>(config)?; // BuilderError is not Send + Sync: see the note below.

    // Take the handle first: run and run_async consume the runtime.
    let handle = runtime.handle();
    tokio::spawn(runtime.run_async());

    // Widen the account subscription with one more owner.
    let extra = Prefilter::builder().account_owners([new_owner]).build()?;
    handle.update_filters(|filters| filters.merge(AccountParser.id(), extra))?;
    // #endregion live-updates

    // #region live-updates-instructions
    // All instruction parsers share one entry.
    handle.update_filters(|filters| {
        filters.merge(InstructionPipeline::ID, extra_for_instructions)
    })?;
    // #endregion live-updates-instructions

    // #region live-updates-reset
    let current = handle.filters();
    println!("live parser IDs: {:?}", current.parser_ids().collect::<Vec<_>>());

    // Back to the filters the runtime started with.
    handle.reset_filters()?;
    // #endregion live-updates-reset

    Ok(())
}
