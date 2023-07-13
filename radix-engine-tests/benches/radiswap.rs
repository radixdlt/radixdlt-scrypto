use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::types::*;
use radix_engine::vm::NoExtension;
#[cfg(not(feature = "rocksdb"))]
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
#[cfg(feature = "rocksdb")]
use radix_engine_stores::rocks_db_with_merkle_tree::RocksDBWithMerkleTreeSubstateStore;
use scrypto_unit::{TestRunner, TestRunnerBuilder};
#[cfg(feature = "rocksdb")]
use std::path::PathBuf;
use transaction::prelude::*;

/// Number of prefilled accounts in the substate store
#[cfg(feature = "rocksdb")]
const NUM_OF_PRE_FILLED_ACCOUNTS: usize = 100_000;
#[cfg(not(feature = "rocksdb"))]
const NUM_OF_PRE_FILLED_ACCOUNTS: usize = 200;

/// To profile with flamegraph, run
/// ```bash
/// sudo cargo flamegraph --bench radiswap --features flamegraph
/// ```
///
/// To benchmark, run
/// ```bash
/// cargo bench --bench radiswap
/// ```
///
/// To benchmark with rocksdb, run
/// ```bash
/// cargo bench --bench radiswap --features rocksdb
/// ```
fn bench_radiswap(c: &mut Criterion) {
    #[cfg(feature = "rocksdb")]
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_database(RocksDBWithMerkleTreeSubstateStore::clear(PathBuf::from(
            "/tmp/radiswap",
        )))
        .without_trace()
        .build();
    #[cfg(not(feature = "rocksdb"))]
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();

    // Create account and publish package
    let (pk, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.publish_package(
        include_bytes!("../../assets/radiswap.wasm").to_vec(),
        manifest_decode(include_bytes!("../../assets/radiswap.rpd")).unwrap(),
        btreemap!(),
        OwnerRole::Updatable(rule!(require(NonFungibleGlobalId::from_public_key(&pk)))),
    );

    // Create freely mintable resources
    let (btc_mint_auth, btc) = test_runner.create_mintable_burnable_fungible_resource(account);
    let (eth_mint_auth, eth) = test_runner.create_mintable_burnable_fungible_resource(account);

    // Create Radiswap
    let component_address: ComponentAddress = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_standard_test_fee(account)
                .call_function(
                    package_address,
                    "Radiswap",
                    "new",
                    manifest_args!(OwnerRole::None, btc, eth),
                )
                .try_deposit_batch_or_abort(account)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk)],
        )
        .expect_commit(true)
        .output(1);

    // Contribute to radiswap
    let btc_init_amount = Decimal::from(500_000);
    let eth_init_amount = Decimal::from(300_000);
    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_standard_test_fee(account)
                .create_proof_from_account_of_non_fungibles(
                    account,
                    btc_mint_auth,
                    &btreeset!(NonFungibleLocalId::integer(1)),
                )
                .mint_fungible(btc, btc_init_amount)
                .create_proof_from_account_of_non_fungibles(
                    account,
                    eth_mint_auth,
                    &btreeset!(NonFungibleLocalId::integer(1)),
                )
                .mint_fungible(eth, eth_init_amount)
                .take_all_from_worktop(btc, "liquidity_part_1")
                .take_all_from_worktop(eth, "liquidity_part_2")
                .with_name_lookup(|builder, lookup| {
                    let bucket1 = lookup.bucket("liquidity_part_1");
                    let bucket2 = lookup.bucket("liquidity_part_2");
                    builder.call_method(
                        component_address,
                        "add_liquidity",
                        manifest_args!(bucket1, bucket2),
                    )
                })
                .try_deposit_batch_or_abort(account)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk)],
        )
        .expect_commit(true);

    let mut accounts = Vec::new();
    for i in 0..NUM_OF_PRE_FILLED_ACCOUNTS {
        if i % 100 == 0 {
            println!("{}/{}", i, NUM_OF_PRE_FILLED_ACCOUNTS);
        }
        let (pk2, _, account2) = test_runner.new_allocated_account();
        test_runner
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_standard_test_fee(account)
                    .create_proof_from_account_of_non_fungibles(
                        account,
                        btc_mint_auth,
                        &btreeset!(NonFungibleLocalId::integer(1)),
                    )
                    .mint_fungible(btc, dec!("100"))
                    .create_proof_from_account_of_non_fungibles(
                        account,
                        eth_mint_auth,
                        &btreeset!(NonFungibleLocalId::integer(1)),
                    )
                    .mint_fungible(eth, dec!("100"))
                    .try_deposit_batch_or_abort(account2)
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(&pk)],
            )
            .expect_commit(true);
        accounts.push((pk2, account2));
    }

    let mut index = 0;
    #[cfg(feature = "flamegraph")]
    for _ in 0..1000 {
        do_swap(
            &mut test_runner,
            &accounts[index % accounts.len()],
            btc,
            component_address,
        );
        index += 1;
    }
    #[cfg(not(feature = "flamegraph"))]
    c.bench_function("transaction::radiswap", |b| {
        b.iter(|| {
            do_swap(
                &mut test_runner,
                &accounts[index % accounts.len()],
                btc,
                component_address,
            );
            index += 1;
        })
    });
}

#[cfg(feature = "rocksdb")]
type DatabaseType = RocksDBWithMerkleTreeSubstateStore;
#[cfg(not(feature = "rocksdb"))]
type DatabaseType = InMemorySubstateDatabase;

fn do_swap(
    test_runner: &mut TestRunner<NoExtension, DatabaseType>,
    account: &(Secp256k1PublicKey, ComponentAddress),
    btc: ResourceAddress,
    component_address: ComponentAddress,
) {
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account.1)
        .withdraw_from_account(account.1, btc, dec!("1"))
        .take_all_from_worktop(btc, "to_trade")
        .with_name_lookup(|builder, lookup| {
            let to_trade_bucket = lookup.bucket("to_trade");
            builder.call_method(component_address, "swap", manifest_args!(to_trade_bucket))
        })
        .try_deposit_batch_or_abort(account.1)
        .build();

    test_runner
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&account.0)],
        )
        .expect_commit_success();
}

criterion_group!(radiswap, bench_radiswap);
criterion_main!(radiswap);
