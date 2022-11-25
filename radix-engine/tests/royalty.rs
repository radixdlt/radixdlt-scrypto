use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

fn run_manifest<F>(creator_function_name: &str, f: F) -> TransactionReceipt
where
    F: FnOnce(ComponentAddress) -> TransactionManifest,
{
    // Basic setup
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package and instantiate component
    let package_address = test_runner.compile_and_publish("./tests/blueprints/royalty");
    let receipt1 = test_runner.execute_manifest(
        ManifestBuilder::new(&NetworkDefinition::simulator())
            .lock_fee(account, 10u32.into())
            .call_function(
                package_address,
                "RoyaltyTest",
                creator_function_name,
                args!(),
            )
            .build(),
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    let component_address = receipt1
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    // Run the provided manifest
    let manifest = f(component_address);
    test_runner.execute_manifest(manifest, vec![])
}

#[test]
fn test_component_royalty() {
    let receipt = run_manifest(
        "create_component_with_royalty_enabled",
        |component_address| {
            ManifestBuilder::new(&NetworkDefinition::simulator())
                .lock_fee(FAUCET_COMPONENT, 100.into())
                .call_method(component_address, "paid_method", args!())
                .build()
        },
    );

    receipt.expect_commit_success();
    assert_eq!(dec!("0.1"), receipt.execution.fee_summary.royalty)
}
