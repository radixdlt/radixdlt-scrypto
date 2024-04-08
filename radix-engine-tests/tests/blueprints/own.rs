use core::ops::*;

use radix_common::*;
use radix_common::constants::*;
use radix_common::data::manifest::*;
use radix_common::math::*;
use radix_common::prelude::*;
use radix_engine_interface::*;
use radix_engine_interface::api::*;
use radix_engine_interface::types::FromPublicKey;
use radix_transactions::builder::*;
use scrypto::prelude::{WORKTOP_BLUEPRINT, WORKTOP_DROP_IDENT};
use scrypto_test::ledger_simulator::*;

#[test]
fn mis_typed_own_passed_to_worktop_drop_function() {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Run manifest
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500)
            .take_from_worktop(XRD, Decimal::ZERO, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    RESOURCE_PACKAGE,
                    WORKTOP_BLUEPRINT,
                    WORKTOP_DROP_IDENT,
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    let error_message = receipt
        .expect_commit_failure()
        .outcome
        .expect_failure()
        .to_string();
    assert!(error_message.contains("ValidationError"))
}
