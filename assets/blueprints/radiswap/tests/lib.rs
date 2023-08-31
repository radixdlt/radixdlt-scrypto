use radiswap::test_bindings::*;
use scrypto::*;
use scrypto_test::prelude::*;

#[test]
fn simple_radiswap_test() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = Package::compile_and_publish(this_package!(), &mut env)?;

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
    let package_address = Package::compile_and_publish(this_package!(), &mut env)?;

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
    let radiswap_state = env.read_component_state::<RadiswapState, _>(radiswap)?;

    let VersionedTwoResourcePoolState::V1(TwoResourcePoolSubstate {
        vaults: [(_, vault1), (_, vault2)],
        ..
    }) = env.read_component_state(radiswap_state.pool_component)?;

    // Assert
    let amount1 = vault1.amount(&mut env)?;
    let amount2 = vault2.amount(&mut env)?;
    assert_eq!(amount1, dec!("100"));
    assert_eq!(amount2, dec!("100"));

    Ok(())
}
