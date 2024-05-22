use scrypto_test::prelude::*;

use ${wasm_name}::hello_test::*;

#[test]
fn test_hello() {
    // Setup the environment
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Create an account
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    // Publish package
    // Package artifacts should be already available if `scrypto` command was used.
    // Macro `this_package_code_and_schema!()` reads artifacts in runtime.
    // Alternatively we could get artifacts in compile time (see below), but syntax
    // checker will complain, until package artifacts are not built (meaning until
    // first `scrypto build` or `scrypto test` successful invocation)
    // ```
    // let wasm = include_code!(".wasm");
    // let rpd = include_code!(".rpd");
    // ```
    let package_address = ledger.publish_package_simple(this_package_code_and_schema!());

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Hello",
            "instantiate_hello",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Test the `free_token` method.
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "free_token", manifest_args!())
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
}

#[test]
fn test_hello_with_test_environment() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    // Package should be already built if `scrypto` command was used
    let (wasm, rpd) = this_package_code_and_schema!();
    let package_address = PackageFactory::publish_simple(wasm, rpd, &mut env)?;

    let mut hello = Hello::instantiate_hello(package_address, &mut env)?;

    // Act
    let bucket = hello.free_token(&mut env)?;

    // Assert
    let amount = bucket.amount(&mut env)?;
    assert_eq!(amount, dec!("1"));

    Ok(())
}
