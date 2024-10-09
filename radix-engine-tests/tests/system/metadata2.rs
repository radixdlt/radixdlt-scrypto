use radix_common::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn test_url_validation() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("metadata2"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "M", "f", manifest_args!())
        .build();
    let start = std::time::Instant::now();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let end = std::time::Instant::now();

    // Usage:
    // ```
    // cargo bench --package radix-engine-tests --bench metadata_validation
    // cargo test --release --package radix-engine-tests --test system_folder -- system::metadata2::test_url_validation --exact --show-output
    // ```
    println!("{:?}", receipt);
    println!("{} ms", end.duration_since(start).as_millis());
}
