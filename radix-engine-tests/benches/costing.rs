use criterion::{criterion_group, criterion_main, Criterion};
use paste::paste;
use radix_common::crypto::{verify_and_recover_secp256k1, verify_secp256k1};
use radix_common::prelude::*;
use radix_engine::{
    utils::ExtractSchemaError,
    vm::{
        wasm::{DefaultWasmEngine, ScryptoV1WasmValidator, WasmEngine, WasmModule},
        ScryptoVmVersion,
    },
};
use radix_engine_interface::prelude::*;
use radix_engine_tests::common::*;
use radix_substate_store_queries::typed_substate_layout::{CodeHash, PackageDefinition};
use sbor::rust::iter;
use scrypto_test::prelude::*;
use wabt::wat2wasm;

fn generate_interesting_bytes_of_length(length: usize) -> Vec<u8> {
    include_workspace_asset_bytes!("radix-transaction-scenarios", "radiswap.rpd")
        .iter()
        .cycle()
        .take(length)
        .cloned()
        .collect()
}

fn bench_decode_rpd_to_manifest_value(c: &mut Criterion) {
    let payload = include_workspace_asset_bytes!("radix-transaction-scenarios", "radiswap.rpd");
    println!("Payload size: {}", payload.len());
    c.bench_function("costing::decode_rpd_to_manifest_value", |b| {
        b.iter(|| manifest_decode::<ManifestValue>(payload))
    });
}

fn bench_decode_rpd_to_manifest_raw_value(c: &mut Criterion) {
    let payload = include_workspace_asset_bytes!("radix-transaction-scenarios", "radiswap.rpd");
    println!("Payload size: {}", payload.len());
    c.bench_function("costing::decode_rpd_to_manifest_raw_value", |b| {
        b.iter(|| manifest_decode::<ManifestRawValue>(payload))
    });
}

fn bench_decode_encoded_u8_array_to_manifest_value(c: &mut Criterion) {
    let example_bytes = generate_interesting_bytes_of_length(1000000);
    let payload = manifest_encode(&example_bytes).unwrap();
    println!("Payload size: {}", payload.len());
    c.bench_function("costing::decode_encoded_u8_array_to_manifest_value", |b| {
        b.iter(|| manifest_decode::<ManifestValue>(&payload))
    });
}

fn bench_decode_encoded_u8_array_to_manifest_raw_value(c: &mut Criterion) {
    let example_bytes = generate_interesting_bytes_of_length(1000000);
    let payload = manifest_encode(&example_bytes).unwrap();
    println!("Payload size: {}", payload.len());
    c.bench_function(
        "costing::decode_encoded_u8_array_to_manifest_raw_value",
        |b| b.iter(|| manifest_decode::<ManifestRawValue>(&payload)),
    );
}

fn bench_decode_encoded_i8_array_to_manifest_value(c: &mut Criterion) {
    let example_i8_array = generate_interesting_bytes_of_length(1000000)
        .into_iter()
        .map(|b| i8::from_be_bytes([b]))
        .collect::<Vec<_>>();
    let payload = manifest_encode(&example_i8_array).unwrap();
    println!("Payload size: {}", payload.len());
    c.bench_function("costing::decode_encoded_i8_array_to_manifest_value", |b| {
        b.iter(|| manifest_decode::<ManifestValue>(&payload))
    });
}

fn bench_decode_encoded_i8_array_to_manifest_raw_value(c: &mut Criterion) {
    let example_i8_array = generate_interesting_bytes_of_length(1000000)
        .into_iter()
        .map(|b| i8::from_be_bytes([b]))
        .collect::<Vec<_>>();
    let payload = manifest_encode(&example_i8_array).unwrap();
    println!("Payload size: {}", payload.len());
    c.bench_function(
        "costing::decode_encoded_i8_array_to_manifest_raw_value",
        |b| b.iter(|| manifest_decode::<ManifestRawValue>(&payload)),
    );
}

fn bench_decode_encoded_tuple_array_to_manifest_value(c: &mut Criterion) {
    let value = generate_interesting_bytes_of_length(1000000)
        .into_iter()
        .map(|b| (b,))
        .collect::<Vec<_>>();
    let payload = manifest_encode(&value).unwrap();
    println!("Payload size: {}", payload.len());
    c.bench_function(
        "costing::decode_encoded_tuple_array_to_manifest_value",
        |b| b.iter(|| manifest_decode::<ManifestValue>(&payload)),
    );
}

fn bench_decode_encoded_tuple_array_to_manifest_raw_value(c: &mut Criterion) {
    let value = generate_interesting_bytes_of_length(1000000)
        .into_iter()
        .map(|b| (b,))
        .collect::<Vec<_>>();
    let payload = manifest_encode(&value).unwrap();
    println!("Payload size: {}", payload.len());
    c.bench_function(
        "costing::decode_encoded_tuple_array_to_manifest_raw_value",
        |b| b.iter(|| manifest_decode::<ManifestRawValue>(&payload)),
    );
}

fn bench_validate_sbor_payload(c: &mut Criterion) {
    let package_definition = manifest_decode::<PackageDefinition>(include_workspace_asset_bytes!(
        "radix-transaction-scenarios",
        "radiswap.rpd"
    ))
    .unwrap();
    let payload = scrypto_encode(&package_definition).unwrap();
    println!("Payload size: {}", payload.len());
    let (index, schema) =
        generate_full_schema_from_single_type::<PackageDefinition, ScryptoCustomSchema>();

    c.bench_function("costing::validate_sbor_payload", |b| {
        b.iter(|| {
            validate_payload_against_schema::<ScryptoCustomExtension, _>(
                &payload,
                schema.v1(),
                index,
                &(),
                SCRYPTO_SBOR_V1_MAX_DEPTH,
            )
        })
    });
}

fn bench_validate_sbor_payload_bytes(c: &mut Criterion) {
    let payload = scrypto_encode(include_workspace_asset_bytes!(
        "radix-transaction-scenarios",
        "radiswap.rpd"
    ))
    .unwrap();
    println!("Payload size: {}", payload.len());
    let (index, schema) = generate_full_schema_from_single_type::<Vec<u8>, ScryptoCustomSchema>();

    c.bench_function("costing::validate_sbor_payload_bytes", |b| {
        b.iter(|| {
            validate_payload_against_schema::<ScryptoCustomExtension, _>(
                &payload,
                schema.v1(),
                index,
                &(),
                SCRYPTO_SBOR_V1_MAX_DEPTH,
            )
        })
    });
}

fn bench_validate_secp256k1(c: &mut Criterion) {
    let message = "m".repeat(1_000_000);
    let message_hash = hash(message.as_bytes());
    let signer = Secp256k1PrivateKey::from_u64(123123123123).unwrap();
    let signature = signer.sign(&message_hash);

    c.bench_function("costing::validate_secp256k1", |b| {
        b.iter(|| {
            let public_key = verify_and_recover_secp256k1(&message_hash, &signature).unwrap();
            verify_secp256k1(&message_hash, &public_key, &signature);
        })
    });
}

// Usage: cargo bench --bench costing -- spin_loop_v1
// Note that this benchmark replaces the `spin_loop` before this commit, which uses NoOpRuntime
fn bench_spin_loop_v1(c: &mut Criterion) {
    // Prepare code
    let code =
        wat2wasm(&include_local_wasm_str!("loop.wat").replace("${n}", &i32::MAX.to_string()))
            .unwrap();
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackagePublishingSource::PublishExisting(
        code,
        single_function_package_definition("Test", "f"),
    ));

    let manifest = ManifestBuilder::new()
        // First, lock the fee so that the loan will be repaid
        .lock_fee_from_faucet()
        // Now spin-loop to wait for the fee loan to burn through
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();

    // The transaction failed, consuming almost all execution cost units.
    assert!(
        ledger
            .execute_manifest(manifest.clone(), [])
            .fee_summary
            .total_execution_cost_units_consumed
            >= 99_000_000
    );

    c.bench_function("costing::spin_loop_v1", |b| {
        b.iter(|| ledger.execute_manifest(manifest.clone(), []))
    });
}

// Usage: cargo bench --bench costing -- spin_loop_v2
// Different from spin_loop_v1, this is the smallest possible loop.
// There is only one instruction `br` per iteration.
// It's extremely helpful for stress testing the `consume_wasm_execution_units` host function.
fn bench_spin_loop_v2(c: &mut Criterion) {
    let code = wat2wasm(&include_local_wasm_str!("loop_v2.wat")).unwrap();
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackagePublishingSource::PublishExisting(
        code,
        single_function_package_definition("Test", "f"),
    ));

    let manifest = ManifestBuilder::new()
        // First, lock the fee so that the loan will be repaid
        .lock_fee_from_faucet()
        // Now spin-loop to wait for the fee loan to burn through
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();

    // The transaction failed, consuming almost all execution cost units.
    assert!(
        ledger
            .execute_manifest(manifest.clone(), [])
            .fee_summary
            .total_execution_cost_units_consumed
            >= 99_000_000
    );

    c.bench_function("costing::spin_loop_v2", |b| {
        b.iter(|| ledger.execute_manifest(manifest.clone(), []))
    });
}

// Usage: cargo bench --bench costing -- scrypto_sha256
fn bench_scrypto_sha256(c: &mut Criterion) {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("costing_sha256"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();

    // The transaction failed, consuming almost all execution cost units.
    assert!(
        ledger
            .execute_manifest(manifest.clone(), [])
            .fee_summary
            .total_execution_cost_units_consumed
            >= 99_000_000
    );

    c.bench_function("costing::scrypto_sha256", |b| {
        b.iter(|| ledger.execute_manifest(manifest.clone(), []))
    });
}

// Usage: cargo bench --bench costing -- scrypto_sbor_decode
fn bench_scrypto_sbor_decode(c: &mut Criterion) {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("costing_sbor_decode"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();

    // The transaction failed, consuming almost all execution cost units.
    assert!(
        ledger
            .execute_manifest(manifest.clone(), [])
            .fee_summary
            .total_execution_cost_units_consumed
            >= 99_000_000
    );

    c.bench_function("costing::scrypto_sbor_decode", |b| {
        b.iter(|| ledger.execute_manifest(manifest.clone(), []))
    });
}

// Usage: cargo bench --bench costing -- scrypto_malloc
fn bench_scrypto_malloc(c: &mut Criterion) {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("costing_malloc"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .then(|mut builder| {
            for _ in 0..100 {
                builder = builder.call_function(
                    package_address,
                    "Test",
                    "f",
                    manifest_args!(100_000usize, 5usize),
                );
            }
            builder
        })
        .build();

    // The transaction failed, consuming almost all execution cost units.
    assert!(
        ledger
            .execute_manifest(manifest.clone(), [])
            .fee_summary
            .total_execution_cost_units_consumed
            >= 99_000_000
    );

    c.bench_function("costing::scrypto_malloc", |b| {
        b.iter(|| ledger.execute_manifest(manifest.clone(), []))
    });
}

macro_rules! bench_instantiate {
    ($what:literal) => {
        paste! {
        fn [< bench_instantiate_ $what >] (c: &mut Criterion) {
            // Prepare code
            let code = include_workspace_asset_bytes!("radix-transaction-scenarios", concat!($what, ".wasm"));

            // Instrument
            let validator = ScryptoV1WasmValidator::new(ScryptoVmVersion::latest());
            let instrumented_code = validator
                .validate(code, iter::empty())
                .map_err(|e| ExtractSchemaError::InvalidWasm(e))
                .unwrap()
                .0;

            c.bench_function(concat!("costing::instantiate_", $what), |b| {
                b.iter(|| {
                    let wasm_engine = DefaultWasmEngine::default();
                    wasm_engine.instantiate(CodeHash(Hash([0u8; 32])), &instrumented_code);
                })
            });

            println!("Code length: {}", instrumented_code.len());
        }
        }
    };
}

bench_instantiate!("radiswap");
bench_instantiate!("flash_loan");

fn bench_validate_wasm(c: &mut Criterion) {
    let code = include_workspace_asset_bytes!("radix-transaction-scenarios", "radiswap.wasm");
    let definition: PackageDefinition = manifest_decode(include_workspace_asset_bytes!(
        "radix-transaction-scenarios",
        "radiswap.rpd"
    ))
    .unwrap();

    c.bench_function("costing::validate_wasm", |b| {
        b.iter(|| {
            ScryptoV1WasmValidator::new(ScryptoVmVersion::latest())
                .validate(code, definition.blueprints.values())
                .unwrap()
        })
    });

    println!("Code length: {}", code.len());
}

fn bench_deserialize_wasm(c: &mut Criterion) {
    let code = include_workspace_asset_bytes!("radix-transaction-scenarios", "radiswap.wasm");

    c.bench_function("costing::deserialize_wasm", |b| {
        b.iter(|| WasmModule::init(code).unwrap())
    });
}

fn bench_prepare_wasm(c: &mut Criterion) {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let code =
        include_workspace_asset_bytes!("radix-transaction-scenarios", "radiswap.wasm").to_vec();
    let package_definition: PackageDefinition = manifest_decode(include_workspace_asset_bytes!(
        "radix-transaction-scenarios",
        "radiswap.rpd"
    ))
    .unwrap();

    c.bench_function("costing::bench_prepare_wasm", |b| {
        b.iter(|| {
            let (pk1, _, _) = ledger.new_allocated_account();
            ledger.publish_package(
                (code.clone(), package_definition.clone()),
                btreemap!(),
                OwnerRole::Updatable(rule!(require(signature(&pk1)))),
            );
        })
    });
}

fn bench_execute_transaction_creating_big_vec_substates(c: &mut Criterion) {
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let (code, definition) = PackageLoader::get("transaction_limits");
    let package_address =
        ledger.publish_package((code, definition), BTreeMap::new(), OwnerRole::None);

    let substate_sizes = [
        1000,
        100000,
        MAX_SUBSTATE_VALUE_SIZE - 100,
        MAX_SUBSTATE_VALUE_SIZE - 100,
        MAX_SUBSTATE_VALUE_SIZE - 100,
        MAX_SUBSTATE_VALUE_SIZE - 100,
    ];

    c.bench_function(
        "costing::execute_transaction_creating_big_vec_substates",
        |b| {
            b.iter(|| {
                ledger
                    .call_function(
                        package_address,
                        "TransactionLimitSubstateTest",
                        "write_large_values",
                        manifest_args!(&substate_sizes),
                    )
                    .expect_commit_success();
            })
        },
    );
}

fn bench_execute_transaction_reading_big_vec_substates(c: &mut Criterion) {
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let (code, definition) = PackageLoader::get("transaction_limits");
    let package_address =
        ledger.publish_package((code, definition), BTreeMap::new(), OwnerRole::None);

    let substate_sizes = [
        1000,
        100000,
        MAX_SUBSTATE_VALUE_SIZE - 100,
        MAX_SUBSTATE_VALUE_SIZE - 100,
        MAX_SUBSTATE_VALUE_SIZE - 100,
        MAX_SUBSTATE_VALUE_SIZE - 100,
    ];
    let component_address = ledger
        .call_function(
            package_address,
            "TransactionLimitSubstateTest",
            "write_large_values",
            manifest_args!(&substate_sizes),
        )
        .expect_commit_success()
        .new_component_addresses()[0];

    let substates_to_read = substate_sizes.len() as u32;

    c.bench_function(
        "costing::execute_transaction_reading_big_vec_substates",
        |b| {
            b.iter(|| {
                ledger
                    .call_method(
                        component_address,
                        "read_values",
                        manifest_args!(substates_to_read),
                    )
                    .expect_commit_success();
            })
        },
    );
}

criterion_group!(
    costing,
    bench_decode_rpd_to_manifest_value,
    bench_decode_rpd_to_manifest_raw_value,
    bench_decode_encoded_u8_array_to_manifest_value,
    bench_decode_encoded_u8_array_to_manifest_raw_value,
    bench_decode_encoded_i8_array_to_manifest_value,
    bench_decode_encoded_i8_array_to_manifest_raw_value,
    bench_decode_encoded_tuple_array_to_manifest_value,
    bench_decode_encoded_tuple_array_to_manifest_raw_value,
    bench_validate_sbor_payload,
    bench_validate_sbor_payload_bytes,
    bench_validate_secp256k1,
    bench_instantiate_radiswap,
    bench_instantiate_flash_loan,
    bench_deserialize_wasm,
    bench_validate_wasm,
    bench_prepare_wasm,
    bench_execute_transaction_creating_big_vec_substates,
    bench_execute_transaction_reading_big_vec_substates,
);

// This group is for longer benchmarks, which might be counted in seconds
criterion_group!(
    name = costing_long;
    config = Criterion::default()
                .sample_size(20)
                .measurement_time(core::time::Duration::from_secs(20))
                .warm_up_time(core::time::Duration::from_millis(3000));
    targets = bench_spin_loop_v1,bench_spin_loop_v2,bench_scrypto_sha256,bench_scrypto_sbor_decode,bench_scrypto_malloc
);
criterion_main!(costing, costing_long);
