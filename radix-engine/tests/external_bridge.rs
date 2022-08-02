#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::address::Bech32Encoder;
use scrypto::core::Network;
use scrypto::{prelude::*, to_struct};
use transaction::builder::ManifestBuilder;

/// This tests the external_blueprint! and external_component! macros
#[test]
fn test_external_bridges() {
    // ARRANGE
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);

    // Part 1 - Upload the target and caller packages
    let target_package_address = test_runner.extract_and_publish_package("component");
    fill_in_package_name_template(
        "./tests/external_blueprint_caller/src/external_blueprint_caller.rs.template",
        "./tests/external_blueprint_caller/src/external_blueprint_caller.rs",
        target_package_address,
    )
    .expect("Package address rewrite expected to succeed");
    let caller_package_address =
        test_runner.extract_and_publish_package("external_blueprint_caller");

    // Part 2 - Get a target component address
    let manifest1 = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(
            target_package_address,
            "ExternalBlueprintTarget",
            "create",
            to_struct!(),
        )
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_success();

    let target_component_address = receipt1.new_component_addresses[0];

    // Part 3 - Get the caller component address
    let manifest2 = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(
            caller_package_address,
            "ExternalBlueprintCaller",
            "create",
            to_struct!(),
        )
        .build();
    let receipt2 = test_runner.execute_manifest(manifest2, vec![]);
    receipt2.expect_success();

    let caller_component_address = receipt2.new_component_addresses[0];

    // ACT
    let manifest3 = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_method(
            caller_component_address,
            "run_tests_with_external_blueprint",
            to_struct!(),
        )
        .call_method(
            caller_component_address,
            "run_tests_with_external_component",
            to_struct!(target_component_address),
        )
        .build();
    let receipt3 = test_runner.execute_manifest(manifest3, vec![]);

    // ASSERT
    receipt3.expect_success();
}

fn fill_in_package_name_template(
    template_file_path: &str,
    code_file_path: &str,
    package_address: PackageAddress,
) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::Path;

    let bech32_encoder = Bech32Encoder::new_from_network(&Network::LocalSimulator);

    let package_address_string = bech32_encoder
        .encode_package_address(&package_address)
        .unwrap();

    println!(
        "Copying template from {:?} to {:?} whilst updating package address to {}",
        Path::new(&template_file_path),
        Path::new(&code_file_path),
        package_address_string
    );

    let mut template_file = File::open(&template_file_path)?;
    let mut template_file_contents = String::new();
    template_file.read_to_string(&mut template_file_contents)?;
    drop(template_file);

    let code_file_contents =
        template_file_contents.replace("%%PACKAGE_ADDRESS%%", &package_address_string);

    // Recreate the file and dump the processed contents to it
    let mut code_file = File::create(&code_file_path)?;
    code_file.write(code_file_contents.as_bytes())?;
    drop(code_file);

    Result::Ok(())
}
