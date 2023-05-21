use scrypto_unit::*;

#[test]
fn setting_single_epoch_succeeds() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let epoch = 12u32;

    // Act
    test_runner.set_current_epoch(epoch);

    // Assert
    assert_eq!(test_runner.get_current_epoch(), epoch);
}

#[test]
fn setting_multiple_epochs_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let epochs: Vec<u32> = vec![0, 100, 12, 19, 128, 4];

    for epoch in epochs.into_iter() {
        // Act
        test_runner.set_current_epoch(epoch);

        // Assert
        assert_eq!(test_runner.get_current_epoch(), epoch);
    }
}
