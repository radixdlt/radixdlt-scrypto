use radix_engine::engine::ScryptoInterpreter;
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::state_manager::StagedSubstateStoreManager;
use radix_engine::transaction::{
    execute_and_commit_transaction, ExecutionConfig, FeeReserveConfig,
};
use radix_engine::types::*;
use radix_engine::wasm::{
    DefaultWasmEngine, InstructionCostRules, WasmInstrumenter, WasmMeteringConfig,
};
use radix_engine_lib::core::NetworkDefinition;
use radix_engine_lib::resource::AccessRule;
use rand::Rng;
use rand_chacha;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use transaction::builder::{ManifestBuilder, TransactionBuilder};
use transaction::model::{NotarizedTransaction, TransactionHeader};
use transaction::signing::EcdsaSecp256k1PrivateKey;
use transaction::validation::{
    NotarizedTransactionValidator, TestIntentHashManager, TransactionValidator, ValidationConfig,
};

#[macro_export]
macro_rules! dec {
    ($x:literal) => {
        radix_engine_lib::math::Decimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a Decimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = radix_engine_lib::math::Decimal::from($base);
            if $shift >= 0 {
                base * radix_engine_lib::math::Decimal::try_from(
                    radix_engine_lib::math::I256::from(10u8)
                        .pow(u32::try_from($shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            } else {
                base / radix_engine_lib::math::Decimal::try_from(
                    radix_engine_lib::math::I256::from(10u8)
                        .pow(u32::try_from(-$shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            }
        }
    };
}

fn execute_single_transaction(transaction: NotarizedTransaction) {
    let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

    let executable = validator
        .validate(&transaction, &TestIntentHashManager::new())
        .unwrap();

    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut scrypto_interpreter = ScryptoInterpreter {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_instrumenter: WasmInstrumenter::default(),
        wasm_metering_config: WasmMeteringConfig::new(
            InstructionCostRules::tiered(1, 5, 10, 5000),
            512,
        ),
    };
    let execution_config = ExecutionConfig::standard();
    let fee_reserve_config = FeeReserveConfig::standard();

    let mut staged_store_manager = StagedSubstateStoreManager::new(&mut store);
    let staged_node = staged_store_manager.new_child_node(0);

    let mut staged_store = staged_store_manager.get_output_store(staged_node);
    execute_and_commit_transaction(
        &mut staged_store,
        &mut scrypto_interpreter,
        &fee_reserve_config,
        &execution_config,
        &executable,
    );
}

struct TransactionFuzzer {
    rng: ChaCha8Rng,
}

impl TransactionFuzzer {
    fn new() -> Self {
        let rng = ChaCha8Rng::seed_from_u64(1234);
        Self { rng }
    }

    fn next_transaction(&mut self) -> NotarizedTransaction {
        let mut builder = ManifestBuilder::new(&NetworkDefinition::simulator());
        let instruction_count = self.rng.gen_range(0u32..20u32);
        for _ in 0..instruction_count {
            let next = self.rng.gen_range(0u32..4u32);
            match next {
                0 => {
                    builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                        builder.call_function(
                            ACCOUNT_PACKAGE,
                            ACCOUNT_BLUEPRINT,
                            "new_with_resource",
                            args!(
                                AccessRule::AllowAll,
                                radix_engine_lib::resource::Bucket(bucket_id)
                            ),
                        )
                    });
                }
                1 => {
                    builder.call_function(
                        ACCOUNT_PACKAGE,
                        ACCOUNT_BLUEPRINT,
                        "new",
                        args!(AccessRule::AllowAll),
                    );
                }
                2 => {
                    builder.take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                        builder.call_function(
                            ACCOUNT_PACKAGE,
                            ACCOUNT_BLUEPRINT,
                            "new_with_resource",
                            args!(
                                AccessRule::AllowAll,
                                radix_engine_lib::resource::Bucket(bucket_id)
                            ),
                        )
                    });
                }
                3 => {
                    builder.call_method(FAUCET_COMPONENT, "lock_fee", args!(dec!("100")));
                }
                _ => panic!("Unexpected"),
            }
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
}

#[test]
fn simple_transaction_fuzz_test() {
    let mut fuzzer = TransactionFuzzer::new();
    let transactions: Vec<NotarizedTransaction> = (0..200)
        .into_iter()
        .map(|_| fuzzer.next_transaction())
        .collect();
    transactions.into_par_iter().for_each(|transaction| {
        execute_single_transaction(transaction);
    });
}
