use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

const PACKAGE_ADDRESS_PLACE_HOLDER: [u8; NodeId::LENGTH] = [
    0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
    0x66, 0x77, 0x88, 0x99, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
];

// TODO: disallow this behavior once every package is forced to declare dependencies.
#[test]
fn test_call_package_address_undeclared() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address1 = test_runner.compile_and_publish("./tests/blueprints/cross_package_call");

    let (mut code, schema) = Compile::compile("./tests/blueprints/cross_package_call");
    let start = find_subsequence(&code, &PACKAGE_ADDRESS_PLACE_HOLDER).unwrap();
    code[start..start + PACKAGE_ADDRESS_PLACE_HOLDER.len()]
        .copy_from_slice(package_address1.as_ref());
    let package_address2 = test_runner.publish_package(
        code,
        schema,
        BTreeMap::new(),
        BTreeMap::new(),
        OwnerRole::None,
    );

    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address2,
            "Sample",
            "call_external_package",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
