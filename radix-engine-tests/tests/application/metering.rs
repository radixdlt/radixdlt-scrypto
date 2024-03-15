use radix_common::prelude::*;
use radix_engine::errors::RejectionReason;
use radix_engine::transaction::CostingParameters;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::transaction::TransactionFeeDetails;
use radix_engine::transaction::TransactionFeeSummary;
use radix_engine::transaction::TransactionReceipt;
use radix_engine_interface::blueprints::access_controller::ACCESS_CONTROLLER_CREATE_PROOF_IDENT;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_tests::common::*;
use scrypto::object_modules::ModuleConfig;
use scrypto::prelude::metadata;
use scrypto::prelude::metadata_init;
use scrypto_test::prelude::*;

// For WASM-specific metering tests, see `wasm_metering.rs`.

#[cfg(not(feature = "alloc"))]
#[test]
#[ignore = "Run this test to update expected costs"]
fn update_expected_costs() {
    run_basic_transfer(Mode::OutputCosting(
        path_local_meterng_csv!("cost_transfer.csv").to_string(),
    ));
    run_basic_transfer_to_virtual_account(Mode::OutputCosting(
        path_local_meterng_csv!("cost_transfer_to_virtual_account.csv").to_string(),
    ));
    run_radiswap(
        Mode::OutputCosting(path_local_meterng_csv!("cost_radiswap.csv").to_string()),
        false,
    );
    run_radiswap(
        Mode::OutputCosting(path_local_meterng_csv!("cost_radiswap_pools_v1_1.csv").to_string()),
        true,
    );
    run_flash_loan(Mode::OutputCosting(
        path_local_meterng_csv!("cost_flash_loan.csv").to_string(),
    ));
    run_publish_large_package(Mode::OutputCosting(
        path_local_meterng_csv!("cost_publish_large_package.csv").to_string(),
    ));
    run_mint_large_size_nfts_from_manifest(Mode::OutputCosting(
        path_local_meterng_csv!("cost_mint_large_size_nfts_from_manifest.csv").to_string(),
    ));
    run_mint_small_size_nfts_from_manifest(Mode::OutputCosting(
        path_local_meterng_csv!("cost_mint_small_size_nfts_from_manifest.csv").to_string(),
    ));
}

#[test]
fn test_basic_transfer() {
    run_basic_transfer(Mode::AssertCosting(load_cost_breakdown(
        include_local_meterng_csv_str!("cost_transfer.csv"),
    )));
}

#[test]
fn test_transfer_to_virtual_account() {
    run_basic_transfer_to_virtual_account(Mode::AssertCosting(load_cost_breakdown(
        include_local_meterng_csv_str!("cost_transfer_to_virtual_account.csv"),
    )));
}

#[test]
fn test_radiswap() {
    run_radiswap(
        Mode::AssertCosting(load_cost_breakdown(include_local_meterng_csv_str!(
            "cost_radiswap.csv"
        ))),
        false,
    );
}

#[test]
fn test_radiswap_with_pools_v1_1() {
    run_radiswap(
        Mode::AssertCosting(load_cost_breakdown(include_local_meterng_csv_str!(
            "cost_radiswap_pools_v1_1.csv"
        ))),
        true,
    );
}

#[test]
fn test_flash_loan() {
    run_flash_loan(Mode::AssertCosting(load_cost_breakdown(
        include_local_meterng_csv_str!("cost_flash_loan.csv"),
    )));
}

#[test]
fn test_publish_large_package() {
    run_publish_large_package(Mode::AssertCosting(load_cost_breakdown(
        include_local_meterng_csv_str!("cost_publish_large_package.csv"),
    )));
}

#[test]
fn test_mint_large_size_nfts_from_manifest() {
    run_mint_large_size_nfts_from_manifest(Mode::AssertCosting(load_cost_breakdown(
        include_local_meterng_csv_str!("cost_mint_large_size_nfts_from_manifest.csv"),
    )));
}

#[test]
fn test_mint_small_size_nfts_from_manifest() {
    run_mint_small_size_nfts_from_manifest(Mode::AssertCosting(load_cost_breakdown(
        include_local_meterng_csv_str!("cost_mint_small_size_nfts_from_manifest.csv"),
    )));
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

pub fn load_cost_breakdown(content: &str) -> (BTreeMap<String, u32>, BTreeMap<String, u32>) {
    let mut execution_breakdown = BTreeMap::<String, u32>::new();
    let mut finalization_breakdown = BTreeMap::<String, u32>::new();
    let lines: Vec<String> = content.split('\n').map(String::from).collect();
    let mut is_execution = true;
    for i in 8..lines.len() {
        if lines[i].starts_with('-') {
            let mut tokens = lines[i].split(',');
            let entry = tokens.next().unwrap().trim()[2..].to_string();
            let cost = tokens.next().unwrap().trim();
            if is_execution {
                &mut execution_breakdown
            } else {
                &mut finalization_breakdown
            }
            .insert(entry, u32::from_str(cost).unwrap());
        } else {
            is_execution = false;
        }
    }
    (execution_breakdown, finalization_breakdown)
}

#[cfg(feature = "alloc")]
pub fn write_cost_breakdown(
    _fee_summary: &TransactionFeeSummary,
    _fee_details: &TransactionFeeDetails,
    _file: &str,
) {
}

#[cfg(not(feature = "alloc"))]
pub fn write_cost_breakdown(
    fee_summary: &TransactionFeeSummary,
    fee_details: &TransactionFeeDetails,
    file: &str,
) {
    use std::fs::File;
    use std::io::Write;

    fn decimal_to_float(d: Decimal) -> f64 {
        f64::from_str(d.to_string().as_str()).unwrap()
    }

    let mut buffer = String::new();
    buffer.push_str(
        format!(
            "{:<75},{:>25}, {:8.1}%\n",
            "Total Cost (XRD)",
            fee_summary.total_cost().to_string(),
            100.0
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>25}, {:8.1}%\n",
            "- Execution Cost (XRD)",
            fee_summary.total_execution_cost_in_xrd.to_string(),
            decimal_to_float(
                fee_summary
                    .total_execution_cost_in_xrd
                    .checked_div(fee_summary.total_cost())
                    .unwrap()
                    .checked_mul(100)
                    .unwrap()
            )
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>25}, {:8.1}%\n",
            "- Finalization Cost (XRD)",
            fee_summary.total_finalization_cost_in_xrd.to_string(),
            decimal_to_float(
                fee_summary
                    .total_finalization_cost_in_xrd
                    .checked_div(fee_summary.total_cost())
                    .unwrap()
                    .checked_mul(100)
                    .unwrap()
            )
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>25}, {:8.1}%\n",
            "- Tipping Cost (XRD)",
            fee_summary.total_tipping_cost_in_xrd.to_string(),
            decimal_to_float(
                fee_summary
                    .total_tipping_cost_in_xrd
                    .checked_mul(fee_summary.total_cost())
                    .unwrap()
                    .checked_mul(100)
                    .unwrap()
            )
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>25}, {:8.1}%\n",
            "- Storage Cost (XRD)",
            fee_summary.total_storage_cost_in_xrd.to_string(),
            decimal_to_float(
                fee_summary
                    .total_storage_cost_in_xrd
                    .checked_div(fee_summary.total_cost())
                    .unwrap()
                    .checked_mul(100)
                    .unwrap()
            )
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>25}, {:8.1}%\n",
            "- Tipping Cost (XRD)",
            fee_summary.total_tipping_cost_in_xrd.to_string(),
            decimal_to_float(
                fee_summary
                    .total_tipping_cost_in_xrd
                    .checked_div(fee_summary.total_cost())
                    .unwrap()
                    .checked_mul(100)
                    .unwrap()
            )
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>25}, {:8.1}%\n",
            "- Royalty Cost (XRD)",
            fee_summary.total_royalty_cost_in_xrd.to_string(),
            decimal_to_float(
                fee_summary
                    .total_royalty_cost_in_xrd
                    .checked_div(fee_summary.total_cost())
                    .unwrap()
                    .checked_mul(100)
                    .unwrap()
            )
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>25}, {:8.1}%\n",
            "Execution Cost Breakdown",
            fee_details.execution_cost_breakdown.values().sum::<u32>(),
            100.0
        )
        .as_str(),
    );
    for (k, v) in &fee_details.execution_cost_breakdown {
        buffer.push_str(
            format!(
                "- {:<73},{:>25}, {:8.1}%\n",
                k,
                v,
                decimal_to_float(
                    Decimal::from(*v)
                        .checked_div(Decimal::from(
                            fee_summary.total_execution_cost_units_consumed
                        ))
                        .unwrap()
                        .checked_mul(100)
                        .unwrap()
                )
            )
            .as_str(),
        );
    }
    buffer.push_str(
        format!(
            "{:<75},{:>25}, {:8.1}%\n",
            "Finalization Cost Breakdown",
            fee_details
                .finalization_cost_breakdown
                .values()
                .sum::<u32>(),
            100.0
        )
        .as_str(),
    );
    for (k, v) in &fee_details.finalization_cost_breakdown {
        buffer.push_str(
            format!(
                "- {:<73},{:>25}, {:8.1}%\n",
                k,
                v,
                decimal_to_float(
                    Decimal::from(*v)
                        .checked_div(Decimal::from(
                            fee_summary.total_finalization_cost_units_consumed
                        ))
                        .unwrap()
                        .checked_mul(100)
                        .unwrap()
                )
            )
            .as_str(),
        );
    }

    let mut f = File::create(file).unwrap();
    f.write_all(buffer.as_bytes()).unwrap();
}

pub enum Mode {
    OutputCosting(String),
    AssertCosting((BTreeMap<String, u32>, BTreeMap<String, u32>)),
}

impl Mode {
    pub fn run(&self, fee_summary: &TransactionFeeSummary, fee_details: &TransactionFeeDetails) {
        match self {
            Mode::OutputCosting(file) => {
                write_cost_breakdown(fee_summary, fee_details, file.as_str());
            }
            Mode::AssertCosting(expected) => {
                assert_eq!(&fee_details.execution_cost_breakdown, &expected.0);
                assert_eq!(&fee_details.finalization_cost_breakdown, &expected.1);
            }
        }
    }
}

fn run_basic_transfer(mode: Mode) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
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

    mode.run(&receipt.fee_summary, &receipt.fee_details.unwrap());
}

fn run_basic_transfer_to_virtual_account(mode: Mode) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key1, _, account1) = ledger.new_allocated_account();
    let account2 = ComponentAddress::virtual_account_from_public_key(&PublicKey::Secp256k1(
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

    mode.run(&receipt.fee_summary, &receipt.fee_details.unwrap());
}

fn run_radiswap(mode: Mode, use_v1_1_pools: bool) {
    let mut ledger = if use_v1_1_pools {
        LedgerSimulatorBuilder::new().build()
    } else {
        LedgerSimulatorBuilder::new().without_pools_v1_1().build()
    };

    // Scrypto developer
    let (pk1, _, _) = ledger.new_allocated_account();
    // Radiswap operator
    let (pk2, _, account2) = ledger.new_allocated_account();
    // Radiswap user
    let (pk3, _, account3) = ledger.new_allocated_account();

    // Publish package
    let package_address = ledger.publish_package(
        (
            include_workspace_asset_bytes!("radiswap.wasm").to_vec(),
            manifest_decode(include_workspace_asset_bytes!("radiswap.rpd")).unwrap(),
        ),
        btreemap!(),
        OwnerRole::Fixed(rule!(require(NonFungibleGlobalId::from_public_key(&pk1)))),
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

    mode.run(&receipt.fee_summary, &receipt.fee_details.unwrap());
}

fn run_flash_loan(mode: Mode) {
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Scrypto developer
    let (pk1, _, _) = ledger.new_allocated_account();
    // Flash loan operator
    let (pk2, _, account2) = ledger.new_allocated_account();
    // Flash loan user
    let (pk3, _, account3) = ledger.new_allocated_account();

    // Publish package
    let package_address = ledger.publish_package(
        (
            include_workspace_asset_bytes!("flash_loan.wasm").to_vec(),
            manifest_decode(include_workspace_asset_bytes!("flash_loan.rpd")).unwrap(),
        ),
        btreemap!(),
        OwnerRole::Fixed(rule!(require(NonFungibleGlobalId::from_public_key(&pk1)))),
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
    mode.run(&receipt.fee_summary, &receipt.fee_details.unwrap());
}

fn run_publish_large_package(mode: Mode) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

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
    mode.run(&receipt.fee_summary, &receipt.fee_details.unwrap());
}

fn run_mint_small_size_nfts_from_manifest(mode: Mode) {
    run_mint_nfts_from_manifest(
        mode,
        TestNonFungibleData {
            metadata: btreemap!(),
        },
    )
}

fn run_mint_large_size_nfts_from_manifest(mode: Mode) {
    const N: usize = 50;

    run_mint_nfts_from_manifest(
        mode,
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

fn run_mint_nfts_from_manifest(mode: Mode, nft_data: TestNonFungibleData) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
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
        if raw_transaction.0.len() > MAX_TRANSACTION_SIZE {
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
    println!(
        "Transaction payload size: {} bytes",
        raw_transaction.0.len()
    );
    println!(
        "Average NFT size: {} bytes",
        scrypto_encode(&nft_data).unwrap().len()
    );
    println!("Managed to mint {} NFTs", n);
    mode.run(&receipt.fee_summary, &receipt.fee_details.unwrap());
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
    let code = include_workspace_asset_bytes!("large_package.wasm").to_vec();
    let definition = manifest_decode(include_workspace_asset_bytes!("large_package.rpd")).unwrap();
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
        ledger.new_ed25519_virtual_account_with_access_controller(3);

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
        validate_notarized_transaction(&network, &tx1).get_executable(),
        CostingParameters::default(),
        ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator())
            .with_cost_breakdown(true),
    );
    receipt.expect_commit_success();
    println!("\n{:?}", receipt);
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
