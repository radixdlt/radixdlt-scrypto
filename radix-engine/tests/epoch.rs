use scrypto_unit::*;

#[test]
fn setting_single_epoch_succeeds() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let epoch = 12u64;

    // Act
    test_runner.set_current_epoch(epoch);

    // Assert
    assert_eq!(test_runner.get_current_epoch(), epoch);
}

#[test]
fn setting_multiple_epochs_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let epochs = vec![0u64, 100u64, 12u64, 19u64, 128u64, 4u64];

    for epoch in epochs.into_iter() {
        // Act
        test_runner.set_current_epoch(epoch);

        // Assert
        assert_eq!(test_runner.get_current_epoch(), epoch);
    }
}
