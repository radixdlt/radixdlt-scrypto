use radiswap::radiswap_test::*;
use radix_engine::blueprints::pool::v1::substates::two_resource_pool::*;
use radix_engine::errors::*;
use scrypto_test::environment::*;
use scrypto_test::sdk::*;
use radix_common::prelude::*;
use radix_engine_interface::prelude::*;
use scrypto_test::*;
use radix_native_sdk::resource::*;

#[test]
fn simple_radiswap_test() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = PackageFactory::compile_and_publish(this_package!(), &mut env)?;

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(100, &mut env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(100, &mut env)?;

    let resource_address1 = bucket1.resource_address(&mut env)?;
    let resource_address2 = bucket2.resource_address(&mut env)?;

    let mut radiswap = Radiswap::new(
        OwnerRole::None,
        resource_address1,
        resource_address2,
        package_address,
        &mut env,
    )?;

    // Act
    let (pool_units, _change) = radiswap.add_liquidity(bucket1, bucket2, &mut env)?;

    // Assert
    assert_eq!(pool_units.amount(&mut env)?, dec!("100"));
    Ok(())
}

#[test]
fn reading_and_asserting_against_radiswap_pool_state() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = PackageFactory::compile_and_publish(this_package!(), &mut env)?;

    let bucket1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(100, &mut env)?;
    let bucket2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(100, &mut env)?;

    let resource_address1 = bucket1.resource_address(&mut env)?;
    let resource_address2 = bucket2.resource_address(&mut env)?;

    let mut radiswap = Radiswap::new(
        OwnerRole::None,
        resource_address1,
        resource_address2,
        package_address,
        &mut env,
    )?;

    // Act
    let _ = radiswap.add_liquidity(bucket1, bucket2, &mut env)?;

    // Assert
    let pool_component = env
        .with_component_state::<RadiswapState, _, _, _>(radiswap, |substate, _| {
            substate.pool_component.clone()
        })?;
    let (amount1, amount2) = env.with_component_state::<VersionedTwoResourcePoolState, _, _, _>(
        pool_component,
        |VersionedTwoResourcePoolState::V1(TwoResourcePoolStateV1 {
             vaults: [(_, vault1), (_, vault2)],
             ..
         }),
         env| { (vault1.amount(env).unwrap(), vault2.amount(env).unwrap()) },
    )?;
    assert_eq!(amount1, dec!("100"));
    assert_eq!(amount2, dec!("100"));

    Ok(())
}
