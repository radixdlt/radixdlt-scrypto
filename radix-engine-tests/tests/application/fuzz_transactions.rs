use radix_common::prelude::*;
use radix_engine::system::bootstrap::Bootstrapper;
use radix_engine::transaction::{
    execute_and_commit_transaction, ExecutionConfig,
};
use radix_engine::vm::wasm::{DefaultWasmEngine, WasmValidatorConfigV1};
use radix_engine::vm::{NoExtension, ScryptoVm, VmInit};
use radix_engine_interface::blueprints::resource::AccessRule;
use radix_engine_interface::prelude::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_transactions::model::{NotarizedTransactionV1, TransactionHeaderV1, TransactionPayload};
use radix_transactions::prelude::*;
use radix_transactions::validation::{
    NotarizedTransactionValidator, TransactionValidator, ValidationConfig,
};
use rand::Rng;
use rand_chacha;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;

struct TransactionFuzzer {
    rng: ChaCha8Rng,
    scrypto_vm: ScryptoVm<DefaultWasmEngine>,
    substate_db: InMemorySubstateDatabase,
}

impl TransactionFuzzer {
    fn new() -> Self {
        let rng = ChaCha8Rng::seed_from_u64(1234);

        let scrypto_vm = ScryptoVm {
            wasm_engine: DefaultWasmEngine::default(),
            wasm_validator_config: WasmValidatorConfigV1::new(),
        };
        let vms = VmInit::new(&scrypto_vm, NoExtension);

        let mut substate_db = InMemorySubstateDatabase::standard();
        Bootstrapper::new(NetworkDefinition::simulator(), &mut substate_db, vms, false)
            .bootstrap_test_default()
            .unwrap();

        Self {
            rng,
            scrypto_vm,
            substate_db,
        }
    }

    fn execute_single_transaction(&mut self, transaction: NotarizedTransactionV1) {
        let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

        let validated = validator
            .validate(transaction.prepare().expect("transaction to be preparable"))
            .expect("transaction to be validatable");

        let execution_config = ExecutionConfig::for_test_transaction();

        let vm_init = VmInit::new(&self.scrypto_vm, NoExtension);

        execute_and_commit_transaction(
            &mut self.substate_db,
            vm_init,
            &execution_config,
            validated.get_executable(),
        );
    }

    fn next_transaction(&mut self) -> NotarizedTransactionV1 {
        let mut builder = ManifestBuilder::new();
        let instruction_count = self.rng.gen_range(0u32..20u32);
        for _ in 0..instruction_count {
            let next = self.rng.gen_range(0u32..4u32);
            builder = match next {
                0 => builder.new_account_advanced(OwnerRole::Fixed(AccessRule::AllowAll), None),
                1 => builder.new_account_advanced(OwnerRole::Fixed(AccessRule::AllowAll), None),
                2 => builder.new_account_advanced(OwnerRole::Fixed(AccessRule::AllowAll), None),
                3 => builder.lock_fee(FAUCET, 100),
                _ => panic!("Unexpected"),
            }
        }

        let manifest = builder.build();
        let private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let header = TransactionHeaderV1 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(100),
            nonce: 5,
            notary_public_key: private_key.public_key().into(),
            notary_is_signatory: false,
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
    let transactions: Vec<NotarizedTransactionV1> = (0..50)
        .into_iter()
        .map(|_| fuzzer.next_transaction())
        .collect();
    transactions.into_iter().for_each(|transaction| {
        fuzzer.execute_single_transaction(transaction);
    });
}
