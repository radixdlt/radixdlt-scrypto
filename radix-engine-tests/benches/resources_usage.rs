use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::kernel::interpreters::ScryptoInterpreter;
use radix_engine::ledger::*;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig, ResourcesUsage};
use radix_engine::types::*;
use radix_engine::wasm::WasmInstrumenter;
use radix_engine::wasm::{DefaultWasmEngine, WasmMeteringConfig};
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::dec;
use radix_engine_interface::rule;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::TestTransaction;

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
struct Bytes(usize);
impl std::fmt::Display for Bytes {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        if self.0 < 1_000 {
            write!(fmt, "{} B", self.0)
        } else if self.0 < 1_000_000 {
            write!(
                fmt,
                "{:.3} kB ({} Bytes)",
                self.0 as f64 / 1_000_f64,
                self.0
            )
        } else if self.0 < 1_000_000_000 {
            write!(
                fmt,
                "{:.3} MB ({} Bytes)",
                self.0 as f64 / 1_000_000_f64,
                self.0
            )
        } else {
            write!(
                fmt,
                "{:.3} GB ({} Bytes)",
                self.0 as f64 / 1_000_000_000_f64,
                self.0
            )
        }
    }
}
impl std::fmt::Debug for Bytes {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self)
    }
}
impl AddAssign for Bytes {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

struct ResourceInfoFramework {
    counter: Bytes,
    sum_allocations: Vec<Bytes>,
    peak_allocations: Vec<Bytes>,
    cpu_cycles: Vec<u64>,
}
impl ResourceInfoFramework {
    pub fn new() -> Self {
        Self {
            counter: Bytes(0),
            sum_allocations: Vec::new(),
            peak_allocations: Vec::new(),
            cpu_cycles: Vec::new(),
        }
    }
    pub fn add_measurement(&mut self, value: &ResourcesUsage) {
        self.counter += Bytes(value.heap_allocations_sum);
        self.sum_allocations.push(Bytes(value.heap_allocations_sum));
        self.peak_allocations.push(Bytes(value.heap_peak_memory));
        self.cpu_cycles.push(value.cpu_cycles);
    }

    pub fn print_report(&self) {
        let mut map: HashMap<Bytes, usize> = HashMap::new();
        for i in self.sum_allocations.iter() {
            map.entry(i.clone())
                .and_modify(|e| {
                    *e += 1;
                })
                .or_insert(1);
        }

        let mut sum_peak = Bytes(0);
        let mut map_peak: HashMap<Bytes, usize> = HashMap::new();
        for i in self.peak_allocations.iter() {
            map_peak
                .entry(i.clone())
                .and_modify(|e| {
                    *e += 1;
                })
                .or_insert(1);
            sum_peak += *i;
        }

        println!("Iterations: {}", self.sum_allocations.len());

        let avg_cpu_cycles: u64 =
            self.cpu_cycles.iter().sum::<u64>() / self.cpu_cycles.len() as u64;
        let max_cpu_cycles: u64 = *self.cpu_cycles.iter().max().unwrap_or(&0);
        let min_cpu_cycles: u64 = *self.cpu_cycles.iter().min().unwrap_or(&0);
        println!(
            "Cpu cycles stats for all iterations (average, max, min): ({}, {}, {})",
            avg_cpu_cycles, max_cpu_cycles, min_cpu_cycles
        );

        println!(
            "Sum of allocated heap memory in all iterations: {}",
            self.counter
        );
        let x = Bytes(self.counter.0 / self.sum_allocations.len());
        println!("Average allocation per iteration: {}", x);
        let x = Bytes(sum_peak.0 / self.peak_allocations.len());
        println!("Average peak allocation per iteration: {}", x);
        println!("Alocated memory chunks (size: count): {:#?}", map);
        println!("Peak allocations (size: count): {:#?}", map_peak);
    }
}

fn transfer_test(c: &mut Criterion) {
    // Set up environment.
    let mut fwk = ResourceInfoFramework::new();

    let mut scrypto_interpreter = ScryptoInterpreter {
        wasm_engine: DefaultWasmEngine::default(),
        wasm_instrumenter: WasmInstrumenter::default(),
        wasm_metering_config: WasmMeteringConfig::V0,
    };
    let mut substate_db = InMemorySubstateDatabase::standard();
    bootstrap(&mut substate_db, &scrypto_interpreter);

    // Create a key pair
    let private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
    let public_key = private_key.public_key();

    // Create two accounts
    let accounts = (0..2)
        .map(|_| {
            let manifest = ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 100.into())
                .new_account_advanced(rule!(require(NonFungibleGlobalId::from_public_key(
                    &public_key
                ))))
                .build();
            let account = execute_and_commit_transaction(
                &mut substate_db,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), 1, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
            )
            .expect_commit(true)
            .new_component_addresses()[0];

            let manifest = ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 100.into())
                .call_method(test_runner.faucet_component(), "free", manifest_args!())
                .call_method(
                    account,
                    "deposit_batch",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .build();
            execute_and_commit_transaction(
                &mut substate_db,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), 1, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
            )
            .expect_commit(true);

            account
        })
        .collect::<Vec<ComponentAddress>>();

    let account1 = accounts[0];
    let account2 = accounts[1];

    // Fill first account
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 100.into())
        .call_method(test_runner.faucet_component(), "free", manifest_args!())
        .call_method(
            account1,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    for nonce in 0..1000 {
        execute_and_commit_transaction(
            &mut substate_db,
            &mut scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
        )
        .expect_commit(true);
    }

    // Create a transfer manifest
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 100.into())
        .withdraw_from_account(account1, RADIX_TOKEN, dec!("0.000001"))
        .call_method(
            account2,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    // Loop
    let mut nonce = 3;
    c.bench_function("Transfer", |b| {
        b.iter(|| {
            let receipt = execute_and_commit_transaction(
                &mut substate_db,
                &mut scrypto_interpreter,
                &FeeReserveConfig::default(),
                &ExecutionConfig::default(),
                &TestTransaction::new(manifest.clone(), nonce, DEFAULT_COST_UNIT_LIMIT)
                    .get_executable(btreeset![NonFungibleGlobalId::from_public_key(&public_key)]),
            );

            fwk.add_measurement(&receipt.execution_trace.resources_usage);

            receipt.expect_commit_success();
            nonce += 1;
        })
    });

    fwk.print_report();
}

criterion_group!(resources_usage, transfer_test);
criterion_main!(resources_usage);
