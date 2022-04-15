#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

enum Action {
    Mint,
    Burn,
}


fn test_resource_auth(action: Action, set_auth: Option<usize>, auth_index: usize, expect_err: bool) {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let (auth_address0, token_address) = test_runner.create_restricted_token(account);
    let (_, auth_address1) = test_runner.create_restricted_token(account);
    let auth_addresses = [auth_address0, auth_address1];
    if let Some(i) = set_auth {
        let function = match action {
            Action::Mint => "set_mintable",
            Action::Burn => "set_burnable",
        };
        test_runner.set_auth((&pk, &sk, account), function, auth_address0, token_address, auth_addresses[i])
    }

    // Act
    let mut builder = test_runner.new_transaction_builder();
    builder.create_proof_from_account_by_amount(Decimal::one(), auth_addresses[auth_index], account);

    match action {
        Action::Mint => builder.mint(Decimal::from("1.0"), token_address),
        Action::Burn => builder
            .withdraw_from_account_by_amount(Decimal::from("1.0"), token_address, account)
            .burn(Decimal::from("1.0"), token_address),
    };

    let transaction = builder.call_method_with_all_resources(account, "deposit_batch")
        .build(test_runner.get_nonce([pk]))
        .sign([&sk]);
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    if expect_err {
        let err = receipt.result.expect_err("Should be a runtime error");
        assert_auth_error!(err);
    } else {
        receipt.result.expect("Should be okay.");
    }
}

#[test]
fn can_mint_with_right_auth() {
    test_resource_auth(Action::Mint, None, 0, false);
    test_resource_auth(Action::Mint, Option::Some(1), 1, false);
}

#[test]
fn cannot_mint_with_wrong_auth() {
    test_resource_auth(Action::Mint, None, 1, true);
    test_resource_auth(Action::Mint, Option::Some(1), 0, true);
}

#[test]
fn can_burn_with_auth() {
    test_resource_auth(Action::Burn, None, 0, false);
    test_resource_auth(Action::Burn, Option::Some(1), 1, false);
}

#[test]
fn cannot_burn_with_wrong_auth() {
    test_resource_auth(Action::Burn, None, 1, true);
    test_resource_auth(Action::Burn, Option::Some(1), 0, true);
}
