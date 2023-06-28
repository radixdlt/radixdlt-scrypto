use radix_engine::transaction::CommitResult;
use radix_engine::{transaction::BalanceChange, types::*};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_balance_changes_when_success() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
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
            .lock_fee(account, 500u32.into())
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
            .lock_fee(test_runner.faucet_component(), 500u32.into())
            .withdraw_from_account(account, RADIX_TOKEN, Decimal::ONE)
            .take_all_from_worktop(RADIX_TOKEN, |builder, bucket| {
                builder.call_method(component_address, "put", manifest_args!(bucket))
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let result = receipt.expect_commit(true);

    assert_eq!(result.balance_changes().len(), 5usize);
    assert_eq!(
        result.balance_changes(),
        &indexmap!(
            test_runner.faucet_component().into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(-(result.fee_summary.total_cost()))
            ),
            account.into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(dec!("-1"))
            ),
            component_address.into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(dec!("2")) // 1 for put another 1 for component royalties
            ),
            package_address.into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(dec!("2"))
            ),
            CONSENSUS_MANAGER.into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(result.fee_summary.expected_reward_if_single_validator())
            )
        )
    );
    assert!(result.direct_vault_updates().is_empty());
}

#[test]
fn test_balance_changes_when_failure() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
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
            .lock_fee(account, 500u32.into())
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
            .lock_fee(test_runner.faucet_component(), 500u32.into())
            .withdraw_from_account(account, RADIX_TOKEN, Decimal::ONE)
            .take_all_from_worktop(RADIX_TOKEN, |builder, bucket| {
                builder.call_method(component_address, "boom", manifest_args!(bucket))
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let result = receipt.expect_commit(false);
    assert!(result.direct_vault_updates().is_empty());
    assert_eq!(
        result.balance_changes(),
        &indexmap!(
            test_runner.faucet_component().into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(-(result.fee_summary.total_cost()))
            ),
            CONSENSUS_MANAGER.into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(result.fee_summary.expected_reward_if_single_validator())
            )
        )
    )
}

#[test]
fn test_balance_changes_when_recall() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    let recallable_token = test_runner.create_recallable_token(account);
    let vaults = test_runner.get_component_vaults(account, recallable_token);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .recall(
            InternalAddress::new_or_panic(vault_id.into()),
            Decimal::one(),
        )
        .call_method(
            other_account,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let result = receipt.expect_commit(true);
    assert_eq!(
        result.balance_changes(),
        &indexmap!(
            test_runner.faucet_component().into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(-(result.fee_summary.total_cost()))
            ),
            other_account.into() => indexmap!(
                recallable_token => BalanceChange::Fungible(dec!("1"))
            ),
            CONSENSUS_MANAGER.into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(result.fee_summary.expected_reward_if_single_validator())
            )
        )
    );
    assert_eq!(
        result.direct_vault_updates(),
        &indexmap!(
            vault_id => indexmap!(
                recallable_token => BalanceChange::Fungible(dec!("-1"))
            )
        )
    )
}

#[test]
fn test_balance_changes_when_transferring_non_fungibles() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .withdraw_from_account(account, resource_address, dec!("1.0"))
        .call_method(
            other_account,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    let result = receipt.expect_commit(true);

    assert_eq!(
        result
            .balance_changes()
            .keys()
            .cloned()
            .collect::<HashSet<GlobalAddress>>(),
        hashset![
            account.into(),
            other_account.into(),
            test_runner.faucet_component().into(),
            CONSENSUS_MANAGER.into(),
        ]
    );

    let (account_added, account_removed) =
        get_non_fungible_changes(result, &account, &resource_address);
    assert_eq!(account_added, BTreeSet::new());
    assert_eq!(account_removed.len(), 1);
    let transferred_non_fungible = account_removed.first().unwrap().clone();

    let (other_account_added, other_account_removed) =
        get_non_fungible_changes(result, &other_account, &resource_address);
    assert_eq!(other_account_added, btreeset!(transferred_non_fungible));
    assert_eq!(other_account_removed, BTreeSet::new());

    let faucet_changes = result
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.faucet_component()))
        .unwrap();
    let total_cost_xrd = result.fee_summary.total_cost();
    assert_eq!(
        faucet_changes,
        &indexmap!(
            RADIX_TOKEN => BalanceChange::Fungible(-total_cost_xrd),
        ),
    );

    assert!(result.direct_vault_updates().is_empty())
}

fn get_non_fungible_changes(
    result: &CommitResult,
    account: &ComponentAddress,
    resource_address: &ResourceAddress,
) -> (BTreeSet<NonFungibleLocalId>, BTreeSet<NonFungibleLocalId>) {
    let balance_change = result
        .balance_changes()
        .get(&GlobalAddress::from(account.clone()))
        .unwrap()
        .get(resource_address)
        .unwrap();
    let account_changes = if let BalanceChange::NonFungible { added, removed } = balance_change {
        Some((added.clone(), removed.clone()))
    } else {
        None
    };
    account_changes.unwrap()
}
