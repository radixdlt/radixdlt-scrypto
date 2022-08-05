use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::transaction::TransactionExecutor;
use radix_engine::wasm::*;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;
use transaction::model::TestTransaction;
use transaction::signing::EcdsaPrivateKey;

#[test]
fn test_hello() {
    // Set up environment.
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut wasm_instrumenter = WasmInstrumenter::new();
    let mut executor = TransactionExecutor::new(
        &mut substate_store,
        &mut wasm_engine,
        &mut wasm_instrumenter,
    );

    // Create a key pair
    let private_key = EcdsaPrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Publish package
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .publish_package(extract_package(compile_package!()).unwrap())
        .build();
    let package_address = executor
        .execute_and_commit(
            &TestTransaction::new(manifest, 1, vec![public_key]),
            &ExecutionConfig::default(),
        )
        .new_package_addresses[0];

    // Create an account
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_method(SYSTEM_COMPONENT, "free_xrd", to_struct!())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.new_account_with_resource(
                &rule!(require(NonFungibleAddress::from_public_key(&public_key))),
                bucket_id,
            )
        })
        .build();
    let account = executor
        .execute_and_commit(
            &TestTransaction::new(manifest, 2, vec![public_key]),
            &ExecutionConfig::default(),
        )
        .new_component_addresses[0];

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Hello", "instantiate_hello", to_struct!())
        .build();
    let receipt = executor.execute_and_commit(
        &TestTransaction::new(manifest, 3, vec![public_key]),
        &ExecutionConfig::default(),
    );
    println!("{:?}\n", receipt);
    receipt.expect_success();
    let component = receipt.new_component_addresses[0];

    // Test the `free_token` method.
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_method(component, "free_token", to_struct!())
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = executor.execute_and_commit(
        &TestTransaction::new(manifest, 4, vec![public_key]),
        &ExecutionConfig::default(),
    );
    println!("{:?}\n", receipt);
    receipt.expect_success();
}
