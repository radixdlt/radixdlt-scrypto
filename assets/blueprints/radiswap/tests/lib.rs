use native_sdk::resource::{NativeBucket, ResourceManager};
use radiswap::test_bindings::*;
use scrypto::*;
use scrypto_test::prelude::*;

#[test]
fn simple_radiswap_test() {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = Package::compile_and_publish(this_package!(), &mut env).unwrap();
    let (resource1, bucket1) = ResourceManager::new_fungible_with_initial_supply(
        OwnerRole::None,
        true,
        18,
        dec!("100"),
        Default::default(),
        MetadataInit::default(),
        Default::default(),
        &mut env,
    )
    .unwrap();
    let (resource2, bucket2) = ResourceManager::new_fungible_with_initial_supply(
        OwnerRole::None,
        true,
        18,
        dec!("100"),
        Default::default(),
        MetadataInit::default(),
        Default::default(),
        &mut env,
    )
    .unwrap();

    let mut radiswap = Radiswap::new(
        OwnerRole::None,
        resource1.0,
        resource2.0,
        package_address,
        &mut env,
    )
    .unwrap();

    // Act
    let (pool_units, _change) = radiswap.add_liquidity(bucket1, bucket2, &mut env).unwrap();

    // Assert
    assert_eq!(pool_units.amount(&mut env).unwrap(), dec!("100"));
}
