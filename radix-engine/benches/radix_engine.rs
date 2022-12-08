use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::engine::ScryptoInterpreter;
use radix_engine::ledger::*;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::wasm::WasmInstrumenter;
use radix_engine::wasm::{DefaultWasmEngine, WasmMeteringConfig};
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use radix_engine_interface::dec;
use radix_engine_interface::model::FromPublicKey;
use radix_engine_interface::rule;
use transaction::builder::ManifestBuilder;
use transaction::model::TestTransaction;
use transaction::signing::EcdsaSecp256k1PrivateKey;

fn bench_transfer(c: &mut Criterion) {
    // Set up environment.
    let mut scrypto_interpreter = ScryptoInterpreter {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_instrumenter: WasmInstrumenter::default(),
        wasm_metering_config: WasmMeteringConfig::V0,
    };
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap(&scrypto_interpreter);

    // Create a key pair
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Create two accounts
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 100.into())
        .call_method(FAUCET_COMPONENT, "free", args!())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.new_account_with_resource(
                &rule!(require(NonFungibleAddress::from_public_key(&public_key))),
                bucket_id,
            )
        })
        .build();

    let account1 = execute_and_commit_transaction(
        &mut substate_store,
        &mut scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::default(),
        &TestTransaction::new(manifest.clone(), 1, DEFAULT_COST_UNIT_LIMIT)
            .get_executable(vec![NonFungibleAddress::from_public_key(&public_key)]),
    )
    .expect_commit()
    .entity_changes
    .new_component_addresses[0];

    let account2 = execute_and_commit_transaction(
        &mut substate_store,
        &mut scrypto_interpreter,
        &FeeReserveConfig::default(),
        &ExecutionConfig::default(),
        &TestTransaction::new(manifest, 2, DEFAULT_COST_UNIT_LIMIT)
            .get_executable(vec![NonFungibleAddress::from_public_key(&public_key)]),
    )
    .expect_commit()
    .entity_changes
    .new_component_addresses[0];

    // Fill first account
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 100.into())
        .call_method(FAUCET_COMPONENT, "free", args!())
        .call_method(
            account1,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    for nonce in 0..1000 {
        execute_and_commit_transaction(
            &mut substate_store,
            &mut scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                .get_executable(vec![NonFungibleAddress::from_public_key(&public_key)]),
        )
        .expect_commit();
    }

    // Create a transfer manifest
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 100.into())
        .withdraw_from_account_by_amount(account1, dec!("0.000001"), RADIX_TOKEN)
        .call_method(
            account2,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();

    // Loop
    let mut nonce = 3;
    c.bench_function("Transfer", |b| {
        b.iter(|| {
            let receipt = execute_and_commit_transaction(
                &mut substate_store,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(vec![NonFungibleAddress::from_public_key(&public_key)]),
            );
            receipt.expect_commit_success();
            nonce += 1;
        })
    });
}

criterion_group!(radix_engine, bench_transfer);
criterion_main!(radix_engine);
