#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

enum Action {
    Mint,
    Burn,
    Withdraw,
}

fn test_resource_auth(action: Action, update_auth: bool, use_other_auth: bool, expect_err: bool) {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (pk, sk, account) = test_runner.new_account();
    let (token_address, mint_auth, burn_auth, withdraw_auth, admin_auth) =
        test_runner.create_restricted_token(account);

    let (_, updated_auth) = test_runner.create_restricted_burn_token(account);

    if update_auth {
        let function = match action {
            Action::Mint => "set_mintable",
            Action::Burn => "set_burnable",
            Action::Withdraw => "set_withdrawable",
        };
        test_runner.set_auth(
            (&pk, &sk, account),
            function,
            admin_auth,
            token_address,
            updated_auth,
        );
    }

    let auth_to_use = if use_other_auth {
        updated_auth
    } else {
        match action {
            Action::Mint => mint_auth,
            Action::Burn => burn_auth,
            Action::Withdraw => withdraw_auth,
        }
    };

    // Act
    let mut builder = test_runner.new_transaction_builder();
    builder.create_proof_from_account_by_amount(Decimal::one(), auth_to_use, account);

    match action {
        Action::Mint => builder.mint(Decimal::from("1.0"), token_address),
        Action::Burn => builder
            .create_proof_from_account(withdraw_auth, account)
            .withdraw_from_account_by_amount(Decimal::from("1.0"), token_address, account)
            .burn(Decimal::from("1.0"), token_address),
        Action::Withdraw => {
            builder.withdraw_from_account_by_amount(Decimal::from("1.0"), token_address, account)
        }
    };

    let transaction = builder
        .call_method_with_all_resources(account, "deposit_batch")
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
    test_resource_auth(Action::Mint, false, false, false);
    test_resource_auth(Action::Mint, true, true, false);
}

#[test]
fn cannot_mint_with_wrong_auth() {
    test_resource_auth(Action::Mint, false, true, true);
    test_resource_auth(Action::Mint, true, false, true);
}

#[test]
fn can_burn_with_auth() {
    test_resource_auth(Action::Burn, false, false, false);
    test_resource_auth(Action::Burn, true, true, false);
}

#[test]
fn cannot_burn_with_wrong_auth() {
    test_resource_auth(Action::Burn, false, true, true);
    test_resource_auth(Action::Burn, true, false, true);
}

#[test]
fn can_withdraw_with_auth() {
    test_resource_auth(Action::Withdraw, false, false, false);
    test_resource_auth(Action::Withdraw, true, true, false);
}

#[test]
fn cannot_withdraw_with_wrong_auth() {
    test_resource_auth(Action::Withdraw, false, true, true);
    test_resource_auth(Action::Withdraw, true, false, true);
}
