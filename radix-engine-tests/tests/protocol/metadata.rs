use scrypto_test::prelude::*;

#[test]
fn metadata_is_changed_after_cuttlefish() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let MetadataValue::Url(url) = ledger.get_metadata(XRD.into(), "icon_url").unwrap() else {
        panic!()
    };

    // Assert
    assert_eq!(url.0, "https://assets.radixdlt.com/icons/icon-xrd.png")
}
