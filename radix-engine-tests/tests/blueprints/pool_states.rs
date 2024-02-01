//! This module tests the pool states defined in the doc comment of [`TwoResourcePool::contribute`]
//! and [`MultiResourcePool::contribute`]. As such, any reference of states 1, 2, 3, or 4 comes from
//! there.
//!
//! [`TwoResourcePool::contribute`]:
//! radix_engine::blueprints::pool::v1::v1_1::TwoResourcePoolBlueprint::contribute
//! [`MultiResourcePool::contribute`]:
//! radix_engine::blueprints::pool::v1::v1_1::MultiResourcePoolBlueprint::contribute

use radix_engine::blueprints::pool::v1::errors::{
    multi_resource_pool::Error as MultiResourcePoolError,
    two_resource_pool::Error as TwoResourcePoolError,
};
use radix_engine_tests::pool_stubs::*;
use scrypto_test::prelude::*;

type TestResult = Result<(), RuntimeError>;

#[test]
fn two_resource_pool_is_in_state_1_when_totally_empty() -> TestResult {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100), env)?;

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
    let (pool_units, change) = pool.contribute((bucket1, bucket2), env)?;

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_eq!(pool_units_amount, dec!(100));
    assert!(change.is_none());

    Ok(())
}

#[test]
fn two_resource_pool_is_in_state_1_when_dusty1() -> TestResult {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;

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
        let bucket1 = bucket1.take(dec!(100), env)?;
        pool.protected_deposit(bucket1, env)?;
    }

    // Act
    let (pool_units, change) = {
        let bucket1 = bucket1.take(dec!(100), env)?;
        let bucket2 = bucket2.take(dec!(100), env)?;
        pool.contribute((bucket1, bucket2), env)?
    };

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_eq!(pool_units_amount, dec!(100));
    assert!(change.is_none());

    Ok(())
}

#[test]
fn two_resource_pool_is_in_state_1_when_dusty2() -> TestResult {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;

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
        let bucket2 = bucket2.take(dec!(100), env)?;
        pool.protected_deposit(bucket2, env)?;
    }

    // Act
    let (pool_units, change) = {
        let bucket1 = bucket1.take(dec!(100), env)?;
        let bucket2 = bucket2.take(dec!(100), env)?;
        pool.contribute((bucket1, bucket2), env)?
    };

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_eq!(pool_units_amount, dec!(100));
    assert!(change.is_none());

    Ok(())
}

#[test]
fn two_resource_pool_is_in_state_1_when_dusty3() -> TestResult {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;

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
        let bucket1 = bucket1.take(dec!(100), env)?;
        pool.protected_deposit(bucket1, env)?;
        let bucket2 = bucket2.take(dec!(100), env)?;
        pool.protected_deposit(bucket2, env)?;
    }

    // Act
    let (pool_units, change) = {
        let bucket1 = bucket1.take(dec!(100), env)?;
        let bucket2 = bucket2.take(dec!(100), env)?;
        pool.contribute((bucket1, bucket2), env)?
    };

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_eq!(pool_units_amount, dec!(100));
    assert!(change.is_none());

    Ok(())
}

#[test]
fn two_resource_pool_is_in_state_2_when_all_reserves_are_empty() -> TestResult {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;

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
        let bucket1 = bucket1.take(dec!(100), env)?;
        let bucket2 = bucket2.take(dec!(100), env)?;
        pool.contribute((bucket1, bucket2), env)?
    };
    {
        let _ = pool.protected_withdraw(resource_address1, dec!(100), Default::default(), env)?;
        let _ = pool.protected_withdraw(resource_address2, dec!(100), Default::default(), env)?;
    }

    // Act
    let rtn = {
        let bucket1 = bucket1.take(dec!(100), env)?;
        let bucket2 = bucket2.take(dec!(100), env)?;
        pool.contribute((bucket1, bucket2), env)
    };

    // Assert
    assert_eq!(
        rtn,
        Err(RuntimeError::ApplicationError(
            ApplicationError::TwoResourcePoolError(
                TwoResourcePoolError::NonZeroPoolUnitSupplyButZeroReserves,
            ),
        )),
    );
    Ok(())
}

#[test]
fn two_resource_pool_is_in_state_3_when_some_reserves_are_empty1() -> TestResult {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;

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
        let bucket1 = bucket1.take(dec!(100), env)?;
        let bucket2 = bucket2.take(dec!(100), env)?;
        pool.contribute((bucket1, bucket2), env)?
    };
    {
        let _ = pool.protected_withdraw(resource_address1, dec!(100), Default::default(), env)?;
    }

    // Act
    let (pool_units, change) = {
        let bucket1 = bucket1.take(dec!(70), env)?;
        let bucket2 = bucket2.take(dec!(70), env)?;
        pool.contribute((bucket1, bucket2), env)?
    };

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_eq!(pool_units_amount, dec!(70));

    let change = change.expect("There should be change");
    let change_resource = change.resource_address(env)?;
    let change_amount = change.amount(env)?;

    assert_eq!(change_resource, resource_address1);
    assert_eq!(change_amount, dec!(70));

    Ok(())
}

#[test]
fn two_resource_pool_is_in_state_3_when_some_reserves_are_empty2() -> TestResult {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;

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
        let bucket1 = bucket1.take(dec!(100), env)?;
        let bucket2 = bucket2.take(dec!(100), env)?;
        pool.contribute((bucket1, bucket2), env)?
    };
    {
        let _ = pool.protected_withdraw(resource_address2, dec!(100), Default::default(), env)?;
    }

    // Act
    let (pool_units, change) = {
        let bucket1 = bucket1.take(dec!(70), env)?;
        let bucket2 = bucket2.take(dec!(70), env)?;
        pool.contribute((bucket1, bucket2), env)?
    };

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_eq!(pool_units_amount, dec!(70));

    let change = change.expect("There should be change");
    let change_resource = change.resource_address(env)?;
    let change_amount = change.amount(env)?;

    assert_eq!(change_resource, resource_address2);
    assert_eq!(change_amount, dec!(70));

    Ok(())
}

#[test]
fn two_resource_pool_is_in_state_4_when_in_normal_operations() -> TestResult {
    // Arrange
    let env = &mut TestEnvironment::new();

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(DIVISIBILITY_MAXIMUM)
        .mint_initial_supply(dec!(100_000_000), env)?;

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
        let bucket1 = bucket1.take(dec!(100), env)?;
        let bucket2 = bucket2.take(dec!(100), env)?;
        pool.contribute((bucket1, bucket2), env)?
    };

    // Act
    let (pool_units, change) = {
        let bucket1 = bucket1.take(dec!(100), env)?;
        let bucket2 = bucket2.take(dec!(110), env)?;
        pool.contribute((bucket1, bucket2), env)?
    };

    // Assert
    let pool_units_amount = pool_units.amount(env)?;
    assert_eq!(pool_units_amount, dec!(100));

    let change = change.expect("There should be change");
    let change_resource = change.resource_address(env)?;
    let change_amount = change.amount(env)?;

    assert_eq!(change_resource, resource_address2);
    assert_eq!(change_amount, dec!(10));

    Ok(())
}

#[test]
fn multi_resource_pool_is_in_state_1_when_totally_empty() -> TestResult {
    // Arrange
    with_multi_resource_pool(
        [18, 18, 18],
        |env, [(bucket1, _), (bucket2, _), (bucket3, _)], mut pool| {
            // Act
            let (pool_units, change) = {
                let bucket1 = bucket1.take(dec!(100), env)?;
                let bucket2 = bucket2.take(dec!(100), env)?;
                let bucket3 = bucket3.take(dec!(100), env)?;
                pool.contribute([bucket1, bucket2, bucket3], env)?
            };

            // Assert
            let pool_units_amount = pool_units.amount(env)?;
            assert_eq!(pool_units_amount, dec!(100));
            assert!(change.is_empty());

            TestResult::Ok(())
        },
    )
}

#[test]
fn multi_resource_pool_is_in_state_1_when_dusty() -> TestResult {
    for (create_dust1, create_dust2, create_dust3) in [
        (false, false, false),
        (false, false, true),
        (false, true, false),
        (false, true, true),
        (true, false, false),
        (true, false, true),
        (true, true, false),
    ] {
        with_multi_resource_pool(
            [18, 18, 18],
            |env, [(bucket1, _), (bucket2, _), (bucket3, _)], mut pool| {
                // Arrange
                {
                    for (create_dust, bucket) in [
                        (create_dust1, bucket1.clone()),
                        (create_dust2, bucket2.clone()),
                        (create_dust3, bucket3.clone()),
                    ] {
                        if create_dust {
                            let bucket = bucket.take(dec!(100), env)?;
                            pool.protected_deposit(bucket, env)?;
                        }
                    }
                }

                // Act
                let (pool_units, change) = {
                    let bucket1 = bucket1.take(dec!(100), env)?;
                    let bucket2 = bucket2.take(dec!(100), env)?;
                    let bucket3 = bucket3.take(dec!(100), env)?;
                    pool.contribute([bucket1, bucket2, bucket3], env)?
                };

                // Assert
                let pool_units_amount = pool_units.amount(env)?;
                assert_eq!(pool_units_amount, dec!(100));
                assert!(change.is_empty());

                TestResult::Ok(())
            },
        )?
    }
    Ok(())
}

#[test]
fn multi_resource_pool_is_in_state_2_when_all_reserves_are_empty() -> TestResult {
    // Arrange
    with_multi_resource_pool(
        [18, 18, 18],
        |env,
         [(bucket1, resource_address1), (bucket2, resource_address2), (bucket3, resource_address3)],
         mut pool| {
            {
                let bucket1 = bucket1.take(dec!(100), env)?;
                let bucket2 = bucket2.take(dec!(100), env)?;
                let bucket3 = bucket3.take(dec!(100), env)?;
                let _ = pool.contribute([bucket1, bucket2, bucket3], env)?;
            };
            {
                for address in [resource_address1, resource_address2, resource_address3] {
                    let _ = pool.protected_withdraw(address, dec!(100), Default::default(), env)?;
                }
            }

            // Act
            let rtn = {
                let bucket1 = bucket1.take(dec!(100), env)?;
                let bucket2 = bucket2.take(dec!(100), env)?;
                let bucket3 = bucket3.take(dec!(100), env)?;
                pool.contribute([bucket1, bucket2, bucket3], env)
            };

            // Assert
            assert_eq!(
                rtn,
                Err(RuntimeError::ApplicationError(
                    ApplicationError::MultiResourcePoolError(
                        MultiResourcePoolError::NoMinimumRatio,
                    ),
                )),
            );
            TestResult::Ok(())
        },
    )
}

#[test]
fn multi_resource_pool_is_in_state_3_when_some_reserves_are_empty() -> TestResult {
    for (remove_reserves1, remove_reserves2, remove_reserves3) in [
        (false, false, true),
        (false, true, false),
        (false, true, true),
        (true, false, false),
        (true, false, true),
        (true, true, false),
    ] {
        with_multi_resource_pool(
            [18, 18, 18],
            |env,
             [(bucket1, resource_address1), (bucket2, resource_address2), (bucket3, resource_address3)],
             mut pool| {
                // Arrange
                {
                    let bucket1 = bucket1.take(dec!(100), env)?;
                    let bucket2 = bucket2.take(dec!(100), env)?;
                    let bucket3 = bucket3.take(dec!(100), env)?;
                    let _ = pool.contribute([bucket1, bucket2, bucket3], env)?;
                };
                for (remove_reserves, resource_address) in [
                    (remove_reserves1, resource_address1),
                    (remove_reserves2, resource_address2),
                    (remove_reserves3, resource_address3),
                ] {
                    if remove_reserves {
                        let _ = pool.protected_withdraw(
                            resource_address,
                            dec!(100),
                            Default::default(),
                            env,
                        )?;
                    }
                }

                // Act
                let (pool_units, change) = {
                    let bucket1 = bucket1.take(dec!(100), env)?;
                    let bucket2 = bucket2.take(dec!(100), env)?;
                    let bucket3 = bucket3.take(dec!(100), env)?;
                    pool.contribute([bucket1, bucket2, bucket3], env)?
                };

                // Assert
                let empty_reserves = [remove_reserves1, remove_reserves2, remove_reserves3]
                    .into_iter()
                    .map(|remove_reserves| remove_reserves as usize)
                    .sum::<usize>();
                let pool_units_amount = pool_units.amount(env)?;

                assert_eq!(change.len(), empty_reserves);
                assert!(change
                    .into_iter()
                    .all(|bucket| bucket.amount(env).unwrap() == dec!(100)));
                assert_eq!(pool_units_amount, dec!(100));

                TestResult::Ok(())
            },
        )?
    }
    Ok(())
}

#[test]
fn multi_resource_pool_is_in_state_4_when_in_normal_operations() -> TestResult {
    // Arrange
    with_multi_resource_pool(
        [18, 18, 18],
        |env,
         [(bucket1, _), (bucket2, resource_address2), (bucket3, resource_address3)],
         mut pool| {
            {
                let bucket1 = bucket1.take(dec!(100), env)?;
                let bucket2 = bucket2.take(dec!(100), env)?;
                let bucket3 = bucket3.take(dec!(100), env)?;
                let _ = pool.contribute([bucket1, bucket2, bucket3], env)?;
            };

            // Act
            let (pool_units, change) = {
                let bucket1 = bucket1.take(dec!(100), env)?;
                let bucket2 = bucket2.take(dec!(110), env)?;
                let bucket3 = bucket3.take(dec!(120), env)?;
                pool.contribute([bucket1, bucket2, bucket3], env)?
            };

            // Assert
            let pool_units_amount = pool_units.amount(env)?;
            assert_eq!(pool_units_amount, dec!(100));

            assert_eq!(change.len(), 2);
            let change2 = change.first().unwrap();
            let change3 = change.get(1).unwrap();

            assert!(
                change2.resource_address(env)? == resource_address2
                    && change2.amount(env)? == dec!(10)
            );
            assert!(
                change3.resource_address(env)? == resource_address3
                    && change3.amount(env)? == dec!(20)
            );

            TestResult::Ok(())
        },
    )
}
