use radix_common::*;
use radix_common::data::manifest::*;
use radix_common::prelude::*;
use radix_engine_interface::*;
use radix_engine_interface::api::*;
use radix_engine_tests::common::*;
use radix_transactions::builder::*;
use scrypto_test::ledger_simulator::*;

#[test]
fn should_be_able_to_get_address_of_an_address_reservation() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("address_reservation"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "AddressReservation",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
