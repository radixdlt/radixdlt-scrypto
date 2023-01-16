use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::engine::ScryptoInterpreter;
use radix_engine::ledger::*;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::wasm::WasmInstrumenter;
use radix_engine::wasm::{DefaultWasmEngine, WasmMeteringConfig};
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_interface::dec;
use radix_engine_interface::model::FromPublicKey;
use radix_engine_interface::rule;
use transaction::builder::ManifestBuilder;
use transaction::model::TestTransaction;
use transaction::signing::EcdsaSecp256k1PrivateKey;


#[derive(Eq, PartialEq, Hash, Clone)]
struct Bytes(usize);
impl std::fmt::Display for Bytes {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        if self.0 < 1_000 {
            write!(fmt, "{} B", self.0)
        } else if self.0 < 1_000_000 {
            write!(fmt, "{:.3} kB ({} Bytes)", self.0 as f64 / 1_000_f64, self.0)
        } else if self.0 < 1_000_000_000 {
            write!(fmt, "{:.3} MB ({} Bytes)", self.0 as f64 / 1_000_000_f64, self.0)
        } else {
            write!(fmt, "{:.3} GB ({} Bytes)", self.0 as f64 / 1_000_000_000_f64, self.0)
        }
    }
}
impl std::fmt::Debug for Bytes {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        write!(fmt, "{}", self) 
    }
}


struct MemInfoFramework {
    counter: Bytes,
    allocations: Vec<Bytes>
}
impl MemInfoFramework {
    pub fn new() -> Self {
        Self {
            counter: Bytes(0),
            allocations: Vec::new()
        }
    }
    pub fn add_measurement(&mut self, value: usize) {
        self.counter.0 += value;
        self.allocations.push(Bytes(value));
    }

    pub fn print_report(&self) {
        let mut map: HashMap<Bytes, usize> = HashMap::new();
        for i in self.allocations.iter() {
            map.entry(i.clone()).and_modify(|e| { *e += 1; }).or_insert(1);
        }

        println!("Iterations: {}", self.allocations.len());
        println!("Sum of allocated heap memory in all iterations: {}", self.counter);
        let x = Bytes(self.counter.0.div_euclid(self.allocations.len()));
        println!("Average allocation per iteration: {}", x);
        println!("Alocated memory chunks (size: count): {:#?}", map);
    }
}



fn mem_test(c: &mut Criterion) {
    // Set up environment.
    let mut fwk = MemInfoFramework::new();

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
    let manifest = ManifestBuilder::new()
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
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 100.into())
        .call_method(FAUCET_COMPONENT, "free", args!())
        .call_method(
            account1,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
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
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 100.into())
        .withdraw_from_account_by_amount(account1, dec!("0.000001"), RADIX_TOKEN)
        .call_method(
            account2,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
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

            fwk.add_measurement(receipt.execution.resources_heap_memory);

            receipt.expect_commit_success();
            nonce += 1;
        })
    });

    fwk.print_report();
}

criterion_group!(resource_usage, mem_test);
criterion_main!(resource_usage);
