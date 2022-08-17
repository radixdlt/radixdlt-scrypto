extern crate core;

use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

enum Action {
    Mint,
    Burn,
    Withdraw,
    Deposit,
}

fn test_resource_auth(action: Action, update_auth: bool, use_other_auth: bool, expect_err: bool) {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (token_address, mint_auth, burn_auth, withdraw_auth, admin_auth) =
        test_runner.create_restricted_token(account);
    let (_, updated_auth) = test_runner.create_restricted_burn_token(account);

    if update_auth {
        let function = match action {
            Action::Mint => "set_mintable",
            Action::Burn => "set_burnable",
            Action::Withdraw => "set_withdrawable",
            Action::Deposit => "set_depositable",
        };
        test_runner.update_resource_auth(
            function,
            admin_auth,
            token_address,
            updated_auth,
            account,
            public_key,
        );
    }

    let auth_to_use = if use_other_auth {
        updated_auth
    } else {
        match action {
            Action::Mint => mint_auth,
            Action::Burn => burn_auth,
            Action::Withdraw => withdraw_auth,
            Action::Deposit => mint_auth, // Any bad auth
        }
    };

    // Act
    let mut builder = ManifestBuilder::new(Network::LocalSimulator);
    builder.lock_fee(10.into(), SYSTEM_COMPONENT);
    builder.create_proof_from_account_by_amount(Decimal::one(), auth_to_use, account);

    match action {
        Action::Mint => builder
            .mint(Decimal::from("1.0"), token_address)
            .call_method_with_all_resources(account, "deposit_batch"),
        Action::Burn => builder
            .create_proof_from_account(withdraw_auth, account)
            .withdraw_from_account_by_amount(Decimal::from("1.0"), token_address, account)
            .burn(Decimal::from("1.0"), token_address)
            .call_method_with_all_resources(account, "deposit_batch"),
        Action::Withdraw => builder
            .withdraw_from_account_by_amount(Decimal::from("1.0"), token_address, account)
            .call_method_with_all_resources(account, "deposit_batch"),
        Action::Deposit => builder
            .create_proof_from_account(withdraw_auth, account)
            .withdraw_from_account_by_amount(Decimal::from("1.0"), token_address, account)
            .take_from_worktop(token_address, |builder, bucket_id| {
                builder.call_method(
                    account,
                    "deposit",
                    args!(scrypto::resource::Bucket(bucket_id)),
                )
            })
            .call_method_with_all_resources(account, "deposit_batch"),
    };

    let manifest = builder.build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    if expect_err {
        receipt.expect_failure(is_auth_error);
    } else {
        receipt.expect_success();
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

#[test]
fn cannot_deposit_with_wrong_auth() {
    test_resource_auth(Action::Deposit, true, false, true);
}

#[test]
fn can_deposit_with_right_auth() {
    test_resource_auth(Action::Deposit, true, true, false);
}
