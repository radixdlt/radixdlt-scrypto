use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_blob_replacement() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    // Act
    let blob = vec![0u8; 512 * 1024];
    let blob_hash = hash(&blob);
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            TRANSACTION_TRACKER,
            "test",
            ((0..1000)
                .map(|_| ManifestBlobRef(blob_hash.0))
                .collect::<Vec<ManifestBlobRef>>(),),
        )
        .then(|mut builder| {
            builder.add_blob(blob);
            builder
        })
        .build();
    let result = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    result.expect_commit_failure();
}
