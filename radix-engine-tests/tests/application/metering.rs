use radix_common::prelude::*;
use radix_engine::errors::RejectionReason;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::updates::*;
use radix_engine_interface::blueprints::access_controller::ACCESS_CONTROLLER_CREATE_PROOF_IDENT;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_tests::common::*;
use radix_transactions::validation::TransactionValidationConfig;
use scrypto::object_modules::ModuleConfig;
use scrypto::prelude::metadata;
use scrypto::prelude::metadata_init;
use scrypto_test::prelude::*;

#[cfg(not(feature = "alloc"))]
use scrypto_test::utils::CostingTaskMode;

// For WASM-specific metering tests, see `wasm_metering.rs`.

#[cfg(not(feature = "alloc"))]
#[test]
#[ignore = "Run this test to update expected costs"]
fn update_expected_costs() {
    run_all(CostingTaskMode::OutputCosting);
}

#[cfg(not(feature = "alloc"))]
#[test]
fn run_cost_tests() {
    run_all(CostingTaskMode::AssertCosting);
}

#[cfg(not(feature = "alloc"))]
fn run_all(mode: CostingTaskMode) {
    use radix_engine_tests::path_local_metering_receipts;

    for protocol_version in ProtocolVersion::all_from(ProtocolVersion::GENESIS) {
        let base_path = path_local_metering_receipts!();
        let execute = move |run: &dyn Fn(DefaultLedgerSimulator) -> TransactionReceipt,
                            file: &'static str| {
            let ledger = LedgerSimulatorBuilder::new()
                .with_cost_breakdown()
                .with_custom_protocol(|builder| builder.from_bootstrap_to(protocol_version))
                .build();
            let receipt = run(ledger);
            let relative_file_path = format!("{}/{}", protocol_version.logical_name(), file);
            mode.run(
                &base_path,
                &relative_file_path,
                &receipt.fee_summary,
                &receipt.fee_details.unwrap(),
            )
        };

        execute(&run_basic_transfer, "cost_transfer.csv");
        execute(
            &run_basic_transfer_to_preallocated_account,
            "cost_transfer_to_preallocated_account.csv",
        );
        execute(&run_radiswap, "cost_radiswap.csv");
        execute(&run_flash_loan, "cost_flash_loan.csv");
        execute(&run_publish_large_package, "cost_publish_large_package.csv");
        execute(
            &run_mint_large_size_nfts_from_manifest,
            "cost_mint_large_size_nfts_from_manifest.csv",
        );
        execute(
            &run_mint_small_size_nfts_from_manifest,
            "cost_mint_small_size_nfts_from_manifest.csv",
        );

        // Run cost tests for crypto_utils test from Cuttlefish onward
        if protocol_version >= ProtocolVersion::CuttlefishPart1 {
            execute(&run_crypto_utils_tests, "cost_crypto_utils.csv");
        }
    }
}

#[cfg(feature = "std")]
fn execute_with_time_logging(
    ledger: &mut DefaultLedgerSimulator,
    manifest: TransactionManifestV1,
    proofs: Vec<NonFungibleGlobalId>,
) -> (TransactionReceipt, u32) {
    let start = std::time::Instant::now();
    let receipt = ledger.execute_manifest(manifest, proofs);
    let duration = start.elapsed();
    println!(
        "Time elapsed is: {:?} - NOTE: this is a very bad measure. Use benchmarks instead.",
        duration
    );
    (receipt, duration.as_millis().try_into().unwrap())
}

#[cfg(feature = "alloc")]
fn execute_with_time_logging(
    ledger: &mut DefaultLedgerSimulator,
    manifest: TransactionManifestV1,
    proofs: Vec<NonFungibleGlobalId>,
) -> (TransactionReceipt, u32) {
    let receipt = ledger.execute_manifest(manifest, proofs);
    (receipt, 0)
}

fn run_basic_transfer(mut ledger: DefaultLedgerSimulator) -> TransactionReceipt {
    // Arrange
    let (public_key1, _, account1) = ledger.new_allocated_account();
    let (_, _, account2) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account1)
        .withdraw_from_account(account1, XRD, 100)
        .try_deposit_entire_worktop_or_abort(account2, None)
        .build();

    let (receipt, _) = execute_with_time_logging(
        &mut ledger,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key1)],
    );
    receipt.expect_commit(true);
    receipt
}

fn run_basic_transfer_to_preallocated_account(
    mut ledger: DefaultLedgerSimulator,
) -> TransactionReceipt {
    // Arrange
    let (public_key1, _, account1) = ledger.new_allocated_account();
    let account2 = ComponentAddress::preallocated_account_from_public_key(&PublicKey::Secp256k1(
        Secp256k1PublicKey([123u8; 33]),
    ));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account1)
        .withdraw_from_account(account1, XRD, 100)
        .try_deposit_entire_worktop_or_abort(account2, None)
        .build();

    let (receipt, _) = execute_with_time_logging(
        &mut ledger,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key1)],
    );
    receipt.expect_commit(true);
    receipt
}

fn run_radiswap(mut ledger: DefaultLedgerSimulator) -> TransactionReceipt {
    // Scrypto developer
    let (pk1, _, _) = ledger.new_allocated_account();
    // Radiswap operator
    let (pk2, _, account2) = ledger.new_allocated_account();
    // Radiswap user
    let (pk3, _, account3) = ledger.new_allocated_account();

    // Publish package
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
        OwnerRole::Fixed(rule!(require(signature(&pk1)))),
    );

    // Instantiate Radiswap
    let btc = ledger.create_fungible_resource(1_000_000.into(), 18, account2);
    let eth = ledger.create_fungible_resource(1_000_000.into(), 18, account2);
    let component_address: ComponentAddress = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_standard_test_fee(account2)
                .call_function(
                    package_address,
                    "Radiswap",
                    "new",
                    manifest_args!(OwnerRole::None, btc, eth),
                )
                .try_deposit_entire_worktop_or_abort(account2, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit(true)
        .output(1);

    // Contributing an initial amount to radiswap
    let btc_init_amount = Decimal::from(500_000);
    let eth_init_amount = Decimal::from(300_000);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_standard_test_fee(account2)
                .withdraw_from_account(account2, btc, btc_init_amount)
                .withdraw_from_account(account2, eth, eth_init_amount)
                .take_all_from_worktop(btc, "btc")
                .take_all_from_worktop(eth, "eth")
                .with_name_lookup(|builder, lookup| {
                    builder.call_method(
                        component_address,
                        "add_liquidity",
                        manifest_args!(lookup.bucket("btc"), lookup.bucket("eth")),
                    )
                })
                .try_deposit_entire_worktop_or_abort(account2, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit(true);

    // Transfer `10,000 BTC` from `account2` to `account3`
    let btc_amount = Decimal::from(10_000);
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 500)
                .withdraw_from_account(account2, btc, btc_amount)
                .try_deposit_entire_worktop_or_abort(account3, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit_success();
    assert_eq!(ledger.get_component_balance(account3, btc), btc_amount);

    // Swap 2,000 BTC into ETH
    let btc_to_swap = Decimal::from(2000);
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account3, 500)
            .withdraw_from_account(account3, btc, btc_to_swap)
            .take_all_from_worktop(btc, "to_trade")
            .with_name_lookup(|builder, lookup| {
                let bucket = lookup.bucket("to_trade");
                builder.call_method(component_address, "swap", manifest_args!(bucket))
            })
            .try_deposit_entire_worktop_or_abort(account3, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk3)],
    );
    let remaining_btc = ledger.get_component_balance(account3, btc);
    let eth_received = ledger.get_component_balance(account3, eth);
    assert_eq!(remaining_btc, btc_amount.checked_sub(btc_to_swap).unwrap());
    assert_eq!(eth_received, dec!("1195.219123505976095617"));
    receipt.expect_commit(true);

    receipt
}

fn run_flash_loan(mut ledger: DefaultLedgerSimulator) -> TransactionReceipt {
    // Scrypto developer
    let (pk1, _, _) = ledger.new_allocated_account();
    // Flash loan operator
    let (pk2, _, account2) = ledger.new_allocated_account();
    // Flash loan user
    let (pk3, _, account3) = ledger.new_allocated_account();

    // Publish package
    let package_address = ledger.publish_package(
        (
            include_workspace_asset_bytes!("radix-transaction-scenarios", "flash_loan.wasm")
                .to_vec(),
            manifest_decode(include_workspace_asset_bytes!(
                "radix-transaction-scenarios",
                "flash_loan.rpd"
            ))
            .unwrap(),
        ),
        btreemap!(),
        OwnerRole::Fixed(rule!(require(signature(&pk1)))),
    );

    // Instantiate flash_loan
    let xrd_init_amount = Decimal::from(100);
    let (component_address, promise_token_address) = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_standard_test_fee(account2)
                .withdraw_from_account(account2, XRD, xrd_init_amount)
                .take_all_from_worktop(XRD, "bucket")
                .with_name_lookup(|builder, lookup| {
                    builder.call_function(
                        package_address,
                        "BasicFlashLoan",
                        "instantiate_default",
                        manifest_args!(lookup.bucket("bucket")),
                    )
                })
                .try_deposit_entire_worktop_or_abort(account2, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit(true)
        .output::<(ComponentAddress, ResourceAddress)>(3);

    // Take loan
    let loan_amount = Decimal::from(50);
    let repay_amount = loan_amount.checked_mul(dec!("1.001")).unwrap();
    let old_balance = ledger.get_component_balance(account3, XRD);
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account3, 500)
            .call_method(component_address, "take_loan", manifest_args!(loan_amount))
            .withdraw_from_account(account3, XRD, dec!(10))
            .take_from_worktop(XRD, repay_amount, "repayment")
            .take_all_from_worktop(promise_token_address, "promise")
            .with_name_lookup(|builder, lookup| {
                builder.call_method(
                    component_address,
                    "repay_loan",
                    manifest_args!(lookup.bucket("repayment"), lookup.bucket("promise")),
                )
            })
            .try_deposit_entire_worktop_or_abort(account3, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk3)],
    );
    receipt.expect_commit(true);
    let new_balance = ledger.get_component_balance(account3, XRD);
    assert!(ledger
        .get_component_balance(account3, promise_token_address)
        .is_zero());
    assert_eq!(
        old_balance.checked_sub(new_balance).unwrap(),
        receipt
            .fee_summary
            .total_cost()
            .checked_add(repay_amount)
            .unwrap()
            .checked_sub(loan_amount)
            .unwrap()
    );
    receipt
}

fn run_publish_large_package(mut ledger: DefaultLedgerSimulator) -> TransactionReceipt {
    // Act
    let code = wat2wasm(&format!(
        r#"
                (module
                    (data (i32.const 0) "{}")
                    (memory $0 64)
                    (export "memory" (memory $0))
                )
            "#,
        "i".repeat(1024 * 1024 - 1024)
    ));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            PackageDefinition::default(),
            BTreeMap::new(),
            OwnerRole::None,
        )
        .build();

    let (receipt, _) = execute_with_time_logging(&mut ledger, manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    receipt
}

fn run_mint_small_size_nfts_from_manifest(ledger: DefaultLedgerSimulator) -> TransactionReceipt {
    run_mint_nfts_from_manifest(
        ledger,
        TestNonFungibleData {
            metadata: btreemap!(),
        },
    )
}

fn run_mint_large_size_nfts_from_manifest(ledger: DefaultLedgerSimulator) -> TransactionReceipt {
    const N: usize = 50;

    run_mint_nfts_from_manifest(
        ledger,
        TestNonFungibleData {
            metadata: btreemap!(
                "Name".to_string() => "Type".repeat(N),
                "Abilities".to_string() => "Lightning Rod".repeat(N),
                "Egg Groups".to_string() => "Field and Fairy or No Eggs Discovered".repeat(N),
                "Hatch time".to_string() => "10 cycles".repeat(N),
                "Height".to_string() => "0.4 m".repeat(N),
                "Weight".to_string() => "6.0 kg".repeat(N),
                "Base experience yield".to_string() => "82".repeat(N),
                "Leveling rate".to_string() => "Medium Fast".repeat(N),
            ),
        },
    )
}

fn run_mint_nfts_from_manifest(
    mut ledger: DefaultLedgerSimulator,
    nft_data: TestNonFungibleData,
) -> TransactionReceipt {
    // Arrange
    let (_, _, account) = ledger.new_allocated_account();

    // Act
    let mut low = 16;
    let mut high = 16 * 1024;
    let mut last_success_receipt = None;
    let mut last_fail_receipt = None;
    while low <= high {
        let mid = low + (high - low) / 2;
        let mut entries = BTreeMap::new();
        for i in 0..mid {
            entries.insert(NonFungibleLocalId::integer(i), nft_data.clone());
        }
        let manifest = ManifestBuilder::new()
            .lock_fee(FAUCET, 1_000_000)
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                false,
                NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata! {},
                Some(entries),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        let transaction = create_notarized_transaction(
            TransactionParams {
                start_epoch_inclusive: Epoch::of(0),
                end_epoch_exclusive: Epoch::of(100),
            },
            manifest.clone(),
        );
        let raw_transaction = transaction.to_raw().unwrap();
        let max_size = TransactionValidationConfig::latest()
            .preparation_settings
            .max_user_payload_length;
        if raw_transaction.as_slice().len() > max_size {
            high = mid - 1;
        } else {
            let receipt = ledger.execute_manifest(manifest, vec![]);
            if receipt.is_commit_success() {
                last_success_receipt = Some((mid, receipt, raw_transaction));
                low = mid + 1;
            } else {
                last_fail_receipt = Some((mid, receipt, raw_transaction));
                high = mid - 1;
            }
        }
    }

    // Assert
    let (n, receipt, raw_transaction) = last_success_receipt.unwrap_or_else(|| {
        // Print an error message from the failing commit
        last_fail_receipt.unwrap().1.expect_commit_success();
        unreachable!()
    });
    println!("Transaction payload size: {} bytes", raw_transaction.len());
    println!(
        "Average NFT size: {} bytes",
        scrypto_encode(&nft_data).unwrap().len()
    );
    println!("Managed to mint {} NFTs", n);
    receipt
}

fn get_aggregate_verify_test_data(
    cnt: u32,
    msg_size: usize,
) -> (
    Vec<Bls12381G1PrivateKey>,
    Vec<Bls12381G1PublicKey>,
    Vec<Vec<u8>>,
    Vec<Bls12381G2Signature>,
) {
    let sks: Vec<Bls12381G1PrivateKey> = (1..(cnt + 1))
        .map(|i| Bls12381G1PrivateKey::from_u64(i.into()).unwrap())
        .collect();

    let msgs: Vec<Vec<u8>> = (1..(cnt + 1))
        .map(|i| {
            let u: u8 = (i % u8::MAX as u32) as u8;
            vec![u; msg_size]
        })
        .collect();
    let sigs: Vec<Bls12381G2Signature> = sks
        .iter()
        .zip(msgs.clone())
        .map(|(sk, msg)| sk.sign_v1(&msg))
        .collect();

    let pks: Vec<Bls12381G1PublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    (sks, pks, msgs, sigs)
}

fn run_crypto_utils_tests(mut ledger: DefaultLedgerSimulator) -> TransactionReceipt {
    let package_address = ledger.publish_package_simple((
        include_workspace_asset_bytes!("radix-transaction-scenarios", "crypto_scrypto_v2.wasm")
            .to_vec(),
        manifest_decode(include_workspace_asset_bytes!(
            "radix-transaction-scenarios",
            "crypto_scrypto_v2.rpd"
        ))
        .unwrap(),
    ));

    let msg = "Test";
    let msg_hash = hash(msg);
    let bls_pk = Bls12381G1PublicKey::from_str("93b1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2").unwrap();
    let msg_hash_bls_signature = Bls12381G2Signature::from_str("8b84ff5a1d4f8095ab8a80518ac99230ed24a7d1ec90c4105f9c719aa7137ed5d7ce1454d4a953f5f55f3959ab416f3014f4cd2c361e4d32c6b4704a70b0e2e652a908f501acb54ec4e79540be010e3fdc1fbf8e7af61625705e185a71c884f1").unwrap();

    let (sks, pks, msgs, sigs) = get_aggregate_verify_test_data(10, 10);

    // Aggregate the signature
    let agg_sig_multiple_msgs = Bls12381G2Signature::aggregate(&sigs, true).unwrap();
    let pks_msgs: Vec<(Bls12381G1PublicKey, Vec<u8>)> =
        pks.iter().zip(msgs).map(|(pk, sk)| (*pk, sk)).collect();

    let sigs: Vec<Bls12381G2Signature> = sks
        .iter()
        .map(|sk| sk.sign_v1(msg_hash.as_bytes()))
        .collect();

    // Aggregate the signature
    let agg_sig_single_msg = Bls12381G2Signature::aggregate(&sigs, true).unwrap();

    let ed25519_pk = Ed25519PublicKey::from_str(
        "4cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29",
    )
    .unwrap();
    let ed25519_msg_hash_signature = Ed25519Signature::from_str("cf0ca64435609b85ab170da339d415bbac87d678dfd505969be20adc6b5971f4ee4b4620c602bcbc34fd347596546675099d696265f4a42a16df343da1af980e").unwrap();

    let secp256k1_pk = Secp256k1PublicKey::from_str(
        "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
    )
    .unwrap();
    let secp256k1_msg_hash_signature = Secp256k1Signature::from_str("00eb8dcd5bb841430dd0a6f45565a1b8bdb4a204eb868832cd006f963a89a662813ab844a542fcdbfda4086a83fbbde516214113051b9c8e42a206c98d564d7122").unwrap();

    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(FAUCET, 1_000)
            .call_function(
                package_address,
                "CryptoScrypto",
                "keccak256_hash",
                manifest_args!(&msg_hash),
            )
            .call_function(
                package_address,
                "CryptoScrypto",
                "blake2b_256_hash",
                manifest_args!(&msg_hash),
            )
            .call_function(
                package_address,
                "CryptoScrypto",
                "bls12381_v1_verify",
                manifest_args!(msg_hash.as_bytes(), &bls_pk, &msg_hash_bls_signature),
            )
            .call_function(
                package_address,
                "CryptoScrypto",
                "bls12381_g2_signature_aggregate",
                manifest_args!(&sigs),
            )
            .call_function(
                package_address,
                "CryptoScrypto",
                "bls12381_v1_aggregate_verify",
                manifest_args!(&pks_msgs, &agg_sig_multiple_msgs),
            )
            .call_function(
                package_address,
                "CryptoScrypto",
                "bls12381_v1_fast_aggregate_verify",
                manifest_args!(msg_hash.as_bytes(), &pks, &agg_sig_single_msg),
            )
            .call_function(
                package_address,
                "CryptoScrypto",
                "ed25519_verify",
                manifest_args!(
                    msg_hash.as_bytes(),
                    &ed25519_pk,
                    &ed25519_msg_hash_signature
                ),
            )
            .call_function(
                package_address,
                "CryptoScrypto",
                "secp256k1_ecdsa_verify",
                manifest_args!(&msg_hash, &secp256k1_pk, &secp256k1_msg_hash_signature),
            )
            .call_function(
                package_address,
                "CryptoScrypto",
                "secp256k1_ecdsa_verify_and_key_recover",
                manifest_args!(&msg_hash, &secp256k1_msg_hash_signature),
            )
            .build(),
        vec![],
    );

    // Make sure above operations return positive results
    assert!(verify_bls12381_v1(
        msg_hash.as_bytes(),
        &bls_pk,
        &msg_hash_bls_signature
    ));
    assert!(aggregate_verify_bls12381_v1(
        &pks_msgs,
        &agg_sig_multiple_msgs
    ));
    assert!(fast_aggregate_verify_bls12381_v1(
        msg_hash.as_bytes(),
        &pks,
        &agg_sig_single_msg
    ));
    assert!(verify_ed25519(
        msg_hash.as_bytes(),
        &ed25519_pk,
        &ed25519_msg_hash_signature
    ));
    assert!(verify_secp256k1(
        &msg_hash,
        &secp256k1_pk,
        &secp256k1_msg_hash_signature
    ));
    assert_eq!(
        verify_and_recover_secp256k1(&msg_hash, &secp256k1_msg_hash_signature).unwrap(),
        secp256k1_pk
    );
    receipt.expect_commit_success();

    receipt
}

#[test]
fn can_run_large_manifest() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .withdraw_from_account(account, XRD, 100);
    for _ in 0..40 {
        let bucket = builder.generate_bucket_name("bucket");
        builder = builder
            .take_from_worktop(XRD, 1, &bucket)
            .return_to_worktop(bucket);
    }
    let manifest = builder
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();

    let (receipt, _) = execute_with_time_logging(
        &mut ledger,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit(true);
}

#[test]
fn should_be_able_to_generate_5_proofs_and_then_lock_fee() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address = ledger.create_fungible_resource(100.into(), 0, account);

    // Act
    let mut builder = ManifestBuilder::new();
    for _ in 0..5 {
        builder = builder.create_proof_from_account_of_amount(account, resource_address, 1);
    }
    let manifest = builder.lock_standard_test_fee(account).build();

    let (receipt, _) = execute_with_time_logging(
        &mut ledger,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit(true);
}

fn setup_ledger_with_fee_blueprint_component() -> (DefaultLedgerSimulator, ComponentAddress) {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Publish package and instantiate component
    let package_address = ledger.publish_package_simple(PackageLoader::get("fee"));
    let receipt1 = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .withdraw_from_account(account, XRD, 10)
            .take_all_from_worktop(XRD, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "Fee",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let commit_result = receipt1.expect_commit(true);
    let component_address = commit_result.new_component_addresses()[0];

    (ledger, component_address)
}

#[test]
fn spin_loop_should_end_in_reasonable_amount_of_time() {
    let (mut ledger, component_address) = setup_ledger_with_fee_blueprint_component();

    let manifest = ManifestBuilder::new()
        // First, lock the fee so that the loan will be repaid
        .lock_fee_from_faucet()
        // Now spin-loop to wait for the fee loan to burn through
        .call_method(component_address, "spin_loop", manifest_args!())
        .build();

    let (receipt, _) = execute_with_time_logging(&mut ledger, manifest, vec![]);

    // No assertion here - this is just a sanity-test
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );
    receipt.expect_commit_failure();
}

#[derive(Clone, ScryptoSbor, ManifestSbor)]
struct TestNonFungibleData {
    metadata: BTreeMap<String, String>,
}

impl NonFungibleData for TestNonFungibleData {
    const MUTABLE_FIELDS: &'static [&'static str] = &["metadata"];
}

/// This test verified that we can publish a large package of size as close as possible to current
/// limit: 1,048,576 bytes minus SBOR overhead.
///
/// If it fails, update `radix-engine-tests/blueprints/large_package/` by adding or removing blueprints
/// to make sure the size is close to 1MB. This is often needed when the WASM interface or compiler
/// changes.
///
/// List of blueprints and its size can be displayed using command
/// `ls -lSk ./radix-engine-tests/tests/blueprints/target/wasm32-unknown-unknown/release/*.wasm`
///
#[test]
fn publish_package_1mib() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let code = include_workspace_asset_bytes!("radix-engine-tests", "large_package.wasm").to_vec();
    let definition = manifest_decode(include_workspace_asset_bytes!(
        "radix-engine-tests",
        "large_package.rpd"
    ))
    .unwrap();
    println!("Code size: {}", code.len());
    assert!(code.len() <= 1000 * 1024);
    assert!(code.len() >= 900 * 1024);

    // internally validates if publish succeeded
    ledger.publish_package((code, definition), BTreeMap::new(), OwnerRole::None);
}

/// Based on product requirements, system loan should be just enough to cover:
/// 1. Notary + 3 signatures in TX
/// 2. Ask AccessController to produce a badge
/// 3. Call withdraw_and_lock_fee on Account
#[test]
fn system_loan_should_cover_intended_use_case() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let network = NetworkDefinition::simulator();
    let (_pk1, sk1, _pk2, sk2, _pk3, sk3, _pk4, sk4, account, access_controller) =
        ledger.new_ed25519_preallocated_account_with_access_controller(3);

    let manifest1 = ManifestBuilder::new()
        .call_method(
            access_controller,
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT,
            manifest_args!(),
        )
        .lock_fee_and_withdraw(account, dec!(10), XRD, dec!(10))
        .then(|mut builder| {
            // Artificial workload
            for _ in 0..10 {
                builder = builder
                    .withdraw_from_account(account, XRD, 1)
                    .try_deposit_entire_worktop_or_abort(account, None);
            }
            builder
        })
        .build();
    let tx1 = create_notarized_transaction_advanced(
        &mut ledger,
        &network,
        manifest1,
        vec![&sk1, &sk2, &sk3], // sign
        &sk4,                   // notarize
        false,
    );
    let receipt = ledger.execute_transaction(
        tx1,
        ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator())
            .with_cost_breakdown(true),
    );
    receipt.expect_commit_success();
    println!("\n{:?}", receipt);
}

/// Verifying that the system loan covers an extended use case:
/// 1. Self-notary signature on the TX
/// 2. Create proof from account
/// 3. Call lock_fee on component, passing in a badge and doing a badge check
///
/// This is not strictly required, but if this test breaks in future, we'll want to discuss this as a team.
#[test]
fn system_loan_should_cover_very_minimal_lock_fee_in_scrypto_component() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().with_cost_breakdown().build();
    let network = NetworkDefinition::simulator();
    let (_, private_key, account_address) = ledger.new_account(true);

    // NOTE: We can't use `PackageLoader::get` here because it builds with `CompileProfile::FastWithTraceLogs`
    // This test's passing is very borderline, and compiling without optimizations means the test doesn't pass.
    // Therefore we have to make sure we do build with optimizations:
    let compiled = Compile::compile(path_local_blueprint!("fee"), CompileProfile::Standard);
    let package_address = ledger.publish_package_simple(compiled);
    let deploy_receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .take_all_from_worktop(XRD, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "Fee",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build(),
        [],
    );
    let component_address = deploy_receipt
        .expect_commit_success()
        .state_update_summary
        .new_components[0];
    let doge_resource = deploy_receipt
        .expect_commit_success()
        .state_update_summary
        .new_resources[0];

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(component_address, "get_doge", ())
                .try_deposit_entire_worktop_or_abort(account_address, None)
                .build(),
            [],
        )
        .expect_commit_success();

    // Act
    let main_manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account_address, doge_resource, 1)
        .pop_from_auth_zone("proof")
        .call_method_with_name_lookup(component_address, "lock_fee_with_badge_check", |lookup| {
            (lookup.proof("proof"),)
        })
        .build();

    let main_transaction = create_notarized_transaction_advanced(
        &mut ledger,
        &network,
        main_manifest,
        vec![],       // no additional signers
        &private_key, // notarize with signer key
        true,
    );

    let receipt = ledger.execute_notarized_transaction(main_transaction);

    // Assert and print
    receipt.expect_commit_success();
    println!(
        "\n{}",
        format_cost_breakdown(&receipt.fee_summary, receipt.fee_details.as_ref().unwrap())
    );

    let encoder = AddressBech32Encoder::for_simulator();
    let display_context = TransactionReceiptDisplayContextBuilder::new()
        .schema_lookup_from_db(ledger.substate_db())
        .encoder(&encoder)
        .build();
    println!("\n{}", receipt.display(display_context));
}

#[test]
fn transaction_with_large_payload_but_insufficient_fee_payment_should_be_rejected() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(true);
    let resource_address = ledger.create_non_fungible_resource(account);

    let manifest = ManifestBuilder::new()
        .lock_fee(account, dec!("0.2"))
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);
    receipt.expect_commit_success();

    let manifest = ManifestBuilder::new()
        .lock_fee(account, dec!("0.2"))
        .then(|mut builder| {
            for _ in 0..10_000 {
                // 640 KB
                builder = builder.assert_worktop_contains_non_fungibles(
                    resource_address,
                    [NonFungibleLocalId::bytes([0u8; NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH]).unwrap()],
                )
            }
            builder
        })
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);
    receipt.expect_specific_rejection(|e| {
        matches!(e, RejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(_))
    })
}
