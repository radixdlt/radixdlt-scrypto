use criterion::{criterion_group, criterion_main, Criterion};
use radix_common::prelude::*;
use radix_engine::vm::NoExtension;
use radix_engine_interface::prelude::*;
use radix_engine_tests::common::*;
#[cfg(not(feature = "rocksdb"))]
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
#[cfg(feature = "rocksdb")]
use radix_substate_store_impls::rocks_db_with_merkle_tree::RocksDBWithMerkleTreeSubstateStore;
use radix_transactions::prelude::*;
use scrypto_test::prelude::{LedgerSimulator, LedgerSimulatorBuilder};
#[cfg(feature = "rocksdb")]
use std::path::PathBuf;

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
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_database(RocksDBWithMerkleTreeSubstateStore::clear(PathBuf::from(
            "/tmp/radiswap",
        )))
        .build();
    #[cfg(not(feature = "rocksdb"))]
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Create account and publish package
    let (pk, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package(
        (
            include_workspace_asset_bytes!("radix-transaction-scenarios", "radiswap.wasm").to_vec(),
            manifest_decode(include_workspace_asset_bytes!(
                "radix-transaction-scenarios",
                "radiswap.rpd"
            ))
            .unwrap(),
        ),
        btreemap!(),
        OwnerRole::Updatable(rule!(require(signature(&pk)))),
    );

    // Create freely mintable resources
    let (btc_mint_auth, btc) = ledger.create_mintable_burnable_fungible_resource(account);
    let (eth_mint_auth, eth) = ledger.create_mintable_burnable_fungible_resource(account);

    // Create Radiswap
    let component_address: ComponentAddress = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_standard_test_fee(account)
                .call_function(
                    package_address,
                    "Radiswap",
                    "new",
                    manifest_args!(OwnerRole::None, btc, eth),
                )
                .try_deposit_entire_worktop_or_abort(account, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk)],
        )
        .expect_commit(true)
        .output(1);

    // Contribute to radiswap
    let btc_init_amount = Decimal::from(500_000);
    let eth_init_amount = Decimal::from(300_000);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_standard_test_fee(account)
                .create_proof_from_account_of_non_fungibles(
                    account,
                    btc_mint_auth,
                    [NonFungibleLocalId::integer(1)],
                )
                .mint_fungible(btc, btc_init_amount)
                .create_proof_from_account_of_non_fungibles(
                    account,
                    eth_mint_auth,
                    [NonFungibleLocalId::integer(1)],
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
                .try_deposit_entire_worktop_or_abort(account, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk)],
        )
        .expect_commit(true);

    let mut accounts = Vec::new();
    for i in 0..NUM_OF_PRE_FILLED_ACCOUNTS {
        if i % 100 == 0 {
            println!("{}/{}", i, NUM_OF_PRE_FILLED_ACCOUNTS);
        }
        let (pk2, _, account2) = ledger.new_allocated_account();
        ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_proof_from_account_of_non_fungibles(
                        account,
                        btc_mint_auth,
                        [NonFungibleLocalId::integer(1)],
                    )
                    .mint_fungible(btc, dec!("100"))
                    .create_proof_from_account_of_non_fungibles(
                        account,
                        eth_mint_auth,
                        [NonFungibleLocalId::integer(1)],
                    )
                    .mint_fungible(eth, dec!("100"))
                    .try_deposit_entire_worktop_or_abort(account2, None)
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
            &mut ledger,
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
                &mut ledger,
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
    ledger: &mut LedgerSimulator<NoExtension, DatabaseType>,
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
        .try_deposit_entire_worktop_or_abort(account.1, None)
        .build();

    ledger
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&account.0)],
        )
        .expect_commit_success();
}

criterion_group!(radiswap, bench_radiswap);
criterion_main!(radiswap);
