use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::kernel::ScryptoInterpreter;
use radix_engine::ledger::*;
use radix_engine::transaction::execute_and_commit_transaction;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use radix_engine::wasm::WasmInstrumenter;
use radix_engine::wasm::{DefaultWasmEngine, WasmMeteringConfig};
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::dec;
use radix_engine_interface::rule;
use scrypto::include_code;
use scrypto_unit::{TestRunner, TestRunnerBuilder};
use transaction::builder::ManifestBuilder;
use transaction::model::TestTransaction;
use transaction::signing::EcdsaSecp256k1PrivateKey;

fn bench_radiswap(c: &mut Criterion) {}
criterion_group!(radiswap, bench_radiswap);
criterion_main!(radiswap);
