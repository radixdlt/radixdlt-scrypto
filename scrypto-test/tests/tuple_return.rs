use scrypto_test::prelude::*;
use tuple_return::tuple_return_test::*;

#[test]
fn tuple_returns_work_with_scrypto_test() {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = PackageFactory::compile_and_publish(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/blueprints/tuple-return"),
        &mut env,
        CompileProfile::Fast,
    )
    .unwrap();

    // Act
    let rtn = TupleReturn::instantiate(package_address, &mut env);

    // Assert
    assert!(rtn.is_ok())
}
