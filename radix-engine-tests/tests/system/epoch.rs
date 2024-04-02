use radix_common::types::Epoch;
use scrypto_test::prelude::*;

#[test]
fn setting_single_epoch_succeeds() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let epoch = Epoch::of(12);

    // Act
    ledger.set_current_epoch(epoch);

    // Assert
    assert_eq!(ledger.get_current_epoch(), epoch);
}

#[test]
fn setting_multiple_epochs_succeed() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let epochs = vec![0, 100, 12, 19, 128, 4]
        .into_iter()
        .map(Epoch::of)
        .collect::<Vec<_>>();

    for epoch in epochs.into_iter() {
        // Act
        ledger.set_current_epoch(epoch);

        // Assert
        assert_eq!(ledger.get_current_epoch(), epoch);
    }
}
