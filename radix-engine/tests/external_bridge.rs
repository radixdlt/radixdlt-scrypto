#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

/// This tests the external_blueprint! and external_component! macros
#[test]
fn test_external_bridges() {
    // ARRANGE
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);

    // Part 1 - Upload the target and caller packages
    let target_package_address = test_runner.publish_package("component");
    fill_in_package_name_template(
        "./tests/external_blueprint_caller/src/external_blueprint_caller.rs.template",
        "./tests/external_blueprint_caller/src/external_blueprint_caller.rs",
        target_package_address
    ).expect("Package address rewrite expected to succeed");
    let caller_package_address = test_runner.publish_package("external_blueprint_caller");

    // Part 2 - Get a target component address
    let transaction1 = test_runner
        .new_transaction_builder()
        .call_function(target_package_address, "ExternalBlueprintTarget", "create", args![])
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt1 = test_runner.validate_and_execute(&transaction1);
    assert!(receipt1.result.is_ok());

    let target_component_address = receipt1.new_component_addresses[0];

    // Part 3 - Get the caller component address
    let transaction2 = test_runner
        .new_transaction_builder()
        .call_function(caller_package_address, "ExternalBlueprintCaller", "create", args![])
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt2 = test_runner.validate_and_execute(&transaction2);
    assert!(receipt2.result.is_ok());

    let caller_component_address = receipt2.new_component_addresses[0];

    // ACT
    let transaction3 = test_runner
        .new_transaction_builder()
        .call_method(caller_component_address, "run_tests_with_external_blueprint", args![])
        .call_method(caller_component_address, "run_tests_with_external_component", args![target_component_address])
        .build(test_runner.get_nonce([]))
        .sign([]);
    let receipt3 = test_runner.validate_and_execute(&transaction3);

    // ASSERT
    assert!(receipt3.result.is_ok());
}

fn fill_in_package_name_template(template_file_path: &str, code_file_path: &str, package_address: PackageAddress) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::Path;

    let package_address_hex = hex::encode(combine(1, &package_address.0));

    debug!("Copying template from {:?} to {:?} whilst updating package address to {}", Path::new(&template_file_path), Path::new(&code_file_path), package_address_hex);

    let mut template_file = File::open(&template_file_path)?;
    let mut template_file_contents = String::new();
    template_file.read_to_string(&mut template_file_contents)?;
    drop(template_file);

    let code_file_contents = template_file_contents
        .replace("%%PACKAGE_ADDRESS%%", &package_address_hex);

    // Recreate the file and dump the processed contents to it
    let mut code_file = File::create(&code_file_path)?;
    code_file.write(code_file_contents.as_bytes())?;
    drop(code_file);

    Result::Ok(())
}