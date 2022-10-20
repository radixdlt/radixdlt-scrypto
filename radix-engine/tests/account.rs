use radix_engine::engine::ResourceChange;
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto::values::ScryptoValue;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

#[test]
fn can_withdraw_from_my_account() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method(
            other_account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .call_method(other_account, "balance", args!(RADIX_TOKEN))
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    let outputs = receipt.expect_commit_success();
    let other_account_balance: Decimal = scrypto_decode(&outputs[3]).unwrap();
    let transfer_amount = other_account_balance - 1000 /* initial balance */;
    let account_id: ComponentId = test_runner.deref_component(account).unwrap().into();
    let other_account_id: ComponentId = test_runner.deref_component(other_account).unwrap().into();

    assert_resource_changes_for_transfer(
        &receipt.expect_commit().resource_changes,
        RADIX_TOKEN,
        account_id,
        other_account_id,
        transfer_amount,
    );
}

#[test]
fn can_withdraw_non_fungible_from_my_account() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), account)
        .withdraw_from_account(resource_address, account)
        .call_method(
            other_account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_withdraw_from_other_account() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10u32.into(), account)
        .withdraw_from_account(RADIX_TOKEN, other_account)
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(is_auth_error);
}

#[test]
fn account_to_bucket_to_account() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), account)
        .withdraw_from_account(RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::CallMethod {
                    method_ident: ScryptoMethodIdent {
                        receiver: ScryptoReceiver::Global(account),
                        method_name: "deposit".to_string(),
                    },
                    args: args!(scrypto::resource::Bucket(bucket_id)),
                })
                .0
        })
        .build();

    // Act
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
    assert_eq!(1, receipt.expect_commit().resource_changes.len()); // Just the fee payment
}

#[test]
fn test_account_balance() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account1) = test_runner.new_account();
    let (_, _, account2) = test_runner.new_account();
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), account1)
        .call_method(account2, "balance", args!(RADIX_TOKEN))
        .build();

    // Act
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    let outputs = receipt.expect_commit_success();

    // Assert
    assert_eq!(1, receipt.expect_commit().resource_changes.len()); // Just the fee payment
    assert_eq!(
        outputs[1],
        ScryptoValue::from_typed(&Decimal::from(1000)).raw
    );
}

fn assert_resource_changes_for_transfer(
    resource_changes: &Vec<ResourceChange>,
    resource_address: ResourceAddress,
    source_account: ComponentId,
    target_account: ComponentId,
    transfer_amount: Decimal,
) {
    assert_eq!(3, resource_changes.len()); // Two transfers (withdrawal, deposit) + fee payment
    assert!(resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.component_id == source_account
            && r.amount == -transfer_amount));
    assert!(resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.component_id == target_account
            && r.amount == Decimal::from(transfer_amount)));
}
