use radix_common::prelude::*;
use radix_transactions::manifest::static_resource_movements::*;
use radix_transactions::manifest::*;
use radix_transactions::prelude::*;

#[test]
fn simple_account_transfer_with_an_explicit_take_all_is_correctly_classified() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let manifest = ManifestBuilder::new_v2()
        .lock_fee_and_withdraw(account1, 100, XRD, 10)
        .take_all_from_worktop(XRD, "bucket")
        .deposit(account2, "bucket")
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest).unwrap();

    // Assert
    assert_eq!(withdraws.len(), 1);
    assert_eq!(deposits.len(), 1);
    assert_eq!(
        withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(
            UnspecifiedResources::NonePresent
        )
        .set(XRD, ResourceBounds::exact_amount(10).unwrap())]),
    );
}

#[test]
fn simple_account_transfer_with_an_explicit_take_exact_is_correctly_classified() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let manifest = ManifestBuilder::new_v2()
        .lock_fee_and_withdraw(account1, 100, XRD, 10)
        .take_from_worktop(XRD, 10, "bucket")
        .deposit(account2, "bucket")
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest).unwrap();

    // Assert
    assert_eq!(withdraws.len(), 1);
    assert_eq!(deposits.len(), 1);
    assert_eq!(
        withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(
            UnspecifiedResources::NonePresent
        )
        .set(XRD, ResourceBounds::exact_amount(10).unwrap())]),
    );
}

#[test]
fn simple_account_transfer_with_two_deposits_is_correctly_classified() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let manifest = ManifestBuilder::new_v2()
        .lock_fee_and_withdraw(account1, 100, XRD, 10)
        .take_from_worktop(XRD, 2, "bucket")
        .take_all_from_worktop(XRD, "bucket2")
        .deposit(account2, "bucket2")
        .deposit(account2, "bucket")
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest).unwrap();

    // Assert
    assert_eq!(withdraws.len(), 1);
    assert_eq!(deposits.len(), 1);
    assert_eq!(
        withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        deposits.get(&account2),
        Some(&vec![
            AccountDeposit::empty(UnspecifiedResources::NonePresent)
                .set(XRD, ResourceBounds::exact_amount(8).unwrap()),
            AccountDeposit::empty(UnspecifiedResources::NonePresent)
                .set(XRD, ResourceBounds::exact_amount(2).unwrap()),
        ]),
    );
}

#[test]
fn simple_account_transfer_with_a_take_all_is_correctly_classified() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let manifest = ManifestBuilder::new_v2()
        .lock_fee_and_withdraw(account1, 100, XRD, 10)
        .take_all_from_worktop(XRD, "bucket")
        .deposit(account2, "bucket")
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest).unwrap();

    // Assert
    assert_eq!(withdraws.len(), 1);
    assert_eq!(deposits.len(), 1);
    assert_eq!(
        withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(
            UnspecifiedResources::NonePresent
        )
        .set(XRD, ResourceBounds::exact_amount(10).unwrap()),]),
    );
}

#[test]
fn simple_account_transfer_deposit_batch_is_correctly_classified() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let manifest = ManifestBuilder::new_subintent_v2()
        .lock_fee_and_withdraw(account1, 100, XRD, 10)
        .deposit_batch(account2, ManifestExpression::EntireWorktop)
        .yield_to_parent(())
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest).unwrap();

    // Assert
    assert_eq!(withdraws.len(), 1);
    assert_eq!(deposits.len(), 1);
    assert_eq!(
        withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::some([
            ChangeSource::InitialYieldFromParent
        ]))
        .set(XRD, ResourceBounds::at_least_amount(10).unwrap()),]),
    );
}

#[test]
fn simple_account_transfer_of_non_fungibles_by_amount_is_classified_correctly() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let non_fungible_address = non_fungible_resource_address(1);
    let manifest = ManifestBuilder::new_subintent_v2()
        .lock_fee_and_withdraw(account1, 100, non_fungible_address, 10)
        .deposit_batch(account2, ManifestExpression::EntireWorktop)
        .yield_to_parent(())
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest).unwrap();

    // Assert
    assert_eq!(withdraws.len(), 1);
    assert_eq!(deposits.len(), 1);
    assert_eq!(
        withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(
            non_fungible_address,
            10.into()
        )])
    );
    assert_eq!(
        deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::some([
            ChangeSource::InitialYieldFromParent
        ]))
        .set(
            non_fungible_address,
            ResourceBounds::at_least_amount(10).unwrap()
        ),]),
    );
}

#[test]
fn simple_account_transfer_of_non_fungibles_by_ids_is_classified_correctly() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let non_fungible_address = non_fungible_resource_address(1);
    let manifest = ManifestBuilder::new_v2()
        .lock_fee_and_withdraw_non_fungibles(
            account1,
            100,
            non_fungible_address,
            [NonFungibleLocalId::integer(1)],
        )
        .deposit_batch(account2, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest).unwrap();

    // Assert
    assert_eq!(withdraws.len(), 1);
    assert_eq!(deposits.len(), 1);
    assert_eq!(
        withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Ids(
            non_fungible_address,
            [NonFungibleLocalId::integer(1)].into_iter().collect(),
        )])
    );
    assert_eq!(
        deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(
            UnspecifiedResources::NonePresent
        )
        .set(
            non_fungible_address,
            ResourceBounds::exact_non_fungibles([NonFungibleLocalId::integer(1)]),
        ),]),
    );
}

#[test]
fn assertion_of_any_with_nothing_on_worktop_does_nothing() {
    // Arrange
    let account = account_address(1);
    let manifest = ManifestBuilder::new_v2()
        .assert_worktop_contains_any(XRD)
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let error = statically_analyze(&manifest).unwrap_err();

    // Assert
    assert_eq!(
        error,
        StaticResourceMovementsError::AssertionCannotBeSatisfied
    );
}

#[test]
fn assertion_of_any_with_unknown_on_worktop_gives_context_to_visitor() {
    // Arrange
    let account = account_address(1);
    let manifest = ManifestBuilder::new_v2()
        .call_method(component_address(1), "random", ())
        .call_method(component_address(1), "random2", ())
        .assert_worktop_contains_any(XRD)
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest).unwrap();

    // Assert
    assert_eq!(withdraws.len(), 0);
    assert_eq!(deposits.len(), 1);
    assert_eq!(withdraws.get(&account), None);
    assert_eq!(
        deposits.get(&account),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::some([
            ChangeSource::invocation_at(0),
            ChangeSource::invocation_at(1),
        ]))
        .set(XRD, ResourceBounds::non_zero()),]),
    );
}

#[test]
fn assertion_of_ids_gives_context_to_visitor() {
    // Arrange
    let account = account_address(1);
    let non_fungible_address = non_fungible_resource_address(1);
    let manifest = ManifestBuilder::new_v2()
        .call_method(component_address(1), "random", ())
        .assert_worktop_contains_non_fungibles(
            non_fungible_address,
            [NonFungibleLocalId::integer(1)],
        )
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest).unwrap();

    // Assert
    assert_eq!(withdraws.len(), 0);
    assert_eq!(deposits.len(), 1);
    assert_eq!(withdraws.get(&account), None);
    assert_eq!(
        deposits.get(&account),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::some([
            ChangeSource::invocation_at(0),
        ]))
        .set(
            non_fungible_address,
            ResourceBounds::at_least_non_fungibles([NonFungibleLocalId::integer(1),]),
        ),]),
    );
}

#[test]
fn complex_assertion_of_amount_gives_context_to_visitor() {
    // Arrange
    let account = account_address(1);
    let resource_address = fungible_resource_address(5);
    let resource_address2 = fungible_resource_address(8);
    let builder = ManifestBuilder::new_v2();
    let lookup = builder.name_lookup();
    let manifest = builder
        .call_method(component_address(1), "random", ())
        .assert_worktop_contains(resource_address, 10)
        .assert_worktop_contains(resource_address2, 5)
        .take_from_worktop(resource_address, 10, "bucket")
        .take_from_worktop(resource_address2, 7, "bucket2")
        .deposit_batch(account, [lookup.bucket("bucket")])
        .assert_worktop_is_empty()
        .return_to_worktop("bucket2")
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest).unwrap();

    // Assert
    assert_eq!(withdraws.len(), 0);
    assert_eq!(deposits.len(), 1);
    assert_eq!(withdraws.get(&account), None);
    assert_eq!(
        deposits.get(&account),
        Some(&vec![
            AccountDeposit::empty(UnspecifiedResources::none())
                .set(resource_address, ResourceBounds::exact_amount(10).unwrap()),
            AccountDeposit::empty(UnspecifiedResources::none())
                .set(resource_address2, ResourceBounds::exact_amount(7).unwrap()),
        ]),
    );
}

#[test]
fn two_buckets_with_separate_histories_are_combined() {
    // Arrange
    let account = account_address(1);
    let resource_address = fungible_resource_address(5);
    let builder = ManifestBuilder::new_v2();
    let lookup = builder.name_lookup();
    let manifest = builder
        .call_method(component_address(1), "unknown_method", (1,))
        .assert_worktop_contains(resource_address, 5)
        .take_from_worktop(resource_address, 2, "call1_2")
        .take_all_from_worktop(resource_address, "call1_remainder")
        .call_method(component_address(1), "unknown_method", (2,))
        .assert_worktop_contains_any(resource_address)
        .return_to_worktop("call1_remainder")
        .take_all_from_worktop(resource_address, "total")
        .deposit_batch(account, [lookup.bucket("total"), lookup.bucket("call1_2")])
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest).unwrap();

    // Assert
    assert_eq!(withdraws.len(), 0);
    assert_eq!(deposits.len(), 1);
    assert_eq!(withdraws.get(&account), None);
    assert_eq!(
        deposits.get(&account),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::none())
            .set(
                resource_address,
                ResourceBounds::at_least_amount(5).unwrap()
            ),]),
    );
}

fn account_address(id: u64) -> ComponentAddress {
    unsafe {
        ComponentAddress::new_unchecked(node_id(EntityType::GlobalPreallocatedEd25519Account, id).0)
    }
}

fn component_address(id: u64) -> ComponentAddress {
    unsafe { ComponentAddress::new_unchecked(node_id(EntityType::GlobalGenericComponent, id).0) }
}

fn fungible_resource_address(id: u64) -> ResourceAddress {
    unsafe {
        ResourceAddress::new_unchecked(node_id(EntityType::GlobalFungibleResourceManager, id).0)
    }
}

fn non_fungible_resource_address(id: u64) -> ResourceAddress {
    unsafe {
        ResourceAddress::new_unchecked(node_id(EntityType::GlobalNonFungibleResourceManager, id).0)
    }
}

fn node_id(entity_type: EntityType, id: u64) -> NodeId {
    let mut bytes = hash(id.to_be_bytes()).lower_bytes::<{ NodeId::LENGTH }>();
    bytes[0] = entity_type as u8;
    NodeId(bytes)
}

fn statically_analyze<M: ReadableManifest>(
    manifest: &M,
) -> Result<
    (
        IndexMap<ComponentAddress, Vec<AccountDeposit>>,
        IndexMap<ComponentAddress, Vec<AccountWithdraw>>,
    ),
    StaticResourceMovementsError,
> {
    let interpreter = StaticManifestInterpreter::new(ValidationRuleset::all(), manifest);
    let mut visitor: StaticResourceMovementsVisitor =
        StaticResourceMovementsVisitor::new(manifest.is_subintent());
    interpreter.validate_and_apply_visitor(&mut visitor)?;
    let output = visitor.output();
    Ok((output.account_deposits(), output.account_withdraws()))
}
