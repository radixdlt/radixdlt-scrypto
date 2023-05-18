use radix_engine::errors::{ApplicationError, ModuleError, RuntimeError, SystemError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::system::system_modules::execution_trace::ResourceChange;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataEntry, MetadataValue};
use radix_engine_interface::blueprints::account::{
    AccountAddResourceToAllowedDepositsListInput, AccountChangeAllowedDepositsModeInput,
    AccountDepositsMode, AccountRemoveResourceFromAllowedDepositsListInput,
    AccountRemoveResourceFromDisallowedDepositsListInput, AccountSecurifyInput,
    ACCOUNT_ADD_RESOURCE_TO_ALLOWED_DEPOSITS_LIST_IDENT,
    ACCOUNT_CHANGE_ALLOWED_DEPOSITS_MODE_IDENT, ACCOUNT_DEPOSIT_BATCH_IDENT,
    ACCOUNT_REMOVE_RESOURCE_FROM_ALLOWED_DEPOSITS_LIST_IDENT,
    ACCOUNT_REMOVE_RESOURCE_FROM_DISALLOWED_DEPOSITS_LIST_IDENT, ACCOUNT_SECURIFY_IDENT,
};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use radix_engine_queries::typed_substate_layout::FungibleResourceManagerError;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::Instruction;

#[test]
fn can_securify_virtual_account() {
    securify_account(true, true, true);
}

#[test]
fn cannot_securify_virtual_account_without_key() {
    securify_account(true, false, false);
}

#[test]
fn cannot_securify_allocated_account() {
    securify_account(false, true, false);
}

fn securify_account(is_virtual: bool, use_key: bool, expect_success: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (key, _, account) = test_runner.new_account(is_virtual);

    let (_, _, storing_account) = test_runner.new_account(true);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            account,
            ACCOUNT_SECURIFY_IDENT,
            to_manifest_value(&AccountSecurifyInput {}),
        )
        .call_method(
            storing_account,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let initial_proofs = if use_key {
        vec![NonFungibleGlobalId::from_public_key(&key)]
    } else {
        vec![]
    };
    let receipt = test_runner.execute_manifest(manifest, initial_proofs);

    // Assert
    if expect_success {
        receipt.expect_commit_success();
    } else {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. }))
            )
        });
    }
}

#[test]
fn can_withdraw_from_my_allocated_account() {
    can_withdraw_from_my_account_internal(|test_runner| {
        let (public_key, _, account) = test_runner.new_account(false);
        (public_key, account)
    });
}

#[test]
fn can_withdraw_from_my_virtual_account() {
    can_withdraw_from_my_account_internal(|test_runner| {
        let (public_key, _, account) = test_runner.new_account(true);
        (public_key, account)
    });
}

fn can_withdraw_from_my_account_internal<F>(new_account: F)
where
    F: FnOnce(&mut TestRunner) -> (EcdsaSecp256k1PublicKey, ComponentAddress),
{
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, account) = new_account(&mut test_runner);
    let (_, _, other_account) = test_runner.new_account(true);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw(account, 10.into(), RADIX_TOKEN, 1.into())
        .call_method(
            other_account,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    let other_account_balance: Decimal = test_runner
        .account_balance(other_account, RADIX_TOKEN)
        .unwrap();
    let transfer_amount = other_account_balance - 10000 /* initial balance */;

    assert_resource_changes_for_transfer(
        &receipt
            .execution_trace
            .resource_changes
            .iter()
            .flat_map(|(_, rc)| rc)
            .cloned()
            .collect(),
        RADIX_TOKEN,
        other_account,
        transfer_amount,
    );
}

fn can_withdraw_non_fungible_from_my_account_internal(use_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(use_virtual);
    let (_, _, other_account) = test_runner.new_account(use_virtual);
    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw(account, 10.into(), resource_address, 1.into())
        .call_method(
            other_account,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_withdraw_non_fungible_from_my_allocated_account() {
    can_withdraw_non_fungible_from_my_account_internal(false)
}

#[test]
fn can_withdraw_non_fungible_from_my_virtual_account() {
    can_withdraw_non_fungible_from_my_account_internal(true)
}

fn cannot_withdraw_from_other_account_internal(is_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(is_virtual);
    let (_, _, other_account) = test_runner.new_account(is_virtual);
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 10u32.into())
        .withdraw_from_account(other_account, RADIX_TOKEN, 1.into())
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(is_auth_error);
}

#[test]
fn virtual_account_is_created_with_public_key_hash_metadata() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let (public_key, _, account) = test_runner.new_account(true);

    // Assert
    let entry = test_runner.get_metadata(account.into(), "owner_keys");

    let public_key_hash = public_key.get_hash().into_enum();
    assert_eq!(
        entry,
        Some(MetadataEntry::List(vec![MetadataValue::PublicKeyHash(
            public_key_hash
        )]))
    );
}

#[test]
fn cannot_withdraw_from_other_allocated_account() {
    cannot_withdraw_from_other_account_internal(false);
}

#[test]
fn cannot_withdraw_from_other_virtual_account() {
    cannot_withdraw_from_other_account_internal(true);
}

fn account_to_bucket_to_account_internal(use_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(use_virtual);
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw(account, 10u32.into(), RADIX_TOKEN, 1.into())
        .take_all_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::CallMethod {
                    component_address: account,
                    method_name: "deposit".to_string(),
                    args: manifest_args!(bucket_id),
                })
                .0
        })
        .build();

    // Act
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    let result = receipt.expect_commit_success();

    let vault_id = test_runner
        .get_component_vaults(account, RADIX_TOKEN)
        .first()
        .cloned()
        .unwrap();
    assert_eq!(
        receipt.execution_trace.resource_changes,
        indexmap!(
            0 => vec![ResourceChange {
                node_id: account.into(),
                vault_id,
                resource_address: RADIX_TOKEN,
                amount: - result.fee_summary.total_execution_cost_xrd - dec!("1")
            }],
            2 => vec![ResourceChange {
                node_id: account.into(),
                vault_id,
                resource_address: RADIX_TOKEN,
                amount: dec!("1")
            }],
        )
    );
}

#[test]
fn account_to_bucket_to_allocated_account() {
    account_to_bucket_to_account_internal(false);
}

#[test]
fn account_to_bucket_to_virtual_account() {
    account_to_bucket_to_account_internal(true);
}

fn assert_resource_changes_for_transfer(
    resource_changes: &Vec<ResourceChange>,
    resource_address: ResourceAddress,
    target_account: ComponentAddress,
    transfer_amount: Decimal,
) {
    println!("transfer: {:?}", transfer_amount);
    println!("{:?}", resource_changes);
    assert_eq!(2, resource_changes.len()); // Two transfers (withdraw + fee, deposit)
    assert!(resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.node_id == target_account.into()
            && r.amount == Decimal::from(transfer_amount)));
}

fn account_deposit_mode_change_test(
    sign: bool,
    is_virtual: bool,
    failure: Option<&dyn Fn(&RuntimeError) -> bool>,
) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(is_virtual);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            account,
            ACCOUNT_CHANGE_ALLOWED_DEPOSITS_MODE_IDENT,
            to_manifest_value(&AccountChangeAllowedDepositsModeInput {
                deposit_mode: AccountDepositsMode::AllowExisting,
            }),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        if sign {
            vec![NonFungibleGlobalId::from_public_key(&public_key)]
        } else {
            vec![]
        },
    );

    // Assert
    if let Some(failure_function) = failure {
        receipt.expect_specific_failure(failure_function);
    } else {
        receipt.expect_commit_success();
    }
}

#[test]
fn virtual_account_deposits_mode_can_not_be_changed_without_owner_auth() {
    account_deposit_mode_change_test(false, true, Some(&is_auth_unauthorized_error))
}

#[test]
fn virtual_account_deposits_mode_can_be_changed_with_owner_auth() {
    account_deposit_mode_change_test(true, true, None)
}

#[test]
fn allocated_account_deposits_mode_can_not_be_changed_without_owner_auth() {
    account_deposit_mode_change_test(false, false, Some(&is_auth_unauthorized_error))
}

#[test]
fn allocated_account_deposits_mode_can_be_changed_with_owner_auth() {
    account_deposit_mode_change_test(true, false, None)
}

#[test]
fn cant_add_allowed_resources_without_owner_auth() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (_, _, account) = test_runner.new_account(false);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            account,
            ACCOUNT_ADD_RESOURCE_TO_ALLOWED_DEPOSITS_LIST_IDENT,
            to_manifest_value(&AccountAddResourceToAllowedDepositsListInput {
                resource_address: RADIX_TOKEN,
            }),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error)
}

#[test]
fn cant_add_allowed_resources_when_in_invalid_mode() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(false);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            account,
            ACCOUNT_ADD_RESOURCE_TO_ALLOWED_DEPOSITS_LIST_IDENT,
            to_manifest_value(&AccountAddResourceToAllowedDepositsListInput {
                resource_address: RADIX_TOKEN,
            }),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(is_account_error)
}

#[test]
fn can_deposit_any_resource_in_allow_all_mode() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(false);
    let resource_address = test_runner.create_freely_mintable_fungible_resource(None, 18, account);

    // Act
    let manifest = ManifestBuilder::new()
        .mint_fungible(resource_address, 1.into())
        .deposit_batch(account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

fn allow_existing_internal_test(
    sign: bool,
    deposit_existing_resource: bool,
    safe_deposit: bool,
    failure: Option<&dyn Fn(&RuntimeError) -> bool>,
) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(false);

    let non_existing_resource =
        test_runner.create_freely_mintable_fungible_resource(None, 18, account);
    let existing_resource =
        test_runner.create_freely_mintable_fungible_resource(Some(10.into()), 18, account);

    let resource_to_deposit = if deposit_existing_resource {
        existing_resource
    } else {
        non_existing_resource
    };

    {
        let manifest = ManifestBuilder::new()
            .call_method(
                account,
                ACCOUNT_CHANGE_ALLOWED_DEPOSITS_MODE_IDENT,
                to_manifest_value(&AccountChangeAllowedDepositsModeInput {
                    deposit_mode: AccountDepositsMode::AllowExisting,
                }),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();
    }

    // Act
    let manifest = {
        let mut manifest_builder = &mut ManifestBuilder::new();

        manifest_builder = manifest_builder.mint_fungible(resource_to_deposit, 1.into());
        manifest_builder = if safe_deposit {
            manifest_builder.safe_deposit_batch(account)
        } else {
            manifest_builder.deposit_batch(account)
        };

        manifest_builder.build()
    };
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        if sign {
            vec![NonFungibleGlobalId::from_public_key(&public_key)]
        } else {
            vec![]
        },
    );

    // Assert
    if let Some(failure_function) = failure {
        receipt.expect_specific_failure(failure_function);
    } else {
        receipt.expect_commit_success();
    }
}

fn allow_list_internal_test(
    sign: bool,
    deposit_allowed_resource: bool,
    safe_deposit: bool,
    failure: Option<&dyn Fn(&RuntimeError) -> bool>,
) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(false);

    let allowed_resource = test_runner.create_freely_mintable_fungible_resource(None, 18, account);
    let non_allowed_resource =
        test_runner.create_freely_mintable_fungible_resource(None, 18, account);

    let resource_to_deposit = if deposit_allowed_resource {
        allowed_resource
    } else {
        non_allowed_resource
    };

    {
        let mut allowed_resources = index_set_new();
        allowed_resources.insert(allowed_resource);

        let manifest = ManifestBuilder::new()
            .call_method(
                account,
                ACCOUNT_CHANGE_ALLOWED_DEPOSITS_MODE_IDENT,
                to_manifest_value(&AccountChangeAllowedDepositsModeInput {
                    deposit_mode: AccountDepositsMode::AllowList(allowed_resources),
                }),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();
    }

    // Act
    let manifest = {
        let mut manifest_builder = &mut ManifestBuilder::new();

        manifest_builder = manifest_builder.mint_fungible(resource_to_deposit, 1.into());
        manifest_builder = if safe_deposit {
            manifest_builder.safe_deposit_batch(account)
        } else {
            manifest_builder.deposit_batch(account)
        };

        manifest_builder.build()
    };
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        if sign {
            vec![NonFungibleGlobalId::from_public_key(&public_key)]
        } else {
            vec![]
        },
    );

    // Assert
    if let Some(failure_function) = failure {
        receipt.expect_specific_failure(failure_function);
    } else {
        receipt.expect_commit_success();
    }
}

fn deny_list_internal_test(
    sign: bool,
    deposit_denied_resource: bool,
    safe_deposit: bool,
    failure: Option<&dyn Fn(&RuntimeError) -> bool>,
) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(false);

    let allowed_resource = test_runner.create_freely_mintable_fungible_resource(None, 18, account);
    let non_allowed_resource =
        test_runner.create_freely_mintable_fungible_resource(None, 18, account);

    let resource_to_deposit = if deposit_denied_resource {
        non_allowed_resource
    } else {
        allowed_resource
    };

    {
        let mut denied_resources = index_set_new();
        denied_resources.insert(non_allowed_resource);

        let manifest = ManifestBuilder::new()
            .call_method(
                account,
                ACCOUNT_CHANGE_ALLOWED_DEPOSITS_MODE_IDENT,
                to_manifest_value(&AccountChangeAllowedDepositsModeInput {
                    deposit_mode: AccountDepositsMode::DisallowList(denied_resources),
                }),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();
    }

    // Act
    let manifest = {
        let mut manifest_builder = &mut ManifestBuilder::new();

        manifest_builder = manifest_builder.mint_fungible(resource_to_deposit, 1.into());
        manifest_builder = if safe_deposit {
            manifest_builder.safe_deposit_batch(account)
        } else {
            manifest_builder.deposit_batch(account)
        };

        manifest_builder.build()
    };
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        if sign {
            vec![NonFungibleGlobalId::from_public_key(&public_key)]
        } else {
            vec![]
        },
    );

    // Assert
    if let Some(failure_function) = failure {
        receipt.expect_specific_failure(failure_function);
    } else {
        receipt.expect_commit_success();
    }
}

#[test]
fn test_existing_resources_only_deposit_mode() {
    let test_vectors: [(bool, bool, bool, Option<ErrorCheckingFunction>); 8] = [
        // Should sign: true
        // Deposit existing resource: true
        // Safe deposit: false
        (true, true, false, None),
        // Should sign: true
        // Deposit existing resource: false
        // Safe deposit: false
        (true, false, false, None),
        // Should sign: false
        // Deposit existing resource: true
        // Safe deposit: false
        (false, true, false, None),
        // Should sign: false
        // Deposit existing resource: false
        // Safe deposit: false
        (false, false, false, Some(&is_assert_rule_error)),
        // Should sign: true
        // Deposit existing resource: true
        // Safe deposit: true
        (true, true, true, None),
        // Should sign: true
        // Deposit existing resource: false
        // Safe deposit: true
        (true, false, true, Some(&is_drop_non_empty_bucket_error)),
        // Should sign: false
        // Deposit existing resource: true
        // Safe deposit: true
        (false, true, true, None),
        // Should sign: false
        // Deposit existing resource: false
        // Safe deposit: true
        (false, false, true, Some(&is_drop_non_empty_bucket_error)),
    ];

    for (should_sign, deposit_existing_resource, safe_deposit, error) in test_vectors.into_iter() {
        allow_existing_internal_test(should_sign, deposit_existing_resource, safe_deposit, error)
    }
}

#[test]
fn test_allow_list_only_deposit_mode() {
    let test_vectors: [(bool, bool, bool, Option<ErrorCheckingFunction>); 8] = [
        // Should sign: true
        // Deposit allowed resource: true
        // Safe Deposit: false
        (true, true, false, None),
        // Should sign: true
        // Deposit allowed resource: false
        // Safe Deposit: false
        (true, false, false, None),
        // Should sign: false
        // Deposit allowed resource: true
        // Safe Deposit: false
        (false, true, false, None),
        // Should sign: false
        // Deposit allowed resource: false
        // Safe Deposit: false
        (false, false, false, Some(&is_assert_rule_error)),
        // Should sign: true
        // Deposit allowed resource: true
        // Safe Deposit: true
        (true, true, true, None),
        // Should sign: true
        // Deposit allowed resource: false
        // Safe Deposit: true
        (true, false, true, Some(&is_drop_non_empty_bucket_error)),
        // Should sign: false
        // Deposit allowed resource: true
        // Safe Deposit: true
        (false, true, true, None),
        // Should sign: false
        // Deposit allowed resource: false
        // Safe Deposit: true
        (false, false, true, Some(&is_drop_non_empty_bucket_error)),
    ];

    for (should_sign, deposit_allowed_resource, safe_deposit, error) in test_vectors.into_iter() {
        allow_list_internal_test(should_sign, deposit_allowed_resource, safe_deposit, error)
    }
}

#[test]
fn test_deny_list_only_deposit_mode() {
    let test_vectors: [(bool, bool, bool, Option<ErrorCheckingFunction>); 8] = [
        // Should sign: true
        // Deposit denied resource: true
        // Safe Deposit: false
        (true, true, false, None),
        // Should sign: true
        // Deposit denied resource: false
        // Safe Deposit: false
        (true, false, false, None),
        // Should sign: false
        // Deposit denied resource: true
        // Safe Deposit: false
        (false, true, false, Some(&is_assert_rule_error)),
        // Should sign: false
        // Deposit denied resource: false
        // Safe Deposit: false
        (false, false, false, None),
        // Should sign: true
        // Deposit denied resource: true
        // Safe Deposit: true
        (true, true, true, Some(&is_drop_non_empty_bucket_error)),
        // Should sign: true
        // Deposit denied resource: false
        // Safe Deposit: true
        (true, false, true, None),
        // Should sign: false
        // Deposit denied resource: true
        // Safe Deposit: true
        (false, true, true, Some(&is_drop_non_empty_bucket_error)),
        // Should sign: false
        // Deposit denied resource: false
        // Safe Deposit: true
        (false, false, true, None),
    ];

    for (should_sign, deposit_denied_resource, safe_deposit, error) in test_vectors.into_iter() {
        deny_list_internal_test(should_sign, deposit_denied_resource, safe_deposit, error)
    }
}

#[test]
fn removing_an_allowed_resource_disallows_its_deposit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(false);

    let resource_address = test_runner.create_freely_mintable_fungible_resource(None, 18, account);

    {
        let mut allowed_resources = index_set_new();
        allowed_resources.insert(resource_address);

        let manifest = ManifestBuilder::new()
            .call_method(
                account,
                ACCOUNT_CHANGE_ALLOWED_DEPOSITS_MODE_IDENT,
                to_manifest_value(&AccountChangeAllowedDepositsModeInput {
                    deposit_mode: AccountDepositsMode::AllowList(allowed_resources),
                }),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        let manifest = ManifestBuilder::new()
            .call_method(
                account,
                ACCOUNT_REMOVE_RESOURCE_FROM_ALLOWED_DEPOSITS_LIST_IDENT,
                to_manifest_value(&AccountRemoveResourceFromAllowedDepositsListInput {
                    resource_address,
                }),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();
    }

    // Act
    let manifest = ManifestBuilder::new()
        .mint_fungible(resource_address, 10.into())
        .deposit_batch(account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_assert_rule_error)
}

#[test]
fn removing_a_disallowed_resource_allows_its_deposit() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(false);

    let resource_address = test_runner.create_freely_mintable_fungible_resource(None, 18, account);

    {
        let mut disallowed_resources = index_set_new();
        disallowed_resources.insert(resource_address);

        let manifest = ManifestBuilder::new()
            .call_method(
                account,
                ACCOUNT_CHANGE_ALLOWED_DEPOSITS_MODE_IDENT,
                to_manifest_value(&AccountChangeAllowedDepositsModeInput {
                    deposit_mode: AccountDepositsMode::DisallowList(disallowed_resources),
                }),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        let manifest = ManifestBuilder::new()
            .call_method(
                account,
                ACCOUNT_REMOVE_RESOURCE_FROM_DISALLOWED_DEPOSITS_LIST_IDENT,
                to_manifest_value(&AccountRemoveResourceFromDisallowedDepositsListInput {
                    resource_address,
                }),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();
    }

    // Act
    let manifest = ManifestBuilder::new()
        .mint_fungible(resource_address, 10.into())
        .deposit_batch(account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn only_existing_is_dependent_on_balances_not_vault_existence() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(false);

    let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
        Some(1.into()),
        18,
        account,
    );

    {
        let manifest = ManifestBuilder::new()
            .call_method(
                account,
                ACCOUNT_CHANGE_ALLOWED_DEPOSITS_MODE_IDENT,
                to_manifest_value(&AccountChangeAllowedDepositsModeInput {
                    deposit_mode: AccountDepositsMode::AllowExisting,
                }),
            )
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();
    }

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account, resource_address, 1.into())
        .safe_deposit_batch(account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(is_drop_non_empty_bucket_error);
}

type ErrorCheckingFunction = &'static dyn Fn(&RuntimeError) -> bool;

fn is_auth_unauthorized_error(runtime_error: &RuntimeError) -> bool {
    matches!(
        runtime_error,
        RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized(_)))
    )
}

fn is_account_error(runtime_error: &RuntimeError) -> bool {
    matches!(
        runtime_error,
        RuntimeError::ApplicationError(ApplicationError::AccountError(..))
    )
}

fn is_assert_rule_error(runtime_error: &RuntimeError) -> bool {
    matches!(
        runtime_error,
        RuntimeError::SystemError(SystemError::AssertAccessRuleFailed)
    )
}

fn is_drop_non_empty_bucket_error(runtime_error: &RuntimeError) -> bool {
    matches!(
        runtime_error,
        RuntimeError::ApplicationError(ApplicationError::ResourceManagerError(
            FungibleResourceManagerError::DropNonEmptyBucket
        ))
    )
}
