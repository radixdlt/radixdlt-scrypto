use scrypto_test::prelude::*;

#[test]
fn bottlenose_account_has_no_balance_method() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.from_bootstrap_to(ProtocolVersion::Bottlenose))
        .build();
    let (_, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account,
                ACCOUNT_BALANCE_IDENT,
                AccountBalanceInput {
                    resource_address: XRD,
                },
            )
            .build(),
        vec![],
    );

    // Assert
    receipt.expect_commit_failure();
}

#[test]
fn balance_method_returns_expected_amount_for_a_resource_the_account_has() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account,
                ACCOUNT_BALANCE_IDENT,
                AccountBalanceInput {
                    resource_address: XRD,
                },
            )
            .build(),
        vec![],
    );

    // Assert
    let balance = receipt
        .expect_commit_success()
        .output::<AccountBalanceOutput>(1);
    assert_eq!(balance, dec!(10_000));
}

#[test]
fn balance_method_returns_zero_for_a_resource_the_account_doesnt_have() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account,
                ACCOUNT_BALANCE_IDENT,
                AccountBalanceInput {
                    resource_address: ACCOUNT_OWNER_BADGE,
                },
            )
            .build(),
        vec![],
    );

    // Assert
    let balance = receipt
        .expect_commit_success()
        .output::<AccountBalanceOutput>(1);
    assert_eq!(balance, dec!(0));
}

#[test]
fn non_fungible_local_ids_method_returns_an_empty_set_if_vault_doesnt_exist() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account,
                ACCOUNT_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                AccountNonFungibleLocalIdsInput {
                    resource_address: VALIDATOR_OWNER_BADGE,
                    limit: 500,
                },
            )
            .build(),
        vec![],
    );

    // Assert
    let ids = receipt
        .expect_commit_success()
        .output::<AccountNonFungibleLocalIdsOutput>(1);
    assert_eq!(ids, indexset! {});
}

#[test]
fn non_fungible_local_ids_method_returns_expected_set() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let resource_address = ledger.create_non_fungible_resource(account);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account,
                ACCOUNT_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                AccountNonFungibleLocalIdsInput {
                    resource_address,
                    limit: 500,
                },
            )
            .build(),
        vec![],
    );

    // Assert
    let ids = receipt
        .expect_commit_success()
        .output::<AccountNonFungibleLocalIdsOutput>(1);
    assert_eq!(
        ids,
        [1, 2, 3]
            .into_iter()
            .map(NonFungibleLocalId::integer)
            .collect::<IndexSet<_>>()
    );
}

#[test]
fn non_fungible_local_ids_method_returns_expected_set_respecting_limit() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let resource_address = ledger.create_non_fungible_resource(account);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account,
                ACCOUNT_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                AccountNonFungibleLocalIdsInput {
                    resource_address,
                    limit: 2,
                },
            )
            .build(),
        vec![],
    );

    // Assert
    let ids = receipt
        .expect_commit_success()
        .output::<AccountNonFungibleLocalIdsOutput>(1);
    assert_eq!(
        ids,
        [2, 3]
            .into_iter()
            .map(NonFungibleLocalId::integer)
            .collect::<IndexSet<_>>()
    );
}

#[test]
fn has_non_fungible_returns_false_if_the_account_doesnt_have_the_non_fungible() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let resource_address = ledger.create_non_fungible_resource(account);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account,
                ACCOUNT_HAS_NON_FUNGIBLE_IDENT,
                AccountHasNonFungibleInput {
                    resource_address,
                    local_id: NonFungibleLocalId::integer(4),
                },
            )
            .build(),
        vec![],
    );

    // Assert
    let has_non_fungible = receipt
        .expect_commit_success()
        .output::<AccountHasNonFungibleOutput>(1);
    assert!(!has_non_fungible)
}

#[test]
fn has_non_fungible_returns_true_if_the_account_has_the_non_fungible() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let resource_address = ledger.create_non_fungible_resource(account);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                account,
                ACCOUNT_HAS_NON_FUNGIBLE_IDENT,
                AccountHasNonFungibleInput {
                    resource_address,
                    local_id: NonFungibleLocalId::integer(3),
                },
            )
            .build(),
        vec![],
    );

    // Assert
    let has_non_fungible = receipt
        .expect_commit_success()
        .output::<AccountHasNonFungibleOutput>(1);
    assert!(has_non_fungible)
}

#[test]
fn has_non_fungible_returns_false_if_the_non_fungibles_have_been_withdrawn() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);
    let resource_address = ledger.create_non_fungible_resource(account);

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, resource_address, 3)
            .call_method(
                account,
                ACCOUNT_HAS_NON_FUNGIBLE_IDENT,
                AccountHasNonFungibleInput {
                    resource_address,
                    local_id: NonFungibleLocalId::integer(3),
                },
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk)],
    );

    // Assert
    let has_non_fungible = receipt
        .expect_commit_success()
        .output::<AccountHasNonFungibleOutput>(2);
    assert!(!has_non_fungible)
}
