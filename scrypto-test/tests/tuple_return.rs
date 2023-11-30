use tuple_return::test_bindings::*;
use scrypto::*;
use scrypto_test::prelude::*;

#[test]
fn tuple_returns_work_with_scrypto_test() {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address =
        Package::compile_and_publish(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/blueprints/tuple-return"), &mut env).unwrap();

    // Act
    let rtn = TupleReturn::instantiate(package_address, &mut env);

    // Assert
    assert!(rtn.is_ok())
}

