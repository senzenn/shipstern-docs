// #region account-fixture
use shipstern_core::Parser;
use shipstern_mock::{account_fixture, tx_fixture};
use shipstern_spl_token_parser::{account, instruction::Instruction, AccountParser, InstructionParser, TokenProgramState};

#[tokio::test]
async fn parses_a_mint() {
    let parser = AccountParser;
    let state = account_fixture!("3SmPYPvZfEmroktLiJsgaNENuPEud3Z52zSfLQ1zJdkK", &parser);

    let TokenProgramState {
        account: Some(account::Account::Mint(mint)),
    } = state
    else {
        panic!("Expected a mint account");
    };

    assert_eq!(mint.decimals, 10);
}
// #endregion account-fixture

// #region tx-fixture
#[tokio::test]
async fn parses_mint_to_checked() {
    let parser = InstructionParser;
    let ixs = tx_fixture!(
        "55kpnRufcX9Fo44oRBXtrkxPRww4UWJKxCpgBV39kzAAag8oyJbd9Y3YWdQQUi3TBqtrhjgsMGb9Nw8bUxy7j5rt",
        &parser
    );

    let Some(Instruction::MintToChecked(ix)) =
        ixs[0].as_ref().and_then(|parsed| parsed.instruction.as_ref())
    else {
        panic!("Expected MintToChecked");
    };

    assert_eq!(ix.args.decimals, 10);
}
// #endregion tx-fixture
