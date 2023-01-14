use scrypto_unit::*;

#[test]
fn test_simple_deterministic_execution() {
    // Arrange
    let mut test_runner0 = TestRunner::new(true);
    let mut test_runner1 = TestRunner::new(true);

    // Act
    let (public_key0, _, account0) = test_runner0.new_allocated_account();
    let (public_key1, _, account1) = test_runner1.new_allocated_account();

    // Assert
    assert_eq!(public_key0, public_key1);
    assert_eq!(account0, account1);
    test_runner0
        .substate_store()
        .assert_eq(test_runner1.substate_store());
}
