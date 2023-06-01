use radix_engine_common::types::Epoch;
use scrypto_unit::*;

#[test]
fn setting_single_epoch_succeeds() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let epoch = Epoch::of(12);

    // Act
    test_runner.set_current_epoch(epoch);

    // Assert
    assert_eq!(test_runner.get_current_epoch(), epoch);
}

#[test]
fn setting_multiple_epochs_succeed() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let epochs = vec![0, 100, 12, 19, 128, 4]
        .into_iter()
        .map(Epoch::of)
        .collect::<Vec<_>>();

    for epoch in epochs.into_iter() {
        // Act
        test_runner.set_current_epoch(epoch);

        // Assert
        assert_eq!(test_runner.get_current_epoch(), epoch);
    }
}
