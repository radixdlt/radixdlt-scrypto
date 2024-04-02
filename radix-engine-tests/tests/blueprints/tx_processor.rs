use radix_engine::{
    blueprints::transaction_processor::{
        TransactionProcessorError, MAX_TOTAL_BLOB_SIZE_PER_INVOCATION,
    },
    errors::{ApplicationError, RuntimeError},
    vm::NoExtension,
};
use radix_engine_tests::include_local_wasm_str;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_transactions::prelude::*;
use scrypto::{crypto::hash, data::manifest::model::ManifestBlobRef, types::PackageAddress};
use scrypto_test::prelude::*;
use wabt::wat2wasm;

#[test]
fn test_blob_replacement_beyond_blob_size_limit() {
    // Arrange
    let mut sim = LedgerSimulatorBuilder::new().build();
    let package_address = publish_test_package(&mut sim);

    // Act
    let blob = vec![0u8; 1024];
    let blob_hash = hash(&blob);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Test",
            "f",
            ((0..MAX_TOTAL_BLOB_SIZE_PER_INVOCATION / blob.len() + 10)
                .map(|_| ManifestBlobRef(blob_hash.0))
                .collect::<Vec<ManifestBlobRef>>(),),
        )
        .then(|mut builder| {
            builder.add_blob(blob);
            builder
        })
        .build();
    let result = sim.execute_manifest(manifest, vec![]);

    // Assert
    result.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::TotalBlobSizeLimitExceeded
            ))
        )
    });
}

#[test]
fn test_blob_replacement_within_blob_size_limit() {
    // Arrange
    let mut sim = LedgerSimulatorBuilder::new().build();
    let package_address = publish_test_package(&mut sim);

    // Act
    let blob = vec![0u8; 1024];
    let blob_hash = hash(&blob);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Test",
            "f",
            ((0..MAX_TOTAL_BLOB_SIZE_PER_INVOCATION / blob.len() - 10)
                .map(|_| ManifestBlobRef(blob_hash.0))
                .collect::<Vec<ManifestBlobRef>>(),),
        )
        .call_function(
            package_address,
            "Test",
            "f",
            ((0..MAX_TOTAL_BLOB_SIZE_PER_INVOCATION / blob.len() - 10)
                .map(|_| ManifestBlobRef(blob_hash.0))
                .collect::<Vec<ManifestBlobRef>>(),),
        )
        .then(|mut builder| {
            builder.add_blob(blob);
            builder
        })
        .build();
    let result = sim.execute_manifest(manifest, vec![]);

    // Assert
    result.expect_commit_success();
}

fn publish_test_package(
    sim: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
) -> PackageAddress {
    let code = wat2wasm(include_local_wasm_str!("basic_package.wat")).unwrap();
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            single_function_package_definition("Test", "f"),
            BTreeMap::new(),
            OwnerRole::None,
        )
        .build();
    let receipt = sim.execute_manifest(manifest, vec![]);
    receipt.expect_commit(true).new_package_addresses()[0]
}
