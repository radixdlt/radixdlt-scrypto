#![cfg_attr(feature = "libfuzzer-sys", no_main)]

#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;

#[cfg(feature = "afl")]
use afl::fuzz;

#[cfg(feature = "simple-fuzzer")]
mod simple_fuzzer;

use radix_engine::kernel::interpreters::ScryptoInterpreter;
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::transaction::{
    execute_and_commit_transaction, ExecutionConfig, FeeReserveConfig,
};
use radix_engine::types::*;
use radix_engine::wasm::{DefaultWasmEngine, WasmInstrumenter, WasmMeteringConfig};
use radix_engine_interface::blueprints::resource::AccessRule;
use transaction::builder::{ManifestBuilder, TransactionBuilder};
use transaction::model::{NotarizedTransaction, TransactionHeader};
use transaction::signing::EcdsaSecp256k1PrivateKey;
use transaction::validation::{
    NotarizedTransactionValidator, TestIntentHashManager, TransactionValidator, ValidationConfig,
};

fn execute_single_transaction(transaction: NotarizedTransaction) {
    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

    let executable = validator
        .validate(&transaction, 0, &TestIntentHashManager::new())
        .unwrap();

    let scrypto_interpreter = ScryptoInterpreter {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_instrumenter: WasmInstrumenter::default(),
        wasm_metering_config: WasmMeteringConfig::V0,
    };
    let mut store = TypedInMemorySubstateStore::with_bootstrap(&scrypto_interpreter);
    let execution_config = ExecutionConfig::default();
    let fee_reserve_config = FeeReserveConfig::default();

    execute_and_commit_transaction(
        &mut store,
        &scrypto_interpreter,
        &fee_reserve_config,
        &execution_config,
        &executable,
    );
}

fn generate_transaction(data: &[u8]) -> NotarizedTransaction {
    let mut builder = ManifestBuilder::new();

    let mut i = 0;

    while i < data.len() {
        match data[i] % 2 {
            0 => {
                builder.new_account(AccessRule::AllowAll);
            }
            1 => {
                builder.call_method(FAUCET_COMPONENT, "lock_fee", args!(dec!("100")));
            }
            _ => panic!("Unexpected"),
        }
        i += 1;
    }
    let manifest = builder.build();
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();

    let header = TransactionHeader {
        version: 1,
        network_id: NetworkDefinition::simulator().id,
        start_epoch_inclusive: 0,
        end_epoch_exclusive: 100,
        nonce: 5,
        notary_public_key: private_key.public_key().into(),
        notary_as_signatory: false,
        cost_unit_limit: 10_000_000,
        tip_percentage: 0,
    };

    TransactionBuilder::new()
        .header(header)
        .manifest(manifest)
        .sign(&private_key)
        .notarize(&private_key)
        .build()
}

fn fuzz_transaction(data: &[u8]) {
    let transaction = generate_transaction(data);
    execute_single_transaction(transaction);
}

// Fuzzer entry points
#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|data: &[u8]| {
    fuzz_transaction(data);
});

#[cfg(feature = "afl")]
fn main() {
    fuzz!(|data: &[u8]| {
        fuzz_transaction(data);
    });
}

#[cfg(feature = "simple-fuzzer")]
fn main() {
    simple_fuzzer::fuzz(|data: &[u8]| {
        fuzz_transaction(data);
    });
}
