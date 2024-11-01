use radix_transactions::manifest::static_resource_movements::*;
use radix_transactions::manifest::*;
use radix_transactions::prelude::*;
use scrypto_test::prelude::*;

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
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 1);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(
        all_withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        all_deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(
            UnspecifiedResources::NonePresent
        )
        .set(XRD, ResourceBounds::exact_amount(10).unwrap())]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 1);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(
        net_withdraws.get(&account1),
        Some(&NetWithdraws::empty().set_fungible(XRD, 10))
    );
    assert_eq!(
        net_deposits.get(&account2),
        Some(
            &NetDeposits::empty(UnspecifiedResources::NonePresent)
                .set(XRD, ResourceBounds::exact_amount(10).unwrap())
        ),
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
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 1);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(
        all_withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        all_deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(
            UnspecifiedResources::NonePresent
        )
        .set(XRD, ResourceBounds::exact_amount(10).unwrap())]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 1);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(
        net_withdraws.get(&account1),
        Some(&NetWithdraws::empty().set_fungible(XRD, 10))
    );
    assert_eq!(
        net_deposits.get(&account2),
        Some(
            &NetDeposits::empty(UnspecifiedResources::NonePresent)
                .set(XRD, ResourceBounds::exact_amount(10).unwrap())
        ),
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
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 1);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(
        all_withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        all_deposits.get(&account2),
        Some(&vec![
            AccountDeposit::empty(UnspecifiedResources::NonePresent)
                .set(XRD, ResourceBounds::exact_amount(8).unwrap()),
            AccountDeposit::empty(UnspecifiedResources::NonePresent)
                .set(XRD, ResourceBounds::exact_amount(2).unwrap()),
        ]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 1);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(
        net_withdraws.get(&account1),
        Some(&NetWithdraws::empty().set_fungible(XRD, 10))
    );
    assert_eq!(
        net_deposits.get(&account2),
        Some(
            &NetDeposits::empty(UnspecifiedResources::NonePresent)
                .set(XRD, ResourceBounds::exact_amount(10).unwrap()),
        ),
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
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 1);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(
        all_withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        all_deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(
            UnspecifiedResources::NonePresent
        )
        .set(XRD, ResourceBounds::exact_amount(10).unwrap()),]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 1);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(
        net_withdraws.get(&account1),
        Some(&NetWithdraws::empty().set_fungible(XRD, 10))
    );
    assert_eq!(
        net_deposits.get(&account2),
        Some(
            &NetDeposits::empty(UnspecifiedResources::NonePresent)
                .set(XRD, ResourceBounds::exact_amount(10).unwrap()),
        ),
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
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 1);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(
        all_withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(XRD, 10.into())])
    );
    assert_eq!(
        all_deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::some([
            ChangeSource::InitialYieldFromParent
        ]))
        .set(XRD, ResourceBounds::at_least_amount(10).unwrap()),]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 1);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(
        net_withdraws.get(&account1),
        Some(&NetWithdraws::empty().set_fungible(XRD, 10))
    );
    assert_eq!(
        net_deposits.get(&account2),
        Some(
            &NetDeposits::empty(UnspecifiedResources::some([
                ChangeSource::InitialYieldFromParent
            ]))
            .set(XRD, ResourceBounds::at_least_amount(10).unwrap()),
        ),
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
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 1);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(
        all_withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Amount(
            non_fungible_address,
            10.into()
        )])
    );
    assert_eq!(
        all_deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::some([
            ChangeSource::InitialYieldFromParent
        ]))
        .set(
            non_fungible_address,
            ResourceBounds::at_least_amount(10).unwrap()
        ),]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 1);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(
        net_withdraws.get(&account1),
        Some(&NetWithdraws::empty().set_non_fungible(non_fungible_address, [], 10))
    );
    assert_eq!(
        net_deposits.get(&account2),
        Some(
            &NetDeposits::empty(UnspecifiedResources::some([
                ChangeSource::InitialYieldFromParent
            ]))
            .set(
                non_fungible_address,
                ResourceBounds::at_least_amount(10).unwrap()
            )
        ),
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
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 1);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(
        all_withdraws.get(&account1),
        Some(&vec![AccountWithdraw::Ids(
            non_fungible_address,
            [NonFungibleLocalId::integer(1)].into_iter().collect(),
        )])
    );
    assert_eq!(
        all_deposits.get(&account2),
        Some(&vec![AccountDeposit::empty(
            UnspecifiedResources::NonePresent
        )
        .set(
            non_fungible_address,
            ResourceBounds::exact_non_fungibles([NonFungibleLocalId::integer(1)]),
        ),]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 1);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(
        net_withdraws.get(&account1),
        Some(&NetWithdraws::empty().set_non_fungible(
            non_fungible_address,
            [NonFungibleLocalId::integer(1)],
            0,
        ))
    );
    assert_eq!(
        net_deposits.get(&account2),
        Some(&NetDeposits::empty(UnspecifiedResources::NonePresent).set(
            non_fungible_address,
            ResourceBounds::exact_non_fungibles([NonFungibleLocalId::integer(1)]),
        )),
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
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 0);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(all_withdraws.get(&account), None);
    assert_eq!(
        all_deposits.get(&account),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::some([
            ChangeSource::invocation_at(0),
            ChangeSource::invocation_at(1),
        ]))
        .set(XRD, ResourceBounds::non_zero()),]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 0);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(net_withdraws.get(&account), None);
    assert_eq!(
        net_deposits.get(&account),
        Some(
            &NetDeposits::empty(UnspecifiedResources::some([
                ChangeSource::invocation_at(0),
                ChangeSource::invocation_at(1),
            ]))
            .set(XRD, ResourceBounds::non_zero())
        ),
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
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 0);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(all_withdraws.get(&account), None);
    assert_eq!(
        all_deposits.get(&account),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::some([
            ChangeSource::invocation_at(0),
        ]))
        .set(
            non_fungible_address,
            ResourceBounds::at_least_non_fungibles([NonFungibleLocalId::integer(1),]),
        ),]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 0);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(net_withdraws.get(&account), None);
    assert_eq!(
        net_deposits.get(&account),
        Some(
            &NetDeposits::empty(UnspecifiedResources::some(
                [ChangeSource::invocation_at(0),]
            ))
            .set(
                non_fungible_address,
                ResourceBounds::at_least_non_fungibles([NonFungibleLocalId::integer(1),]),
            )
        ),
    );
}

#[test]
fn assertion_of_next_call_returns_only_constrains_resources() {
    // Arrange
    let account = account_address(1);
    let non_fungible_address = non_fungible_resource_address(1);
    let manifest = ManifestBuilder::new_v2()
        .assert_next_call_returns_only(
            ManifestResourceConstraints::new()
                .with_amount_range(XRD, 5, 10)
                .with_at_least_non_fungibles(
                    non_fungible_address,
                    [NonFungibleLocalId::integer(3)],
                ),
        )
        .call_method(component_address(1), "random", ())
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 0);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(all_withdraws.get(&account), None);
    assert_eq!(
        all_deposits.get(&account),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::none())
            .set(XRD, ResourceBounds::general_fungible(5, 10).unwrap())
            .set(
                non_fungible_address,
                ResourceBounds::at_least_non_fungibles([NonFungibleLocalId::integer(3)])
            ),]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 0);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(net_withdraws.get(&account), None);
    assert_eq!(
        net_deposits.get(&account),
        Some(
            &NetDeposits::empty(UnspecifiedResources::none())
                .set(XRD, ResourceBounds::general_fungible(5, 10).unwrap())
                .set(
                    non_fungible_address,
                    ResourceBounds::at_least_non_fungibles([NonFungibleLocalId::integer(3)])
                ),
        ),
    );
}

#[test]
fn assertion_of_next_call_returns_include_constrains_resources() {
    // Arrange
    let account = account_address(1);
    let non_fungible_address = non_fungible_resource_address(1);
    let manifest = ManifestBuilder::new_v2()
        .assert_next_call_returns_include(
            ManifestResourceConstraints::new()
                .with_amount_range(XRD, 5, 10)
                .with_at_least_non_fungibles(
                    non_fungible_address,
                    [NonFungibleLocalId::integer(3)],
                ),
        )
        .call_method(component_address(1), "random", ())
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 0);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(all_withdraws.get(&account), None);
    assert_eq!(
        all_deposits.get(&account),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::some([
            ChangeSource::invocation_at(1)
        ]))
        .set(XRD, ResourceBounds::general_fungible(5, 10).unwrap())
        .set(
            non_fungible_address,
            ResourceBounds::at_least_non_fungibles([NonFungibleLocalId::integer(3)])
        )]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 0);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(net_withdraws.get(&account), None);
    assert_eq!(
        net_deposits.get(&account),
        Some(
            &NetDeposits::empty(UnspecifiedResources::some([ChangeSource::invocation_at(1)]))
                .set(XRD, ResourceBounds::general_fungible(5, 10).unwrap())
                .set(
                    non_fungible_address,
                    ResourceBounds::at_least_non_fungibles([NonFungibleLocalId::integer(3)])
                )
        ),
    );
}

#[test]
fn assertion_of_worktop_resources_only_constrains_resources() {
    // Arrange
    let account = account_address(1);
    let non_fungible_address = non_fungible_resource_address(1);
    let manifest = ManifestBuilder::new_v2()
        .call_method(component_address(1), "random", ())
        .assert_worktop_resources_only(
            ManifestResourceConstraints::new()
                .with_exact_amount(XRD, "9.452")
                .with_exact_non_fungibles(non_fungible_address, [NonFungibleLocalId::integer(3)]),
        )
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 0);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(all_withdraws.get(&account), None);
    assert_eq!(
        all_deposits.get(&account),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::none())
            .set(XRD, ResourceBounds::exact_amount("9.452").unwrap())
            .set(
                non_fungible_address,
                ResourceBounds::exact_non_fungibles([NonFungibleLocalId::integer(3)])
            ),]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 0);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(net_withdraws.get(&account), None);
    assert_eq!(
        net_deposits.get(&account),
        Some(
            &NetDeposits::empty(UnspecifiedResources::none())
                .set(XRD, ResourceBounds::exact_amount("9.452").unwrap())
                .set(
                    non_fungible_address,
                    ResourceBounds::exact_non_fungibles([NonFungibleLocalId::integer(3)])
                ),
        ),
    );
}

#[test]
fn assertion_of_worktop_resources_include_constrains_resources() {
    // Arrange
    let account = account_address(1);
    let non_fungible_address = non_fungible_resource_address(1);
    let manifest = ManifestBuilder::new_v2()
        .call_method(component_address(1), "random", ())
        .assert_worktop_resources_include(
            ManifestResourceConstraints::new()
                .with_exact_amount(XRD, "9.452")
                .with_exact_non_fungibles(non_fungible_address, [NonFungibleLocalId::integer(3)]),
        )
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    // Act
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 0);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(all_withdraws.get(&account), None);
    assert_eq!(
        all_deposits.get(&account),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::some([
            ChangeSource::invocation_at(0)
        ]))
        .set(XRD, ResourceBounds::exact_amount("9.452").unwrap())
        .set(
            non_fungible_address,
            ResourceBounds::exact_non_fungibles([NonFungibleLocalId::integer(3)])
        ),]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 0);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(net_withdraws.get(&account), None);
    assert_eq!(
        net_deposits.get(&account),
        Some(
            &NetDeposits::empty(UnspecifiedResources::some([ChangeSource::invocation_at(0)]))
                .set(XRD, ResourceBounds::exact_amount("9.452").unwrap())
                .set(
                    non_fungible_address,
                    ResourceBounds::exact_non_fungibles([NonFungibleLocalId::integer(3)])
                )
        ),
    );
}

#[test]
fn assertion_of_bucket_constrains_resources() {
    // Arrange
    let account = account_address(1);
    let non_fungible_address = non_fungible_resource_address(1);
    let builder = ManifestBuilder::new_v2();
    let lookup = builder.name_lookup();
    let manifest = builder
        .call_method(component_address(1), "random", ())
        .take_all_from_worktop(non_fungible_address, "my_bucket")
        .assert_bucket_contents(
            "my_bucket",
            ManifestResourceConstraint::General(GeneralResourceConstraint {
                required_ids: indexset!(NonFungibleLocalId::string("hello").unwrap(),),
                lower_bound: LowerBound::Inclusive(dec!(1)),
                upper_bound: UpperBound::Inclusive(dec!(2)),
                allowed_ids: AllowedIds::Allowlist(indexset!(
                    NonFungibleLocalId::string("hello").unwrap(),
                    NonFungibleLocalId::string("world").unwrap(),
                    NonFungibleLocalId::string("it_is").unwrap(),
                    NonFungibleLocalId::string("me").unwrap(),
                )),
            }),
        )
        .deposit_batch(account, [lookup.bucket("my_bucket")])
        .build();

    // Act
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 0);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(all_withdraws.get(&account), None);
    assert_eq!(
        all_deposits.get(&account),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::none())
            .set(
                non_fungible_address,
                ResourceBounds::general_non_fungible_with_allowlist(
                    [NonFungibleLocalId::string("hello").unwrap()],
                    1,
                    2,
                    [
                        NonFungibleLocalId::string("hello").unwrap(),
                        NonFungibleLocalId::string("world").unwrap(),
                        NonFungibleLocalId::string("it_is").unwrap(),
                        NonFungibleLocalId::string("me").unwrap(),
                    ]
                )
                .unwrap()
            ),]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 0);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(net_withdraws.get(&account), None);
    assert_eq!(
        net_deposits.get(&account),
        Some(
            &NetDeposits::empty(UnspecifiedResources::none()).set(
                non_fungible_address,
                ResourceBounds::general_non_fungible_with_allowlist(
                    [NonFungibleLocalId::string("hello").unwrap()],
                    1,
                    2,
                    [
                        NonFungibleLocalId::string("hello").unwrap(),
                        NonFungibleLocalId::string("world").unwrap(),
                        NonFungibleLocalId::string("it_is").unwrap(),
                        NonFungibleLocalId::string("me").unwrap(),
                    ]
                )
                .unwrap()
            ),
        ),
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
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 0);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(all_withdraws.get(&account), None);
    assert_eq!(
        all_deposits.get(&account),
        Some(&vec![
            AccountDeposit::empty(UnspecifiedResources::none())
                .set(resource_address, ResourceBounds::exact_amount(10).unwrap()),
            AccountDeposit::empty(UnspecifiedResources::none())
                .set(resource_address2, ResourceBounds::exact_amount(7).unwrap()),
        ]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 0);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(net_withdraws.get(&account), None);
    assert_eq!(
        net_deposits.get(&account),
        Some(
            &NetDeposits::empty(UnspecifiedResources::none())
                .set(resource_address, ResourceBounds::exact_amount(10).unwrap())
                .set(resource_address2, ResourceBounds::exact_amount(7).unwrap()),
        ),
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
    let (all_deposits, all_withdraws, net_deposits, net_withdraws) =
        statically_analyze(&manifest).unwrap();

    // Assert All
    assert_eq!(all_withdraws.len(), 0);
    assert_eq!(all_deposits.len(), 1);
    assert_eq!(all_withdraws.get(&account), None);
    assert_eq!(
        all_deposits.get(&account),
        Some(&vec![AccountDeposit::empty(UnspecifiedResources::none())
            .set(
                resource_address,
                ResourceBounds::at_least_amount(5).unwrap()
            ),]),
    );

    // Assert Net
    assert_eq!(net_withdraws.len(), 0);
    assert_eq!(net_deposits.len(), 1);
    assert_eq!(net_withdraws.get(&account), None);
    assert_eq!(
        net_deposits.get(&account),
        Some(&NetDeposits::empty(UnspecifiedResources::none()).set(
            resource_address,
            ResourceBounds::at_least_amount(5).unwrap()
        )),
    );
}

#[test]
fn aggregation_balance_change_test_cases() {
    // Arrange
    let account = account_address(1);
    let nf_resource_address = non_fungible_resource_address(5);
    let nf_local_id = NonFungibleLocalId::integer(5);
    let nf_global_id = NonFungibleGlobalId::new(nf_resource_address, nf_local_id.clone());

    {
        // `Deposit {#5#}, Withdraw 1` => `Deposited 1, Withdrawn 1` (because we can't guarantee we deposited #5#)
        let manifest = ManifestBuilder::new_subintent_v2()
            .take_non_fungibles_from_worktop(nf_resource_address, [nf_local_id.clone()], "bucket")
            .deposit(account, "bucket")
            .assert_worktop_is_empty()
            .withdraw_from_account(account, nf_resource_address, 1)
            .yield_to_parent((ManifestExpression::EntireWorktop,))
            .build();

        let (_, _, net_deposits, net_withdraws) = statically_analyze(&manifest).unwrap();
        let expected_withdraws = NetWithdraws::empty().set_non_fungible(nf_resource_address, [], 1);
        let expected_deposits = NetDeposits::empty(UnspecifiedResources::NonePresent).set(
            nf_resource_address,
            ResourceBounds::exact_amount(1).unwrap(),
        );
        assert_eq!(net_deposits.get(&account), Some(&expected_deposits));
        assert_eq!(net_withdraws.get(&account), Some(&expected_withdraws));
    }
    {
        // `Withdraw 1, Deposit {#5#}` => `Deposited {#5#}, Withdrawn 1` (order matters!)
        let manifest = ManifestBuilder::new_subintent_v2()
            .withdraw_from_account(account, nf_resource_address, 1)
            .take_non_fungibles_from_worktop(nf_resource_address, [nf_local_id.clone()], "bucket")
            .deposit(account, "bucket")
            .yield_to_parent((ManifestExpression::EntireWorktop,))
            .build();

        let (_, _, net_deposits, net_withdraws) = statically_analyze(&manifest).unwrap();
        let expected_withdraws = NetWithdraws::empty().set_non_fungible(nf_resource_address, [], 1);
        let expected_deposits = NetDeposits::empty(UnspecifiedResources::NonePresent).set(
            nf_resource_address,
            ResourceBounds::exact_non_fungibles([nf_local_id.clone()]),
        );
        assert_eq!(net_deposits.get(&account), Some(&expected_deposits));
        assert_eq!(net_withdraws.get(&account), Some(&expected_withdraws));
    }
    {
        // `Withdraw {#5#}, Deposit 1` => `Deposited 1, Withdrawn 1`
        let manifest = ManifestBuilder::new_subintent_v2()
            .withdraw_non_fungibles_from_account(
                account,
                nf_resource_address,
                [nf_local_id.clone()],
            )
            .take_from_worktop(nf_resource_address, 1, "bucket")
            .deposit(account, "bucket")
            .yield_to_parent((ManifestExpression::EntireWorktop,))
            .build();

        let (_, _, net_deposits, net_withdraws) = statically_analyze(&manifest).unwrap();
        let expected_withdraws = NetWithdraws::empty().set_non_fungible(nf_resource_address, [], 1);
        let expected_deposits = NetDeposits::empty(UnspecifiedResources::NonePresent).set(
            nf_resource_address,
            ResourceBounds::exact_amount(1).unwrap(),
        );
        assert_eq!(net_deposits.get(&account), Some(&expected_deposits));
        assert_eq!(net_withdraws.get(&account), Some(&expected_withdraws));
    }
    {
        // `Deposit 1, Withdraw {#5#}` => `Deposited 1, Withdrawn {#5#}`
        let manifest = ManifestBuilder::new_subintent_v2()
            .take_from_worktop(nf_resource_address, 1, "bucket")
            .deposit(account, "bucket")
            .withdraw_non_fungibles_from_account(
                account,
                nf_resource_address,
                [nf_local_id.clone()],
            )
            .yield_to_parent((ManifestExpression::EntireWorktop,))
            .build();

        let (_, _, net_deposits, net_withdraws) = statically_analyze(&manifest).unwrap();
        let expected_withdraws =
            NetWithdraws::empty().set_non_fungible(nf_resource_address, [nf_local_id.clone()], 0);
        let expected_deposits = NetDeposits::empty(UnspecifiedResources::NonePresent).set(
            nf_resource_address,
            ResourceBounds::exact_amount(1).unwrap(),
        );
        assert_eq!(net_deposits.get(&account), Some(&expected_deposits));
        assert_eq!(net_withdraws.get(&account), Some(&expected_withdraws));
    }
    {
        // `Withdraw {#2}, Deposit {#2#}, Withdraw {#2}` => `Withdraw {#2}`
        // Withdraw, Deposit, Withdraw of a single NF id flattens to a single withdraw
        let manifest = ManifestBuilder::new_subintent_v2()
            .assert_worktop_is_empty()
            .withdraw_non_fungible_from_account(account, nf_global_id.clone())
            .deposit_entire_worktop(account)
            .withdraw_non_fungible_from_account(account, nf_global_id)
            .yield_to_parent((ManifestExpression::EntireWorktop,))
            .build();

        let (_, _, net_deposits, net_withdraws) = statically_analyze(&manifest).unwrap();

        let expected_withdraws =
            NetWithdraws::empty().set_non_fungible(nf_resource_address, [nf_local_id], 0);
        assert_eq!(net_deposits.get(&account), None);
        assert_eq!(net_withdraws.get(&account), Some(&expected_withdraws));
    }

    {
        // `Withdraw 1, Deposit between 3 and 7, Withdraw 4` => `Deposited between 3 and 7, Withdrawn 5`
        // Withdraw, Deposit, Withdraw of fungible resource does NOT flatten
        let manifest = ManifestBuilder::new_subintent_v2()
            .assert_worktop_resources_only(
                ManifestResourceConstraints::new().with_amount_range(XRD, 2, 6),
            )
            .withdraw_from_account(account, XRD, 1)
            .deposit_entire_worktop(account)
            .withdraw_from_account(account, XRD, 4)
            .yield_to_parent((ManifestExpression::EntireWorktop,))
            .build();

        let (_, _, net_deposits, net_withdraws) = statically_analyze(&manifest).unwrap();

        let expected_deposits = NetDeposits::empty(UnspecifiedResources::none())
            .set(XRD, ResourceBounds::general_fungible(3, 7).unwrap());
        let expected_withdraws = NetWithdraws::empty().set_fungible(XRD, 5);
        assert_eq!(net_deposits.get(&account), Some(&expected_deposits));
        assert_eq!(net_withdraws.get(&account), Some(&expected_withdraws));
    }
}

#[test]
fn static_analysis_on_a_vault_direct_method_succeeds() {
    // Arrange
    let manifest = ManifestBuilder::new()
        .call_direct_access_method(
            vault_id(1),
            VAULT_FREEZE_IDENT,
            VaultFreezeManifestInput {
                to_freeze: VaultFreezeFlags::all(),
            },
        )
        .build();

    // Act
    let rtn = statically_analyze(&manifest);

    // Act
    assert!(rtn.is_ok());
}

#[test]
fn static_analysis_on_account_add_authorized_depositor_with_named_address_succeeds() {
    // Arrange
    let manifest_string = r#"
    CALL_METHOD
        Address("component_tdx_2_1cptxxxxxxxxxfaucetxxxxxxxxx000527798379xxxxxxxxxyulkzl")
        "lock_fee"
        Decimal("5000")
    ;
    CALL_METHOD
        Address("account_tdx_2_16996e320lnez82q6430eunaz9l3n5fnwk6eh9avrmtmj22e7ll92cg")
        "set_default_deposit_rule"
        Enum<1u8>()
    ;
    CALL_METHOD
        Address("account_tdx_2_168qgdkgfqxpnswu38wy6fy5v0q0um52zd0umuely5t9xrf88x4wqmf")
        "set_default_deposit_rule"
        Enum<1u8>()
    ;
    ALLOCATE_GLOBAL_ADDRESS
        Address("package_tdx_2_1pkgxxxxxxxxxresrcexxxxxxxxx000538436477xxxxxxxxxmn4mes")
        "FungibleResourceManager"
        AddressReservation("reservation1")
        NamedAddress("address1")
    ;
    CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY
        Enum<0u8>()
        true
        18u8
        Decimal("1")
        Tuple(
            Enum<0u8>(),
            Enum<0u8>(),
            Enum<0u8>(),
            Enum<0u8>(),
            Enum<0u8>(),
            Enum<0u8>()
        )
        Tuple(
            Map<String, Tuple>(),
            Map<String, Enum>()
        )
        Enum<1u8>(
            AddressReservation("reservation1")
        )
    ;
    CALL_METHOD
        Address("account_tdx_2_168qgdkgfqxpnswu38wy6fy5v0q0um52zd0umuely5t9xrf88x4wqmf")
        "add_authorized_depositor"
        Enum<1u8>(
            NamedAddress("address1")
        )
    ;
    CALL_METHOD
        Address("account_tdx_2_16996e320lnez82q6430eunaz9l3n5fnwk6eh9avrmtmj22e7ll92cg")
        "deposit_batch"
        Expression("ENTIRE_WORKTOP")
    ;
    "#;
    let manifest = compile(
        manifest_string,
        &NetworkDefinition::stokenet(),
        MockBlobProvider::new(),
    )
    .unwrap();

    // Act
    let rtn = statically_analyze(&manifest);

    // Assert
    assert!(rtn.is_ok());
}

fn account_address(id: u64) -> ComponentAddress {
    unsafe {
        ComponentAddress::new_unchecked(node_id(EntityType::GlobalPreallocatedEd25519Account, id).0)
    }
}

fn component_address(id: u64) -> ComponentAddress {
    unsafe { ComponentAddress::new_unchecked(node_id(EntityType::GlobalGenericComponent, id).0) }
}

fn vault_id(id: u64) -> InternalAddress {
    unsafe { InternalAddress::new_unchecked(node_id(EntityType::InternalFungibleVault, id).0) }
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
        IndexMap<ComponentAddress, NetDeposits>,
        IndexMap<ComponentAddress, NetWithdraws>,
    ),
    StaticResourceMovementsError,
> {
    let interpreter = StaticManifestInterpreter::new(ValidationRuleset::all(), manifest);
    let mut visitor: StaticResourceMovementsVisitor =
        StaticResourceMovementsVisitor::new(manifest.is_subintent());
    interpreter.validate_and_apply_visitor(&mut visitor)?;
    let output = visitor.output();
    let (all_deposits, all_withdraws) = (
        output.resolve_account_deposits(),
        output.resolve_account_withdraws(),
    );
    let (net_withdraws, net_deposits) = output.resolve_account_changes()?;
    Ok((all_deposits, all_withdraws, net_deposits, net_withdraws))
}
