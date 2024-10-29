use radix_engine::blueprints::pool::v1::errors::{
    multi_resource_pool::Error as MultiResourcePoolError,
    one_resource_pool::Error as OneResourcePoolError,
    two_resource_pool::Error as TwoResourcePoolError,
};
use radix_engine_tests::pool_stubs::*;
use scrypto_test::prelude::*;

macro_rules! atto {
    (
        $($tokens: tt)*
    ) => {
        Decimal::from_attos(I192::from($($tokens)*))
    };
}

#[test]
fn one_resource_pool_redemption_value_calculation_does_not_lose_precision_at_divisibility_18(
) -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();
    let mut pool = OneResourcePool::instantiate(XRD, OwnerRole::None, rule!(allow_all), None, env)?;

    let bucket = env
        .with_auth_module_disabled(|env| ResourceManager(XRD).mint_fungible(dec!(100_000), env))?;
    let _ = pool.contribute(bucket, env)?;

    // Act
    let redemption_value = pool.get_redemption_value(atto!(1), env)?;

    // Assert
    assert_ne!(redemption_value, Decimal::ZERO);
    Ok(())
}

#[test]
fn one_resource_pool_redemption_value_calculation_does_not_lose_precision_at_divisibility_2(
) -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(2)
        .mint_initial_supply(dec!(100_000), env)?;
    let resource_address = bucket.resource_address(env)?;
    let mut pool = OneResourcePool::instantiate(
        resource_address,
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;
    let _ = pool.contribute(bucket, env)?;

    // Act
    let redemption_value = pool.get_redemption_value(dec!(0.01), env)?;

    // Assert
    assert_ne!(redemption_value, Decimal::ZERO);
    Ok(())
}

#[test]
fn one_resource_pool_redemption_returning_zero_fails_with_error() -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(2)
        .mint_initial_supply(dec!(100_000), env)?;
    let resource_address = bucket.resource_address(env)?;
    let mut pool = OneResourcePool::instantiate(
        resource_address,
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;
    let pool_units = pool.contribute(bucket, env)?;

    // Act
    let pool_units_to_redeem = pool_units.take(dec!(0.001), env)?;
    let rtn = pool.redeem(pool_units_to_redeem, env);

    // Assert
    assert_matches!(
        rtn,
        Err(RuntimeError::ApplicationError(
            ApplicationError::OneResourcePoolError(OneResourcePoolError::RedeemedZeroTokens)
        ))
    );
    Ok(())
}

#[test]
fn one_resource_pool_contributions_must_return_pool_units() -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();
    let mut pool = OneResourcePool::instantiate(XRD, OwnerRole::None, rule!(allow_all), None, env)?;

    let contribution_bucket = env.with_auth_module_disabled(|env| {
        ResourceManager(XRD).mint_fungible(dec!(100_000_000_000), env)
    })?;
    let _ = pool.contribute(contribution_bucket, env)?;

    // Act
    let second_contribution_bucket = env.with_auth_module_disabled(|env| {
        ResourceManager(XRD).mint_fungible(dec!(0.000000010000000000), env)
    })?;
    let pool_units = pool.contribute(second_contribution_bucket, env)?;

    // Assert
    assert_ne!(pool_units.amount(env)?, dec!(0));
    Ok(())
}

/// In this test very small amount of pool units ≡ very large amount of tokens. This is what we call
/// concentrated pool units.
///
/// In this test roughly 100,000,000,000 XRD is equivalent to 1 Atto of a Pool Unit. Contributions
/// of anything less than 100,000,000,000 XRD would mean minting less than 1 Atto of a pool unit
/// which is not possible. Thus, this results in an error.
#[test]
fn one_resource_pool_contributing_to_pool_with_concentrated_pool_units_should_error(
) -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();
    let mut pool = OneResourcePool::instantiate(XRD, OwnerRole::None, rule!(allow_all), None, env)?;

    let xrd_bucket = env.with_auth_module_disabled(|env| {
        ResourceManager(XRD).mint_fungible(dec!(100_000_000_000), env)
    })?;
    let contribution_bucket = xrd_bucket.take(atto!(1), env)?;
    let _ = pool.contribute(contribution_bucket, env)?;

    // Act
    pool.protected_deposit(xrd_bucket, env)?;
    let contribution_bucket =
        env.with_auth_module_disabled(|env| ResourceManager(XRD).mint_fungible(dec!(1), env))?;
    let rtn = pool.contribute(contribution_bucket, env);

    // Assert
    assert_matches!(
        rtn,
        Err(RuntimeError::ApplicationError(
            ApplicationError::OneResourcePoolError(OneResourcePoolError::ZeroPoolUnitsMinted)
        ))
    );
    Ok(())
}

#[test]
fn two_resource_pool_redemption_value_calculation_does_not_lose_precision_at_divisibility_18(
) -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;

    let resource_address1 = bucket1.resource_address(env)?;
    let resource_address2 = bucket2.resource_address(env)?;

    let mut pool = TwoResourcePool::instantiate(
        (resource_address1, resource_address2),
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;
    let _ = pool.contribute((bucket1, bucket2), env)?;

    // Act
    let redemption_value = pool.get_redemption_value(atto!(1), env)?;

    // Assert
    assert!(redemption_value
        .values()
        .all(|value| *value != Decimal::ZERO));
    Ok(())
}

#[test]
fn two_resource_pool_redemption_value_calculation_does_not_lose_precision_at_divisibility_2(
) -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(2)
        .mint_initial_supply(dec!(100_000_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(2)
        .mint_initial_supply(dec!(100_000_000_000), env)?;

    let resource_address1 = bucket1.resource_address(env)?;
    let resource_address2 = bucket2.resource_address(env)?;

    let mut pool = TwoResourcePool::instantiate(
        (resource_address1, resource_address2),
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;
    let _ = pool.contribute((bucket1, bucket2), env)?;

    // Act
    let redemption_value = pool.get_redemption_value(dec!(0.01), env)?;

    // Assert
    assert!(redemption_value
        .values()
        .all(|value| *value != Decimal::ZERO));
    Ok(())
}

#[test]
fn two_resource_pool_very_small_contributions_should_return_pool_units1() -> Result<(), RuntimeError>
{
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;

    let resource_address1 = bucket1.resource_address(env)?;
    let resource_address2 = bucket2.resource_address(env)?;

    let mut pool = TwoResourcePool::instantiate(
        (resource_address1, resource_address2),
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;
    let _ = {
        let contribution_bucket1 = bucket1.take(dec!(1000), env)?;
        let contribution_bucket2 = bucket2.take(dec!(1000), env)?;
        pool.contribute((contribution_bucket1, contribution_bucket2), env)?
    };

    // Act
    let contribution_bucket1 = bucket1.take(atto!(1), env)?;
    let contribution_bucket2 = bucket2.take(atto!(1), env)?;
    let (pool_units, _) = pool.contribute((contribution_bucket1, contribution_bucket2), env)?;

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_ne!(pool_units_amount, Decimal::ZERO);

    Ok(())
}

#[test]
fn two_resource_pool_calculations_loading_to_zero_should_error() -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(atto!(2), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(atto!(150), env)?;

    let resource_address1 = bucket1.resource_address(env)?;
    let resource_address2 = bucket2.resource_address(env)?;

    let mut pool = TwoResourcePool::instantiate(
        (resource_address1, resource_address2),
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;
    let _ = {
        let contribution_bucket1 = bucket1.take(atto!(1), env)?;
        let contribution_bucket2 = bucket2.take(atto!(100), env)?;
        pool.contribute((contribution_bucket1, contribution_bucket2), env)?
    };

    // Act
    let contribution_bucket1 = bucket1.take(atto!(1), env)?;
    let contribution_bucket2 = bucket2.take(atto!(50), env)?;
    let rtn = pool.contribute((contribution_bucket1, contribution_bucket2), env);

    // Assert
    assert_matches!(
        rtn,
        Err(RuntimeError::ApplicationError(
            ApplicationError::TwoResourcePoolError(
                TwoResourcePoolError::LargerContributionRequiredToMeetRatio
            )
        ))
    );

    Ok(())
}

#[test]
fn two_resource_pool_very_small_contributions_should_return_pool_units_even_for_small_divisibilities(
) -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(2)
        .mint_initial_supply(dec!(100_000_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(2)
        .mint_initial_supply(dec!(100_000_000_000), env)?;

    let resource_address1 = bucket1.resource_address(env)?;
    let resource_address2 = bucket2.resource_address(env)?;

    let mut pool = TwoResourcePool::instantiate(
        (resource_address1, resource_address2),
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;
    let _ = {
        let contribution_bucket1 = bucket1.take(dec!(2000), env)?;
        let contribution_bucket2 = bucket2.take(dec!(1000), env)?;
        pool.contribute((contribution_bucket1, contribution_bucket2), env)?
    };

    // Act
    let contribution_bucket1 = bucket1.take(dec!(0.02), env)?;
    let contribution_bucket2 = bucket2.take(dec!(0.01), env)?;
    let (pool_units, _) = pool.contribute((contribution_bucket1, contribution_bucket2), env)?;

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_ne!(pool_units_amount, Decimal::ZERO);

    Ok(())
}

#[test]
fn two_resource_pool_one_sided_liquidity_can_be_provided_when_one_of_the_reserves_is_zero1(
) -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;

    let resource_address1 = bucket1.resource_address(env)?;
    let resource_address2 = bucket2.resource_address(env)?;

    let mut pool = TwoResourcePool::instantiate(
        (resource_address1, resource_address2),
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;
    let _ = {
        let contribution_bucket1 = bucket1.take(dec!(1000), env)?;
        let contribution_bucket2 = bucket2.take(dec!(1000), env)?;
        pool.contribute((contribution_bucket1, contribution_bucket2), env)?
    };
    let _ = pool.protected_withdraw(resource_address1, dec!(1000), WithdrawStrategy::Exact, env)?;

    // Act
    let contribution_bucket1 = bucket1.take(dec!(1000), env)?;
    let contribution_bucket2 = bucket2.take(dec!(1000), env)?;
    let (pool_units, change) =
        pool.contribute((contribution_bucket1, contribution_bucket2), env)?;

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_eq!(pool_units_amount, dec!(1000));
    assert!(
        change.is_some_and(
            |bucket| bucket.resource_address(env).unwrap() == resource_address1
                && bucket.amount(env).unwrap() == dec!(1000)
        )
    );

    Ok(())
}

#[test]
fn two_resource_pool_one_sided_liquidity_can_be_provided_when_one_of_the_reserves_is_zero2(
) -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;

    let resource_address1 = bucket1.resource_address(env)?;
    let resource_address2 = bucket2.resource_address(env)?;

    let mut pool = TwoResourcePool::instantiate(
        (resource_address1, resource_address2),
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;
    let _ = {
        let contribution_bucket1 = bucket1.take(dec!(1000), env)?;
        let contribution_bucket2 = bucket2.take(dec!(1000), env)?;
        pool.contribute((contribution_bucket1, contribution_bucket2), env)?
    };
    let _ = pool.protected_withdraw(resource_address2, dec!(1000), WithdrawStrategy::Exact, env)?;

    // Act
    let contribution_bucket1 = bucket1.take(dec!(1000), env)?;
    let contribution_bucket2 = bucket2.take(dec!(1000), env)?;
    let (pool_units, change) =
        pool.contribute((contribution_bucket1, contribution_bucket2), env)?;

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_eq!(pool_units_amount, dec!(1000));
    assert!(
        change.is_some_and(
            |bucket| bucket.resource_address(env).unwrap() == resource_address2
                && bucket.amount(env).unwrap() == dec!(1000)
        )
    );

    Ok(())
}

#[test]
fn two_resource_pool_initial_contribution_should_not_return_zero_pool_units(
) -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;

    let resource_address1 = bucket1.resource_address(env)?;
    let resource_address2 = bucket2.resource_address(env)?;

    let mut pool = TwoResourcePool::instantiate(
        (resource_address1, resource_address2),
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;

    // Act
    let (pool_units, _) = pool.contribute(
        (
            bucket1,
            FungibleBucket(Bucket::create(resource_address2, env)?),
        ),
        env,
    )?;

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_ne!(pool_units_amount, Decimal::ZERO);

    Ok(())
}

#[test]
fn two_resource_pool_contributing_to_pool_with_concentrated_pool_units_should_error(
) -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;

    let resource_address1 = bucket1.resource_address(env)?;
    let resource_address2 = bucket2.resource_address(env)?;

    let mut pool = TwoResourcePool::instantiate(
        (resource_address1, resource_address2),
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;

    let _ = {
        let contribution_bucket1 = bucket1.take(atto!(1), env)?;
        let contribution_bucket2 = bucket2.take(atto!(1), env)?;
        pool.contribute((contribution_bucket1, contribution_bucket2), env)?
    };
    bucket1
        .take(dec!(100_000_000), env)
        .and_then(|bucket| pool.protected_deposit(bucket, env))?;
    bucket2
        .take(dec!(100_000_000), env)
        .and_then(|bucket| pool.protected_deposit(bucket, env))?;

    // Act
    let rtn = {
        let contribution_bucket1 = bucket1.take(dec!(10_000_000), env)?;
        let contribution_bucket2 = bucket2.take(dec!(10_000_000), env)?;
        pool.contribute((contribution_bucket1, contribution_bucket2), env)
    };

    // Assert
    assert_matches!(
        rtn,
        Err(RuntimeError::ApplicationError(
            ApplicationError::TwoResourcePoolError(TwoResourcePoolError::ZeroPoolUnitsMinted)
        ))
    );
    Ok(())
}

#[test]
fn two_resource_pool_contribution_errors_when_both_reserves_are_empty() -> Result<(), RuntimeError>
{
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000_000), env)?;

    let resource_address1 = bucket1.resource_address(env)?;
    let resource_address2 = bucket2.resource_address(env)?;

    let mut pool = TwoResourcePool::instantiate(
        (resource_address1, resource_address2),
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;

    let _ = {
        let contribution_bucket1 = bucket1.take(dec!(1), env)?;
        let contribution_bucket2 = bucket2.take(dec!(1), env)?;
        pool.contribute((contribution_bucket1, contribution_bucket2), env)?
    };

    let _ = pool.protected_withdraw(resource_address1, dec!(1), WithdrawStrategy::Exact, env)?;
    let _ = pool.protected_withdraw(resource_address2, dec!(1), WithdrawStrategy::Exact, env)?;

    // Act
    let rtn = {
        let contribution_bucket1 = bucket1.take(dec!(1), env)?;
        let contribution_bucket2 = bucket2.take(dec!(1), env)?;
        pool.contribute((contribution_bucket1, contribution_bucket2), env)
    };

    // Assert
    assert_matches!(
        rtn,
        Err(RuntimeError::ApplicationError(
            ApplicationError::TwoResourcePoolError(
                TwoResourcePoolError::NonZeroPoolUnitSupplyButZeroReserves
            )
        ))
    );

    Ok(())
}

#[test]
fn two_resource_pool_errors_out_when_one_of_the_resources_is_calculated_out_to_be_zero_in_normal_operations(
) -> Result<(), RuntimeError> {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(dec!(1000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(2)
        .mint_initial_supply(dec!(1000), env)?;

    let resource_address1 = bucket1.resource_address(env)?;
    let resource_address2 = bucket2.resource_address(env)?;

    let mut pool = TwoResourcePool::instantiate(
        (resource_address1, resource_address2),
        OwnerRole::None,
        rule!(allow_all),
        None,
        env,
    )?;

    {
        let bucket1 = bucket1.take(dec!(1), env)?;
        let bucket2 = bucket2.take(dec!(0.05), env)?;
        let _ = pool.contribute((bucket1, bucket2), env)?;
    }

    // Act
    let rtn = {
        let bucket1 = bucket1.take(dec!(0.01), env)?;
        let bucket2 = bucket2.take(dec!(0.01), env)?;
        pool.contribute((bucket1, bucket2), env)
    };

    // Assert
    assert!(
        matches!(
            rtn,
            Err(RuntimeError::ApplicationError(
                ApplicationError::TwoResourcePoolError(
                    TwoResourcePoolError::LargerContributionRequiredToMeetRatio
                )
            ))
        ),
        "{rtn:#?}",
    );

    Ok(())
}

#[test]
fn multi_resource_pool_accepts_very_large_contributions() -> Result<(), RuntimeError> {
    // Arrange
    let divisibility = core::array::from_fn::<u8, 16, _>(|_| DIVISIBILITY_MAXIMUM);
    with_multi_resource_pool(divisibility, |env, buckets, mut pool| {
        // Act
        let (pool_units, _) = pool
            .contribute(buckets.map(|(bucket, _)| bucket.0), env)
            .expect("Must Succeed!");

        // Assert
        let pool_units_amount = pool_units.amount(env)?;
        assert!(approximately_equals(pool_units_amount, MINT_LIMIT));
        Ok(())
    })
}

#[test]
fn multi_resource_pool_permits_some_zero_contributions_initially() -> Result<(), RuntimeError> {
    // Arrange
    with_multi_resource_pool(
        [18, 18, 18],
        |env, [(bucket1, _), (bucket2, _), (bucket3, _)], mut pool| {
            let (pool_units, change) = pool
                .contribute(
                    [
                        bucket1.take(dec!(100_000_000), env)?,
                        bucket2.take(dec!(0), env)?,
                        bucket3.take(dec!(100_000_000), env)?,
                    ],
                    env,
                )
                .expect("Must Succeed!");

            // Assert
            let pool_units_amount = pool_units.amount(env)?;

            assert!(approximately_equals(pool_units_amount, dec!(100_000_000)));
            assert_eq!(change.len(), 0);
            Ok(())
        },
    )
}

#[test]
fn multi_resource_pool_permits_some_zero_contributions1() -> Result<(), RuntimeError> {
    // Arrange
    with_multi_resource_pool(
        [18, 18, 18],
        |env, [(bucket1, _), (bucket2, resource_address2), (bucket3, _)], mut pool| {
            let _ = pool
                .contribute(
                    [
                        bucket1.take(dec!(100_000_000), env)?,
                        bucket2.take(dec!(0), env)?,
                        bucket3.take(dec!(100_000_000), env)?,
                    ],
                    env,
                )
                .expect("Must Succeed!");

            // Act
            let (pool_units, change) = pool
                .contribute(
                    [
                        bucket1.take(dec!(100_000_000), env)?,
                        bucket2.take(dec!(100_000_000), env)?,
                        bucket3.take(dec!(100_000_000), env)?,
                    ],
                    env,
                )
                .expect("Must Succeed!");

            // Assert
            let pool_units_amount = pool_units.amount(env)?;
            let change_bucket = change.first().unwrap();

            assert!(approximately_equals(pool_units_amount, dec!(100_000_000)));
            assert_eq!(change.len(), 1);
            assert_eq!(change_bucket.resource_address(env)?, resource_address2);
            assert_eq!(change_bucket.amount(env)?, dec!(100_000_000));
            Ok(())
        },
    )
}

#[test]
fn multi_resource_pool_permits_some_zero_contributions2() -> Result<(), RuntimeError> {
    // Arrange
    with_multi_resource_pool(
        [18, 18, 18],
        |env, [(bucket1, _), (bucket2, resource_address2), (bucket3, _)], mut pool| {
            let _ = pool
                .contribute(
                    [
                        bucket1.take(dec!(100_000_000), env)?,
                        bucket2.take(dec!(100_000_000), env)?,
                        bucket3.take(dec!(100_000_000), env)?,
                    ],
                    env,
                )
                .expect("Must Succeed!");
            let _ = pool.protected_withdraw(
                resource_address2,
                dec!(100_000_000),
                WithdrawStrategy::Exact,
                env,
            )?;

            // Act
            let (pool_units, change) = pool
                .contribute(
                    [
                        bucket1.take(dec!(100_000_000), env)?,
                        bucket2.take(dec!(100_000_000), env)?,
                        bucket3.take(dec!(100_000_000), env)?,
                    ],
                    env,
                )
                .expect("Must Succeed!");

            // Assert
            let pool_units_amount = pool_units.amount(env)?;
            let change_bucket = change.first().unwrap();

            assert!(approximately_equals(pool_units_amount, dec!(100_000_000)));
            assert_eq!(change.len(), 1);
            assert_eq!(change_bucket.resource_address(env)?, resource_address2);
            assert_eq!(change_bucket.amount(env)?, dec!(100_000_000));
            Ok(())
        },
    )
}

#[test]
fn multi_resource_pool_permits_some_zero_contributions3() -> Result<(), RuntimeError> {
    // Arrange
    with_multi_resource_pool(
        [18, 18, 18],
        |env, [(bucket1, _), (bucket2, _), (bucket3, _)], mut pool| {
            let _ = pool
                .contribute(
                    [
                        bucket1.take(dec!(100_000_000), env)?,
                        bucket2.take(dec!(0), env)?,
                        bucket3.take(dec!(100_000_000), env)?,
                    ],
                    env,
                )
                .expect("Must Succeed!");

            // Act
            let (pool_units, change) = pool
                .contribute(
                    [
                        bucket1.take(dec!(100_000_000), env)?,
                        bucket2.take(dec!(0), env)?,
                        bucket3.take(dec!(100_000_000), env)?,
                    ],
                    env,
                )
                .expect("Must Succeed!");

            // Assert
            let pool_units_amount = pool_units.amount(env)?;

            assert!(approximately_equals(pool_units_amount, dec!(100_000_000)));
            assert_eq!(change.len(), 0);
            Ok(())
        },
    )
}

#[test]
fn multi_resource_pool_rejects_contributions_if_all_liquidity_has_been_removed(
) -> Result<(), RuntimeError> {
    // Arrange
    with_multi_resource_pool(
        [18, 18, 18],
        |env,
         [(bucket1, resource_address1), (bucket2, resource_address2), (bucket3, resource_address3)],
         mut pool| {
            let _ = pool
                .contribute(
                    [
                        bucket1.take(dec!(100_000_000), env)?,
                        bucket2.take(dec!(100_000_000), env)?,
                        bucket3.take(dec!(100_000_000), env)?,
                    ],
                    env,
                )
                .expect("Must Succeed!");
            for resource_address in [resource_address1, resource_address2, resource_address3] {
                let _ = pool.protected_withdraw(
                    resource_address,
                    dec!(100_000_000),
                    WithdrawStrategy::Exact,
                    env,
                )?;
            }

            // Act
            let rtn = pool.contribute(
                [
                    bucket1.take(dec!(100_000_000), env)?,
                    bucket2.take(dec!(100_000_000), env)?,
                    bucket3.take(dec!(100_000_000), env)?,
                ],
                env,
            );

            // Assert
            assert_matches!(
                rtn,
                Err(RuntimeError::ApplicationError(
                    ApplicationError::MultiResourcePoolError(
                        MultiResourcePoolError::NoMinimumRatio
                    )
                ))
            );
            Ok(())
        },
    )
}

#[test]
fn multi_resource_pool_mints_pool_units_for_very_small_contributions() -> Result<(), RuntimeError> {
    // Arrange
    with_multi_resource_pool(
        [18, 18, 18],
        |env, [(bucket1, _), (bucket2, _), (bucket3, _)], mut pool| {
            let _ = pool
                .contribute(
                    [
                        bucket1.take(dec!(100_000_000), env)?,
                        bucket2.take(dec!(100_000_000), env)?,
                        bucket3.take(dec!(100_000_000), env)?,
                    ],
                    env,
                )
                .expect("Must Succeed!");

            // Act
            let (pool_units, change) = pool.contribute(
                [
                    bucket1.take(atto!(1), env)?,
                    bucket2.take(atto!(1), env)?,
                    bucket3.take(atto!(1), env)?,
                ],
                env,
            )?;

            // Assert
            let pool_units_amount = pool_units.amount(env)?;
            assert_ne!(pool_units_amount, Decimal::ZERO);
            assert!(change.is_empty());
            Ok(())
        },
    )
}

#[test]
fn multi_resource_pool_contributing_to_pool_with_concentrated_pool_units_should_error(
) -> Result<(), RuntimeError> {
    // Arrange
    with_multi_resource_pool(
        [18, 18, 18],
        |env, [(bucket1, _), (bucket2, _), (bucket3, _)], mut pool| {
            let _ = pool
                .contribute(
                    [
                        bucket1.take(atto!(1), env)?,
                        bucket2.take(atto!(1), env)?,
                        bucket3.take(atto!(1), env)?,
                    ],
                    env,
                )
                .expect("Must Succeed!");
            for bucket in [bucket1.clone(), bucket2.clone(), bucket3.clone()] {
                let deposit_bucket = bucket.take(dec!(100_000_000), env)?;
                pool.protected_deposit(deposit_bucket, env)?;
            }

            // Act
            let rtn = pool.contribute(
                [
                    bucket1.take(dec!(1000), env)?,
                    bucket2.take(dec!(1000), env)?,
                    bucket3.take(dec!(1000), env)?,
                ],
                env,
            );

            // Assert
            assert_matches!(
                rtn,
                Err(RuntimeError::ApplicationError(
                    ApplicationError::MultiResourcePoolError(
                        MultiResourcePoolError::ZeroPoolUnitsMinted
                    )
                ))
            );
            Ok(())
        },
    )
}

#[test]
fn multi_resource_pool_redemption_value_calculation_does_not_lose_precision_at_divisibility_18(
) -> Result<(), RuntimeError> {
    // Arrange
    with_multi_resource_pool(
        [18, 18, 18],
        |env, [(bucket1, _), (bucket2, _), (bucket3, _)], mut pool| {
            let _ = pool
                .contribute(
                    [
                        bucket1.take(dec!(100_000_000), env)?,
                        bucket2.take(dec!(100_000_000), env)?,
                        bucket3.take(dec!(100_000_000), env)?,
                    ],
                    env,
                )
                .expect("Must Succeed!");

            // Act
            let redemption_amount = pool.get_redemption_value(atto!(1), env)?;

            // Assert
            assert!(redemption_amount.values().all(|value| !value.is_zero()));
            Ok(())
        },
    )
}

#[test]
fn multi_resource_pool_contribution_errors_when_both_reserves_are_empty() -> Result<(), RuntimeError>
{
    // Arrange
    with_multi_resource_pool(
        [18, 18],
        |env, [(bucket1, resource_address1), (bucket2, resource_address2)], mut pool| {
            let _ = {
                let contribution_bucket1 = bucket1.take(dec!(1), env)?;
                let contribution_bucket2 = bucket2.take(dec!(1), env)?;
                pool.contribute([contribution_bucket1, contribution_bucket2], env)?
            };

            let _ =
                pool.protected_withdraw(resource_address1, dec!(1), WithdrawStrategy::Exact, env)?;
            let _ =
                pool.protected_withdraw(resource_address2, dec!(1), WithdrawStrategy::Exact, env)?;

            // Act
            let rtn = {
                let contribution_bucket1 = bucket1.take(dec!(1), env)?;
                let contribution_bucket2 = bucket2.take(dec!(1), env)?;
                pool.contribute([contribution_bucket1, contribution_bucket2], env)
            };

            // Assert
            assert_matches!(
                rtn,
                Err(RuntimeError::ApplicationError(
                    ApplicationError::MultiResourcePoolError(
                        MultiResourcePoolError::NoMinimumRatio
                    )
                ))
            );

            Ok(())
        },
    )
}

#[test]
fn multi_resource_pool_errors_out_when_one_of_the_resources_is_calculated_out_to_be_zero_in_normal_operations(
) -> Result<(), RuntimeError> {
    // Arrange
    with_multi_resource_pool([18, 2], |env, [(bucket1, _), (bucket2, _)], mut pool| {
        {
            let bucket1 = bucket1.take(dec!(1), env)?;
            let bucket2 = bucket2.take(dec!(0.05), env)?;
            let _ = pool.contribute([bucket1, bucket2], env)?;
        }

        // Act
        let rtn = {
            let bucket1 = bucket1.take(dec!(0.01), env)?;
            let bucket2 = bucket2.take(dec!(0.01), env)?;
            pool.contribute([bucket1, bucket2], env)
        };

        // Assert
        assert!(
            matches!(
                rtn,
                Err(RuntimeError::ApplicationError(
                    ApplicationError::MultiResourcePoolError(
                        MultiResourcePoolError::LargerContributionRequiredToMeetRatio
                    )
                ))
            ),
            "{rtn:#?}",
        );

        Ok(())
    })
}

pub fn approximately_equals(this: Decimal, other: Decimal) -> bool {
    ((other - this) / this).checked_abs().unwrap() < dec!(0.01)
}
