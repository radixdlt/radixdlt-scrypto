use radix_common::prelude::*;
use radix_transactions::manifest::static_resource_movements_visitor::*;
use radix_transactions::manifest::*;
use radix_transactions::prelude::*;

#[test]
fn simple_account_transfer_with_an_explicit_take_is_correctly_classified() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let manifest = ManifestV2Builder::new_typed()
        .lock_fee_and_withdraw(account1, 100, XRD, 10)
        .take_from_worktop(XRD, 10, "bucket")
        .deposit(account2, "bucket")
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest);

    // Assert
    assert_eq!(withdraws.len(), 1);
    assert_eq!(deposits.len(), 1);
    assert_eq!(
        withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        deposits.get(&account2),
        Some(&vec![AccountDeposit::KnownFungible(
            XRD,
            FungibleBounds {
                lower: LowerFungibleBound::Amount(10.into()),
                upper: UpperFungibleBound::Amount(10.into())
            }
        )])
    );
}

#[test]
fn simple_account_transfer_with_a_take_all_is_correctly_classified() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let manifest = ManifestV2Builder::new_typed()
        .lock_fee_and_withdraw(account1, 100, XRD, 10)
        .take_all_from_worktop(XRD, "bucket")
        .deposit(account2, "bucket")
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest);

    // Assert
    assert_eq!(withdraws.len(), 1);
    assert_eq!(deposits.len(), 1);
    assert_eq!(
        withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        deposits.get(&account2),
        Some(&vec![AccountDeposit::KnownFungible(
            XRD,
            FungibleBounds {
                lower: LowerFungibleBound::Amount(10.into()),
                upper: UpperFungibleBound::Amount(10.into())
            }
        )])
    );
}

#[test]
fn simple_account_transfer_deposit_batch_is_correctly_classified() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let manifest = ManifestV2Builder::new_typed()
        .lock_fee_and_withdraw(account1, 100, XRD, 10)
        .deposit_batch(account2, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest);

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
            AccountDeposit::KnownFungible(
                XRD,
                FungibleBounds {
                    lower: LowerFungibleBound::Amount(10.into()),
                    upper: UpperFungibleBound::Amount(10.into())
                }
            ),
            AccountDeposit::Unknown(WorktopUncertaintySource::YieldFromParent)
        ])
    );
}

#[test]
fn simple_account_transfer_of_non_fungibles_by_amount_is_classified_correctly() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let non_fungible_address = non_fungible_resource_address(1);
    let manifest = ManifestV2Builder::new_typed()
        .lock_fee_and_withdraw(account1, 100, non_fungible_address, 10)
        .deposit_batch(account2, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest);

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
        Some(&vec![
            AccountDeposit::KnownNonFungible(
                non_fungible_address,
                NonFungibleBounds {
                    amount_bounds: FungibleBounds::new_exact(10.into()),
                    id_bounds: NonFungibleIdBounds::Unknown
                }
            ),
            AccountDeposit::Unknown(WorktopUncertaintySource::YieldFromParent)
        ])
    );
}

#[test]
fn simple_account_transfer_of_non_fungibles_by_ids_is_classified_correctly() {
    // Arrange
    let account1 = account_address(1);
    let account2 = account_address(2);
    let non_fungible_address = non_fungible_resource_address(1);
    let manifest = ManifestV2Builder::new_typed()
        .lock_fee_and_withdraw_non_fungibles(
            account1,
            100,
            non_fungible_address,
            indexset! {
                NonFungibleLocalId::integer(1)
            },
        )
        .deposit_batch(account2, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest);

    // Assert
    assert_eq!(withdraws.len(), 1);
    assert_eq!(deposits.len(), 1);
    assert_eq!(
        withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Ids(
            non_fungible_address,
            indexset! { NonFungibleLocalId::integer(1) }
        )])
    );
    assert_eq!(
        deposits.get(&account2),
        Some(&vec![
            AccountDeposit::KnownNonFungible(
                non_fungible_address,
                NonFungibleBounds {
                    amount_bounds: FungibleBounds::new_exact(1.into()),
                    id_bounds: NonFungibleIdBounds::FullyKnown(indexset! {
                        NonFungibleLocalId::integer(1)
                    })
                }
            ),
            AccountDeposit::Unknown(WorktopUncertaintySource::YieldFromParent)
        ])
    );
}

#[test]
fn assertion_of_any_gives_context_to_visitor() {
    // Arrange
    let account = account_address(1);
    let manifest = ManifestV2Builder::new_typed()
        .assert_worktop_contains_any(XRD)
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest);

    // Assert
    assert_eq!(withdraws.len(), 0);
    assert_eq!(deposits.len(), 1);
    assert_eq!(withdraws.get(&account), None);
    assert_eq!(
        deposits.get(&account),
        Some(&vec![
            AccountDeposit::KnownFungible(
                XRD,
                FungibleBounds {
                    lower: LowerFungibleBound::NonZero,
                    upper: UpperFungibleBound::Unbounded
                }
            ),
            AccountDeposit::Unknown(WorktopUncertaintySource::YieldFromParent)
        ])
    );
}

#[test]
fn assertion_of_ids_gives_context_to_visitor() {
    // Arrange
    let account = account_address(1);
    let non_fungible_address = non_fungible_resource_address(1);
    let manifest = ManifestV2Builder::new_typed()
        .assert_worktop_contains_non_fungibles(
            non_fungible_address,
            indexset! {
                NonFungibleLocalId::integer(1)
            },
        )
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest);

    // Assert
    assert_eq!(withdraws.len(), 0);
    assert_eq!(deposits.len(), 1);
    assert_eq!(withdraws.get(&account), None);
    assert_eq!(
        deposits.get(&account),
        Some(&vec![
            AccountDeposit::KnownNonFungible(
                non_fungible_address,
                NonFungibleBounds {
                    amount_bounds: FungibleBounds {
                        lower: LowerFungibleBound::Amount(1.into()),
                        upper: UpperFungibleBound::Unbounded,
                    },
                    id_bounds: NonFungibleIdBounds::PartiallyKnown(indexset! {
                        NonFungibleLocalId::integer(1)
                    })
                }
            ),
            AccountDeposit::Unknown(WorktopUncertaintySource::YieldFromParent)
        ])
    );
}

#[test]
fn assertion_of_amount_gives_context_to_visitor() {
    // Arrange
    let account = account_address(1);
    let manifest = ManifestV2Builder::new_typed()
        .assert_worktop_contains(XRD, 10)
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (deposits, withdraws) = statically_analyze(&manifest);

    // Assert
    assert_eq!(withdraws.len(), 0);
    assert_eq!(deposits.len(), 1);
    assert_eq!(withdraws.get(&account), None);
    assert_eq!(
        deposits.get(&account),
        Some(&vec![
            AccountDeposit::KnownFungible(
                XRD,
                FungibleBounds {
                    lower: LowerFungibleBound::Amount(10.into()),
                    upper: UpperFungibleBound::Unbounded
                }
            ),
            AccountDeposit::Unknown(WorktopUncertaintySource::YieldFromParent)
        ])
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
) -> (
    IndexMap<ComponentAddress, Vec<AccountDeposit>>,
    IndexMap<ComponentAddress, Vec<AccountWithdraw>>,
) {
    let interpreter = StaticManifestInterpreter::new(ValidationRuleset::v1(), manifest);
    let mut visitor = StaticResourceMovementsVisitor::new(true);
    interpreter.interpret_or_err(&mut visitor).expect("Error");
    visitor.output()
}
