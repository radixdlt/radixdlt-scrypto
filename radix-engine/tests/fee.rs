use radix_engine::engine::ApplicationError;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::ledger::WriteableSubstateStore;
use radix_engine::model::KeyValueStoreEntryWrapper;
use radix_engine::model::WorktopError;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

fn run_manifest<F>(f: F) -> TransactionReceipt
where
    F: FnOnce(ComponentAddress) -> TransactionManifest,
{
    // Basic setup
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();

    // Publish package and instantiate component
    let package_address = test_runner.compile_and_publish("./tests/fee");
    let receipt1 = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .lock_fee(10.into(), account)
            .withdraw_from_account_by_amount(10.into(), RADIX_TOKEN, account)
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.call_function(
                    package_address,
                    "Fee",
                    "new",
                    args!(scrypto::resource::Bucket(bucket_id)),
                );
                builder
            })
            .build(),
        vec![public_key.into()],
    );
    let component_address = receipt1
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Run the provided manifest
    let manifest = f(component_address);
    test_runner.execute_manifest(manifest, vec![])
}

#[test]
fn should_succeed_when_fee_is_paid() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_method(component_address, "lock_fee", args!(Decimal::from(10)))
            .build()
    });

    receipt.expect_commit_success();
}

#[test]
fn should_be_rejected_when_no_fee_is_paid() {
    let receipt =
        run_manifest(|_| ManifestBuilder::new(&NetworkDefinition::local_simulator()).build());

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_insufficient_balance() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_method(
                component_address,
                "lock_fee_with_empty_vault",
                args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_non_xrd() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_method(
                component_address,
                "lock_fee_with_doge",
                args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_system_loan_is_not_fully_repaid() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_method(
                component_address,
                "lock_fee",
                args!(Decimal::from_str("0.001").unwrap()), // = 1000 cost units
            )
            .build()
    });

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_lock_fee_with_temp_vault() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_method(
                component_address,
                "lock_fee_with_temp_vault",
                args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_query_vault_and_lock_fee() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_method(
                component_address,
                "query_vault_and_lock_fee",
                args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_rejection();
}

#[test]
fn should_succeed_when_lock_fee_and_query_vault() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new(&NetworkDefinition::local_simulator())
            .call_method(
                component_address,
                "lock_fee_and_query_vault",
                args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_commit_success();
}

fn query_account_balance<'s, S>(
    test_runner: &mut TestRunner<'s, S>,
    account_address: ComponentAddress,
    resource_address: ResourceAddress,
) -> Decimal
where
    S: radix_engine::ledger::ReadableSubstateStore + WriteableSubstateStore,
{
    if let Some(account_comp) = test_runner.inspect_component_state(account_address) {
        let account_comp_state = ScryptoValue::from_slice(account_comp.state()).unwrap();
        if let Some(kv_store_id) = account_comp_state.kv_store_ids.iter().next() {
            if let Some(KeyValueStoreEntryWrapper(Some(value))) = test_runner
                .inspect_key_value_entry(kv_store_id.clone(), scrypto_encode(&resource_address))
            {
                let kv_entry_value = ScryptoValue::from_slice(&value).unwrap();
                let vault_id = kv_entry_value.vault_ids.iter().next().unwrap();
                let vault = test_runner.inspect_vault(vault_id.clone()).unwrap();
                return vault.total_amount();
            }
        }
    }
    return 0.into();
}

#[test]
fn test_fee_accounting_success() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account1) = test_runner.new_account();
    let (_, _, account2) = test_runner.new_account();
    let account1_balance = query_account_balance(&mut test_runner, account1, RADIX_TOKEN);
    let account2_balance = query_account_balance(&mut test_runner, account2, RADIX_TOKEN);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), account1)
        .withdraw_from_account_by_amount(66.into(), RADIX_TOKEN, account1)
        .call_method(
            account2,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_commit_success();
    let account1_new_balance = query_account_balance(&mut test_runner, account1, RADIX_TOKEN);
    let account2_new_balance = query_account_balance(&mut test_runner, account2, RADIX_TOKEN);
    let summary = &receipt.execution.fee_summary;
    assert_eq!(
        account1_new_balance,
        account1_balance
            - 66
            - (summary.cost_unit_price + summary.cost_unit_price * summary.tip_percentage / 100)
                * summary.cost_unit_consumed
    );
    assert_eq!(account2_new_balance, account2_balance + 66);
}

#[test]
fn test_fee_accounting_failure() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account1) = test_runner.new_account();
    let (_, _, account2) = test_runner.new_account();
    let account1_balance = query_account_balance(&mut test_runner, account1, RADIX_TOKEN);
    let account2_balance = query_account_balance(&mut test_runner, account2, RADIX_TOKEN);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), account1)
        .withdraw_from_account_by_amount(66.into(), RADIX_TOKEN, account1)
        .call_method(
            account2,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .assert_worktop_contains_by_amount(1.into(), RADIX_TOKEN)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::WorktopError(
                WorktopError::AssertionFailed
            ))
        )
    });
    let account1_new_balance = query_account_balance(&mut test_runner, account1, RADIX_TOKEN);
    let account2_new_balance = query_account_balance(&mut test_runner, account2, RADIX_TOKEN);
    let summary = &receipt.execution.fee_summary;
    assert_eq!(
        account1_new_balance,
        account1_balance
            - (summary.cost_unit_price + summary.cost_unit_price * summary.tip_percentage / 100)
                * summary.cost_unit_consumed
    );
    assert_eq!(account2_new_balance, account2_balance);
}

#[test]
fn test_fee_accounting_rejection() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account1) = test_runner.new_account();
    let account1_balance = query_account_balance(&mut test_runner, account1, RADIX_TOKEN);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(Decimal::from_str("0.000000000000000001").unwrap(), account1)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_rejection();
    let account1_new_balance = query_account_balance(&mut test_runner, account1, RADIX_TOKEN);
    assert_eq!(account1_new_balance, account1_balance);
}

#[test]
fn test_contingent_fee_accounting_success() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key1, _, account1) = test_runner.new_account();
    let (public_key2, _, account2) = test_runner.new_account();
    let account1_balance = query_account_balance(&mut test_runner, account1, RADIX_TOKEN);
    let account2_balance = query_account_balance(&mut test_runner, account2, RADIX_TOKEN);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(dec!("10"), account1)
        .lock_contingent_fee(dec!("0.001"), account2)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![public_key1.into(), public_key2.into()]);

    // Assert
    receipt.expect_commit_success();
    let account1_new_balance = query_account_balance(&mut test_runner, account1, RADIX_TOKEN);
    let account2_new_balance = query_account_balance(&mut test_runner, account2, RADIX_TOKEN);
    let summary = &receipt.execution.fee_summary;
    let effective_price =
        summary.cost_unit_price + summary.cost_unit_price * summary.tip_percentage / 100;
    let contingent_fee =
        (dec!("0.001") / effective_price).round(0, RoundingMode::TowardsZero) * effective_price;
    assert_eq!(
        account1_new_balance,
        account1_balance - effective_price * summary.cost_unit_consumed + contingent_fee
    );
    assert_eq!(account2_new_balance, account2_balance - contingent_fee);
}

#[test]
fn test_contingent_fee_accounting_failure() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key1, _, account1) = test_runner.new_account();
    let (public_key2, _, account2) = test_runner.new_account();
    let account1_balance = query_account_balance(&mut test_runner, account1, RADIX_TOKEN);
    let account2_balance = query_account_balance(&mut test_runner, account2, RADIX_TOKEN);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(dec!("10"), account1)
        .lock_contingent_fee(dec!("0.001"), account2)
        .assert_worktop_contains_by_amount(1.into(), RADIX_TOKEN)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![public_key1.into(), public_key2.into()]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::WorktopError(
                WorktopError::AssertionFailed
            ))
        )
    });
    let account1_new_balance = query_account_balance(&mut test_runner, account1, RADIX_TOKEN);
    let account2_new_balance = query_account_balance(&mut test_runner, account2, RADIX_TOKEN);
    let summary = &receipt.execution.fee_summary;
    let effective_price =
        summary.cost_unit_price + summary.cost_unit_price * summary.tip_percentage / 100;
    assert_eq!(
        account1_new_balance,
        account1_balance - effective_price * summary.cost_unit_consumed
    );
    assert_eq!(account2_new_balance, account2_balance);
}
