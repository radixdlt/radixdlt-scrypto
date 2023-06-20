use radix_engine::types::*;
use scrypto_unit::*;
use utils::ContextualDisplay;

#[test]
fn scan_native_blueprint_schemas_and_highlight_unsafe_types() {
    let test_runner = TestRunner::builder().build();
    let bech32 = Bech32Encoder::for_simulator();

    let package_addresses = test_runner.find_all_packages();
    for package_address in package_addresses {
        let schemas_by_hash = test_runner.get_package_schema(&package_address);
        println!(
            "Found {} schemas for {}",
            schemas_by_hash.len(),
            package_address.to_string(&bech32)
        );
    }
}
