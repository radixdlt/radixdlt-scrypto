use radix_engine::system::system_modules::costing::FeeSummary;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_unit::*;
use transaction::builder::*;
use utils::ContextualDisplay;

// For WASM-specific metering tests, see `wasm_metering.rs`.

#[cfg(not(feature = "alloc"))]
#[test]
#[ignore = "Run this test to update expected costs"]
fn update_expected_costs() {
    run_basic_transfer(Mode::OutputCosting(
        "./assets/cost_transfer.csv".to_string(),
    ));
    run_basic_transfer_to_virtual_account(Mode::OutputCosting(
        "./assets/cost_transfer_to_virtual_account.csv".to_string(),
    ));
    run_radiswap(Mode::OutputCosting(
        "./assets/cost_radiswap.csv".to_string(),
    ));
    run_flash_loan(Mode::OutputCosting(
        "./assets/cost_flash_loan.csv".to_string(),
    ));
    run_publish_large_package(Mode::OutputCosting(
        "./assets/cost_publish_large_package.csv".to_string(),
    ));
}

#[test]
fn test_basic_transfer() {
    run_basic_transfer(Mode::AssertCosting(load_cost_breakdown(include_str!(
        "../assets/cost_transfer.csv"
    ))));
}

#[test]
fn test_transfer_to_virtual_account() {
    run_basic_transfer_to_virtual_account(Mode::AssertCosting(load_cost_breakdown(include_str!(
        "../assets/cost_transfer_to_virtual_account.csv"
    ))));
}

#[test]
fn test_radiswap() {
    run_radiswap(Mode::AssertCosting(load_cost_breakdown(include_str!(
        "../assets/cost_radiswap.csv"
    ))));
}

#[test]
fn test_flash_loan() {
    run_flash_loan(Mode::AssertCosting(load_cost_breakdown(include_str!(
        "../assets/cost_flash_loan.csv"
    ))));
}

#[test]
fn test_publish_large_package() {
    run_publish_large_package(Mode::AssertCosting(load_cost_breakdown(include_str!(
        "../assets/cost_publish_large_package.csv"
    ))));
}

#[cfg(feature = "std")]
fn execute_with_time_logging(
    test_runner: &mut TestRunner,
    manifest: TransactionManifestV1,
    proofs: Vec<NonFungibleGlobalId>,
) -> (TransactionReceipt, u32) {
    let start = std::time::Instant::now();
    let receipt = test_runner.execute_manifest(manifest, proofs);
    let duration = start.elapsed();
    println!(
        "Time elapsed is: {:?} - NOTE: this is a very bad measure. Use benchmarks instead.",
        duration
    );
    (receipt, duration.as_millis().try_into().unwrap())
}

#[cfg(feature = "alloc")]
fn execute_with_time_logging(
    test_runner: &mut TestRunner,
    manifest: TransactionManifestV1,
    proofs: Vec<NonFungibleGlobalId>,
) -> (TransactionReceipt, u32) {
    let receipt = test_runner.execute_manifest(manifest, proofs);
    (receipt, 0)
}

pub fn load_cost_breakdown(content: &str) -> BTreeMap<String, u32> {
    let mut breakdown = BTreeMap::<String, u32>::new();
    content
        .split("\n")
        .filter(|x| x.len() > 0)
        .skip(6)
        .for_each(|x| {
            let mut tokens = x.split(",");
            let entry = tokens.next().unwrap().trim();
            let cost = tokens.next().unwrap().trim();
            breakdown.insert(entry.to_string(), u32::from_str(cost).unwrap());
        });
    breakdown
}

#[cfg(feature = "alloc")]
pub fn write_cost_breakdown(_fee_summary: &FeeSummary, _file: &str) {}

#[cfg(not(feature = "alloc"))]
pub fn write_cost_breakdown(fee_summary: &FeeSummary, file: &str) {
    use std::fs::File;
    use std::io::Write;

    fn decimal_to_float(d: Decimal) -> f64 {
        f64::from_str(d.to_string().as_str()).unwrap()
    }

    let mut buffer = String::new();
    buffer.push_str(
        format!(
            "{:<75},{:>15}, {:8.2}%\n",
            "Total Cost (XRD)",
            fee_summary.total_cost().to_string(),
            100.0
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>15}, {:8.2}%\n",
            "+ Execution Cost (XRD)",
            fee_summary.total_execution_cost_xrd.to_string(),
            decimal_to_float(fee_summary.total_execution_cost_xrd / fee_summary.total_cost() * 100)
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>15}, {:8.2}%\n",
            "+ Tipping Cost (XRD)",
            fee_summary.total_tipping_cost_xrd.to_string(),
            decimal_to_float(fee_summary.total_tipping_cost_xrd / fee_summary.total_cost() * 100)
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>15}, {:8.2}%\n",
            "+ State Expansion Cost (XRD)",
            fee_summary.total_state_expansion_cost_xrd.to_string(),
            decimal_to_float(
                fee_summary.total_state_expansion_cost_xrd / fee_summary.total_cost() * 100
            )
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>15}, {:8.2}%\n",
            "+ Royalty Cost (XRD)",
            fee_summary.total_royalty_cost_xrd.to_string(),
            decimal_to_float(fee_summary.total_royalty_cost_xrd / fee_summary.total_cost() * 100)
        )
        .as_str(),
    );
    buffer.push_str(
        format!(
            "{:<75},{:>15}, {:8.2}%\n",
            "Total Cost Units Consumed",
            fee_summary.execution_cost_breakdown.values().sum::<u32>(),
            100.0
        )
        .as_str(),
    );
    for (k, v) in &fee_summary.execution_cost_breakdown {
        buffer.push_str(
            format!(
                "{:<75},{:>15}, {:8.2}%\n",
                k,
                v,
                decimal_to_float(
                    Decimal::from(*v) / Decimal::from(fee_summary.execution_cost_sum) * 100
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
    AssertCosting(BTreeMap<String, u32>),
}

impl Mode {
    pub fn run(&self, fee_summary: &FeeSummary) {
        match self {
            Mode::OutputCosting(file) => {
                write_cost_breakdown(fee_summary, file.as_str());
            }
            Mode::AssertCosting(expected) => {
                assert_eq!(&fee_summary.execution_cost_breakdown, expected);
            }
        }
    }
}

fn run_basic_transfer(mode: Mode) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key1, _, account1) = test_runner.new_allocated_account();
    let (_, _, account2) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, 500u32.into())
        .withdraw_from_account(account1, RADIX_TOKEN, 100u32.into())
        .call_method(
            account2,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let (receipt, _) = execute_with_time_logging(
        &mut test_runner,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key1)],
    );
    let commit_result = receipt.expect_commit(true);

    mode.run(&commit_result.fee_summary);
}

fn run_basic_transfer_to_virtual_account(mode: Mode) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key1, _, account1) = test_runner.new_allocated_account();
    let account2 = ComponentAddress::virtual_account_from_public_key(&PublicKey::Secp256k1(
        Secp256k1PublicKey([123u8; 33]),
    ));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, 500u32.into())
        .withdraw_from_account(account1, RADIX_TOKEN, 100u32.into())
        .call_method(
            account2,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let (receipt, _) = execute_with_time_logging(
        &mut test_runner,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key1)],
    );
    let commit_result = receipt.expect_commit(true);

    mode.run(&commit_result.fee_summary);
}

fn run_radiswap(mode: Mode) {
    let mut test_runner = TestRunner::builder().build();

    // Scrypto developer
    let (pk1, _, _) = test_runner.new_allocated_account();
    // Radiswap operator
    let (pk2, _, account2) = test_runner.new_allocated_account();
    // Radiswap user
    let (pk3, _, account3) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.publish_package(
        include_bytes!("../../assets/radiswap.wasm").to_vec(),
        manifest_decode(include_bytes!("../../assets/radiswap.schema")).unwrap(),
        btreemap!(),
        OwnerRole::Fixed(rule!(require(NonFungibleGlobalId::from_public_key(&pk1)))),
    );

    // Instantiate Radiswap
    let btc = test_runner.create_fungible_resource(1_000_000.into(), 18, account2);
    let eth = test_runner.create_fungible_resource(1_000_000.into(), 18, account2);
    let component_address: ComponentAddress = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 500u32.into())
                .call_function(package_address, "Radiswap", "new", manifest_args!(btc, eth))
                .call_method(
                    account2,
                    "try_deposit_batch_or_abort",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit(true)
        .output(1);

    // Contributing an initial amount to radiswap
    let btc_init_amount = Decimal::from(500_000);
    let eth_init_amount = Decimal::from(300_000);
    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 500u32.into())
                .withdraw_from_account(account2, btc, btc_init_amount)
                .withdraw_from_account(account2, eth, eth_init_amount)
                .take_all_from_worktop(btc, |builder, bucket1| {
                    builder.take_all_from_worktop(eth, |builder, bucket2| {
                        builder.call_method(
                            component_address,
                            "add_liquidity",
                            manifest_args!(bucket1, bucket2),
                        )
                    })
                })
                .call_method(
                    account2,
                    "try_deposit_batch_or_abort",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit(true);

    // Transfer `10,000 BTC` from `account2` to `account3`
    let btc_amount = Decimal::from(10_000);
    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 500u32.into())
                .withdraw_from_account(account2, btc, btc_amount)
                .call_method(
                    account3,
                    "try_deposit_batch_or_abort",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit_success();
    assert_eq!(test_runner.account_balance(account3, btc), Some(btc_amount));

    // Swap 2,000 BTC into ETH
    let btc_to_swap = Decimal::from(2000);
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account3, 500u32.into())
            .withdraw_from_account(account3, btc, btc_to_swap)
            .take_all_from_worktop(btc, |builder, bucket| {
                builder.call_method(component_address, "swap", manifest_args!(bucket))
            })
            .call_method(
                account3,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk3)],
    );
    let remaining_btc = test_runner.account_balance(account3, btc).unwrap();
    let eth_received = test_runner.account_balance(account3, eth).unwrap();
    assert_eq!(remaining_btc, btc_amount - btc_to_swap);
    assert_eq!(eth_received, dec!("1195.219123505976095617"));
    let commit_result = receipt.expect_commit(true);

    mode.run(&commit_result.fee_summary);
}

fn run_flash_loan(mode: Mode) {
    let mut test_runner = TestRunner::builder().build();

    // Scrypto developer
    let (pk1, _, _) = test_runner.new_allocated_account();
    // Flash loan operator
    let (pk2, _, account2) = test_runner.new_allocated_account();
    // Flash loan user
    let (pk3, _, account3) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.publish_package(
        include_bytes!("../../assets/flash_loan.wasm").to_vec(),
        manifest_decode(include_bytes!("../../assets/flash_loan.schema")).unwrap(),
        btreemap!(),
        OwnerRole::Fixed(rule!(require(NonFungibleGlobalId::from_public_key(&pk1)))),
    );

    // Instantiate flash_loan
    let xrd_init_amount = Decimal::from(100);
    let (component_address, promise_token_address) = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(account2, 500u32.into())
                .withdraw_from_account(account2, RADIX_TOKEN, xrd_init_amount)
                .take_all_from_worktop(RADIX_TOKEN, |builder, bucket1| {
                    builder.call_function(
                        package_address,
                        "BasicFlashLoan",
                        "instantiate_default",
                        manifest_args!(bucket1),
                    )
                })
                .call_method(
                    account2,
                    "try_deposit_batch_or_abort",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&pk2)],
        )
        .expect_commit(true)
        .output::<(ComponentAddress, ResourceAddress)>(3);

    // Take loan
    let loan_amount = Decimal::from(50);
    let repay_amount = loan_amount * dec!("1.001");
    let old_balance = test_runner.account_balance(account3, RADIX_TOKEN).unwrap();
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account3, 500u32.into())
            .call_method(component_address, "take_loan", manifest_args!(loan_amount))
            .withdraw_from_account(account3, RADIX_TOKEN, dec!(10))
            .take_from_worktop(RADIX_TOKEN, repay_amount, |builder, bucket1| {
                builder.take_all_from_worktop(promise_token_address, |builder, bucket2| {
                    builder.call_method(
                        component_address,
                        "repay_loan",
                        manifest_args!(bucket1, bucket2),
                    )
                })
            })
            .call_method(
                account3,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk3)],
    );
    let commit_result = receipt.expect_commit(true);
    let new_balance = test_runner.account_balance(account3, RADIX_TOKEN).unwrap();
    assert!(test_runner
        .account_balance(account3, promise_token_address)
        .is_none());
    assert_eq!(
        old_balance - new_balance,
        commit_result.fee_summary.total_execution_cost_xrd
            + commit_result.fee_summary.total_tipping_cost_xrd
            + commit_result.fee_summary.total_state_expansion_cost_xrd
            + commit_result.fee_summary.total_royalty_cost_xrd
            + (repay_amount - loan_amount)
    );
    mode.run(&commit_result.fee_summary);
}

fn run_publish_large_package(mode: Mode) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

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
        .lock_fee(test_runner.faucet_component(), 5000u32.into())
        .publish_package_advanced(
            None,
            code,
            PackageDefinition::default(),
            BTreeMap::new(),
            OwnerRole::None,
        )
        .build();

    let (receipt, _) = execute_with_time_logging(&mut test_runner, manifest, vec![]);

    // Assert
    let commit_result = receipt.expect_commit_success();
    mode.run(&commit_result.fee_summary);
}

#[test]
fn should_be_able_run_large_manifest() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let mut builder = ManifestBuilder::new();
    builder.lock_fee(account, 500u32.into());
    builder.withdraw_from_account(account, RADIX_TOKEN, 100u32.into());
    for _ in 0..40 {
        builder.take_from_worktop(RADIX_TOKEN, 1.into(), |builder, bid| {
            builder.return_to_worktop(bid)
        });
    }
    builder.call_method(
        account,
        "try_deposit_batch_or_abort",
        manifest_args!(ManifestExpression::EntireWorktop),
    );
    let manifest = builder.build();

    let (receipt, _) = execute_with_time_logging(
        &mut test_runner,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit(true);
}

#[test]
fn should_be_able_to_generate_5_proofs_and_then_lock_fee() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address = test_runner.create_fungible_resource(100.into(), 0, account);

    // Act
    let mut builder = ManifestBuilder::new();
    for _ in 0..5 {
        builder.create_proof_from_account_of_amount(account, resource_address, 1.into());
    }
    builder.lock_fee(account, 500u32.into());
    let manifest = builder.build();

    let (receipt, _) = execute_with_time_logging(
        &mut test_runner,
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit(true);
}

fn setup_test_runner_with_fee_blueprint_component() -> (TestRunner, ComponentAddress) {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package and instantiate component
    let package_address = test_runner.compile_and_publish("./tests/blueprints/fee");
    let receipt1 = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .withdraw_from_account(account, RADIX_TOKEN, 10u32.into())
            .take_all_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.call_function(package_address, "Fee", "new", manifest_args!(bucket_id));
                builder
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let commit_result = receipt1.expect_commit(true);
    let component_address = commit_result.new_component_addresses()[0];

    (test_runner, component_address)
}

#[test]
fn spin_loop_should_end_in_reasonable_amount_of_time() {
    let (mut test_runner, component_address) = setup_test_runner_with_fee_blueprint_component();

    let manifest = ManifestBuilder::new()
        // First, lock the fee so that the loan will be repaid
        .lock_fee(FAUCET, 500u32.into())
        // Now spin-loop to wait for the fee loan to burn through
        .call_method(component_address, "spin_loop", manifest_args!())
        .build();

    let (receipt, _) = execute_with_time_logging(&mut test_runner, manifest, vec![]);

    // No assertion here - this is just a sanity-test
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );
    receipt.expect_commit_failure();
}
