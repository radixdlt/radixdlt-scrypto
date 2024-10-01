use scrypto_test::prelude::*;

#[test]
fn invalid_blob_passes_validation_before_cuttlefish() {
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.from_bootstrap_to(ProtocolVersion::Bottlenose))
        .build();

    let raw_transaction = build_transaction_v1_with_invalid_blob(&mut ledger);

    raw_transaction
        .validate(ledger.transaction_validator())
        .expect("Expected transaction to be valid");

    ledger
        .execute_notarized_transaction(&raw_transaction)
        .expect_commit_failure();
}

#[test]
fn invalid_blob_does_not_pass_validation_at_latest_protocol_version() {
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let raw_transaction = build_transaction_v1_with_invalid_blob(&mut ledger);

    raw_transaction
        .validate(ledger.transaction_validator())
        .expect_err("Expected validation to fail after cuttlefish");
}

fn build_transaction_v1_with_invalid_blob(
    ledger: &mut DefaultLedgerSimulator,
) -> RawNotarizedTransaction {
    ledger
        .construct_unsigned_notarized_transaction_v1(
            ManifestBuilder::new_v1()
                .lock_fee_from_faucet()
                .call_method(XRD, "foobar", (ManifestBlobRef([0; 32]),))
                .build_no_validate(),
        )
        .to_raw()
        .unwrap()
}
