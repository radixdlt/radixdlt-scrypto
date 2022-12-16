use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto_unit::*;

// TODO: Test should be moved to a higher level package (e.g. scrypto-tests)
#[test]
fn can_compile_and_publish_blueprint_with_import() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Act
    test_runner.compile_and_publish("./tests/blueprints/import");
}
