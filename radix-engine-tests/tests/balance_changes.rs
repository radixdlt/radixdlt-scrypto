use radix_engine::{transaction::BalanceChange, types::*};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_balance_changes_when_success() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));
    let package_address = test_runner.compile_and_publish_with_owner(
        "./tests/blueprints/balance_changes",
        owner_badge_addr.clone(),
    );

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_function(
                package_address,
                "BalanceChangesTest",
                "instantiate",
                manifest_args!(),
            )
            .build(),
        vec![
            NonFungibleGlobalId::from_public_key(&public_key),
            owner_badge_addr.clone(),
        ],
    );
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Call the put method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, XRD, Decimal::ONE)
            .take_all_from_worktop(XRD, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_method(
                    component_address,
                    "put",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let result = receipt.expect_commit_success();

    assert_eq!(
        test_runner
            .sum_descendant_balance_changes(result, test_runner.faucet_component().as_node_id()),
        indexmap!(
            XRD => BalanceChange::Fungible(receipt.fee_summary.total_cost().safe_neg().unwrap())
        )
    );
    assert_eq!(
        test_runner.sum_descendant_balance_changes(result, account.as_node_id()),
        indexmap!(
            XRD => BalanceChange::Fungible(dec!("-1"))
        )
    );
    assert_eq!(
        test_runner.sum_descendant_balance_changes(result, component_address.as_node_id()),
        indexmap!(
            XRD => BalanceChange::Fungible(dec!("2")) // 1 for put another 1 for component royalties
        )
    );
    assert_eq!(
        test_runner.sum_descendant_balance_changes(result, package_address.as_node_id()),
        indexmap!(
            XRD => BalanceChange::Fungible(dec!("2"))
        )
    );
    assert_eq!(
        test_runner.sum_descendant_balance_changes(result, CONSENSUS_MANAGER.as_node_id()),
        indexmap!(
            XRD => BalanceChange::Fungible(receipt.fee_summary.expected_reward_if_single_validator())
        )
    );
}

#[test]
fn test_balance_changes_when_failure() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));
    let package_address = test_runner.compile_and_publish_with_owner(
        "./tests/blueprints/balance_changes",
        owner_badge_addr.clone(),
    );

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_function(
                package_address,
                "BalanceChangesTest",
                "instantiate",
                manifest_args!(),
            )
            .build(),
        vec![
            NonFungibleGlobalId::from_public_key(&public_key),
            owner_badge_addr.clone(),
        ],
    );
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Call the put method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, XRD, Decimal::ONE)
            .take_all_from_worktop(XRD, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_method(
                    component_address,
                    "boom",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let result = receipt.expect_commit_failure();

    assert_eq!(
        test_runner
            .sum_descendant_balance_changes(result, test_runner.faucet_component().as_node_id(),),
        indexmap!(
            XRD => BalanceChange::Fungible(receipt.fee_summary.total_cost().safe_neg().unwrap() )
        )
    );
    assert_eq!(
        test_runner.sum_descendant_balance_changes(result, CONSENSUS_MANAGER.as_node_id()),
        indexmap!(
            XRD => BalanceChange::Fungible(receipt.fee_summary.expected_reward_if_single_validator())
        )
    );
}

#[test]
fn test_balance_changes_when_recall() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    let recallable_token = test_runner.create_recallable_token(account);
    let vaults = test_runner.get_component_vaults(account, recallable_token);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .recall(
            InternalAddress::new_or_panic(vault_id.into()),
            Decimal::one(),
        )
        .try_deposit_batch_or_abort(other_account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let result = receipt.expect_commit_success();

    assert_eq!(
        test_runner
            .sum_descendant_balance_changes(result, test_runner.faucet_component().as_node_id()),
        indexmap!(
            XRD => BalanceChange::Fungible(receipt.fee_summary.total_cost().safe_neg().unwrap() )
        )
    );
    assert_eq!(
        test_runner.sum_descendant_balance_changes(result, other_account.as_node_id()),
        indexmap!(
            recallable_token => BalanceChange::Fungible(dec!(1))
        )
    );
    assert_eq!(
        test_runner.sum_descendant_balance_changes(result, CONSENSUS_MANAGER.as_node_id()),
        indexmap!(
            XRD => BalanceChange::Fungible(receipt.fee_summary.expected_reward_if_single_validator())
        )
    );
    assert_eq!(
        test_runner.sum_descendant_balance_changes(result, account.as_node_id()),
        indexmap!(
            recallable_token => BalanceChange::Fungible(dec!("-1"))
        )
    );
}

#[test]
fn test_balance_changes_when_transferring_non_fungibles() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pk, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resource_address, dec!("1.0"))
        .try_deposit_batch_or_abort(other_account, None)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    let result = receipt.expect_commit_success();

    let BalanceChange::NonFungible {
        added: account_added,
        removed: account_removed,
    } = test_runner
        .sum_descendant_balance_changes(result, account.as_node_id())
        .get(&resource_address)
        .unwrap()
        .clone()
    else {
        panic!("must be non-fungible")
    };
    assert_eq!(account_added, BTreeSet::new());
    assert_eq!(account_removed.len(), 1);
    let transferred_non_fungible = account_removed.first().unwrap().clone();

    let BalanceChange::NonFungible {
        added: other_account_added,
        removed: other_account_removed,
    } = test_runner
        .sum_descendant_balance_changes(result, other_account.as_node_id())
        .get(&resource_address)
        .unwrap()
        .clone()
    else {
        panic!("must be non-fungible")
    };
    assert_eq!(other_account_added, btreeset!(transferred_non_fungible));
    assert_eq!(other_account_removed, BTreeSet::new());

    let faucet_changes = test_runner
        .sum_descendant_balance_changes(result, test_runner.faucet_component().as_node_id());
    let total_cost_in_xrd = receipt.fee_summary.total_cost();
    assert_eq!(
        faucet_changes,
        indexmap!(
            XRD => BalanceChange::Fungible(total_cost_in_xrd.safe_neg().unwrap()),
        ),
    );
}
