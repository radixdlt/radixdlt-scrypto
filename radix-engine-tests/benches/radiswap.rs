use criterion::{criterion_group, criterion_main, Criterion};
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::dec;
use scrypto_unit::TestRunner;
use transaction::{
    builder::{ManifestBuilder, TransactionBuilder},
    model::{NotarizedTransaction, TransactionHeader},
    validation::{
        HashStatus, NotarizedTransactionValidator, TestIntentHashManager, TransactionValidator,
        ValidationConfig,
    },
};

fn bench_radiswap(c: &mut Criterion) {
    let mut test_runner = TestRunner::builder().without_trace().build();

    // Scrypto developer
    let (pk1, _, _) = test_runner.new_allocated_account();
    // Radiswap operator
    let (pk2, _, account2) = test_runner.new_allocated_account();
    // Radiswap user
    let (pk3, sk3, account3) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.publish_package(
        include_bytes!("../../assets/radiswap.wasm").to_vec(),
        scrypto_decode(include_bytes!("../../assets/radiswap.abi")).unwrap(),
        btreemap!(
            "Radiswap".to_owned() => RoyaltyConfigBuilder::new()
                .add_rule("instantiate_pool", 5)
                .add_rule("add_liquidity", 1)
                .add_rule("remove_liquidity", 1)
                .add_rule("swap", 2)
                .default(0),
        ),
        btreemap!(),
        package_access_rules_from_owner_badge(&NonFungibleGlobalId::from_public_key(&pk1)),
    );

    // Instantiate radiswap
    let btc = test_runner.create_fungible_resource(1_000_000.into(), 18, account2);
    let eth = test_runner.create_fungible_resource(1_000_000.into(), 18, account2);
    let btc_init_amount = Decimal::from(500_000);
    let eth_init_amount = Decimal::from(300_000);
    let fee_amount = dec!("0.01");
    let (component_address, _) = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 10u32.into())
                .withdraw_from_account(account2, btc, btc_init_amount)
                .withdraw_from_account(account2, eth, eth_init_amount)
                .take_from_worktop(btc, |builder, bucket1| {
                    builder.take_from_worktop(eth, |builder, bucket2| {
                        builder.call_function(
                            package_address,
                            "Radiswap",
                            "instantiate_pool",
                            args!(
                                bucket1,
                                bucket2,
                                dec!("1000"),
                                "LP__ETH",
                                "LP token for /ETH swap",
                                "https://www.radiswap.com",
                                fee_amount
                            ),
                        )
                    })
                })
                .call_method(
                    account2,
                    "deposit_batch",
                    args!(ManifestExpression::EntireWorktop),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .output::<(ComponentAddress, Own)>(5);

    // Transfer `10,000 BTC` from `account2` to `account3`
    let btc_amount = Decimal::from(10_000);
    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 10u32.into())
                .withdraw_from_account(account2, btc, btc_amount)
                .call_method(
                    account3,
                    "deposit_batch",
                    args!(ManifestExpression::EntireWorktop),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit_success();

    // Swap 1 BTC into ETH
    let manifest = ManifestBuilder::new()
        .lock_fee(account3, 10u32.into())
        .withdraw_from_account(account3, btc, dec!(1))
        .take_from_worktop(btc, |builder, bucket| {
            builder.call_method(component_address, "swap", args!(bucket))
        })
        .call_method(
            account3,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let mut intent_hash_mgr = TestIntentHashManager::new();
    let mut transactions: Vec<Vec<u8>> = (1_000..20_000)
        .map(|i| {
            TransactionBuilder::new()
                .header(TransactionHeader {
                    version: 1,
                    network_id: NetworkDefinition::simulator().id,
                    start_epoch_inclusive: 0,
                    end_epoch_exclusive: 100,
                    nonce: i,
                    notary_public_key: pk3.clone().into(),
                    notary_as_signatory: true,
                    cost_unit_limit: 100_000_000,
                    tip_percentage: 5,
                })
                .manifest(manifest.clone())
                .notarize(&sk3)
                .build()
                .to_bytes()
                .unwrap()
        })
        .collect();

    // Loop
    c.bench_function("Radiswap", |b| {
        b.iter(|| {
            let payload = transactions.pop().unwrap();

            // Decode payload
            let transaction: NotarizedTransaction = scrypto_decode(&payload).unwrap();

            // Validate
            let executable = NotarizedTransactionValidator::new(ValidationConfig::default(
                NetworkDefinition::simulator().id,
            ))
            .validate(&transaction, payload.len(), &intent_hash_mgr)
            .unwrap();

            // Execute & commit
            test_runner
                .execute_transaction(executable)
                .expect_commit_success();

            // Update intent hash manager
            intent_hash_mgr.insert(
                transaction.signed_intent.intent.hash().unwrap(),
                HashStatus::Committed,
            );
        })
    });
}

criterion_group!(radiswap, bench_radiswap);
criterion_main!(radiswap);
