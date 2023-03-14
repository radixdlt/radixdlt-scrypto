use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_balance_changes_when_success() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/balance_changes");

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 10u32.into())
            .call_function(
                package_address,
                "BalanceChangesTest",
                "instantiate",
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Call the put method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(FAUCET_COMPONENT, 100.into())
            .withdraw_from_account(account, RADIX_TOKEN, Decimal::ONE)
            .take_from_worktop(RADIX_TOKEN, |builder, bucket| {
                builder.call_method(component_address, "put", manifest_args!(bucket))
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}", receipt);

    // TODO: assert!
}
