use radix_common::constants::MAX_CALL_DEPTH;
use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::limits::TransactionLimitsError;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn test_max_call_depth_success() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("recursion"));

    // Act
    // ============================
    // Stack layout:
    // * 0: Executor
    // * 1: Transaction Executor
    // * 2-15: Caller::call x 14
    // ============================
    let num_calls = u32::try_from(MAX_CALL_DEPTH).unwrap() - 1u32;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Caller",
            "recursive",
            manifest_args!(num_calls),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_max_call_depth_failure() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("recursion"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Caller",
            "recursive",
            manifest_args!(16u32),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::MaxCallDepthLimitReached
            ))
        )
    });
}









#[test]
fn test() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (key, _, account) = ledger.new_allocated_account();
    let (key2, _, account2) = ledger.new_allocated_account();
    let (btc_mint_auth, btc) = ledger.create_mintable_burnable_fungible_resource_with_initial_amount(account, Some(Decimal::from(1000)));
    let (_, usdc) = ledger.create_mintable_burnable_fungible_resource_with_initial_amount(account2, Some(Decimal::from(1000)));

    let execution_config = {
        let mut execution_config = ExecutionConfig::for_test_transaction();
        execution_config.system_overrides = Some(SystemOverrides {
            disable_costing: true,
            disable_limits: true,
            disable_auth: false,
            ..Default::default()
        });
        execution_config
    };
    let thread0 = {
        let mut manifest = ManifestBuilder::new()
            .withdraw_from_account(account, btc, Decimal::from(2))
            .deposit_batch(account)
            .send_to_subtransaction(&())
            .build();

        let (instructions, blobs) = manifest.for_intent();

        let prepared_instructions = instructions.prepare_partial().unwrap();
        let encoded_instructions = manifest_encode(&prepared_instructions.inner.0).unwrap();
        let references = prepared_instructions.references;
        let blobs = blobs.prepare_partial().unwrap().blobs_by_hash;
        ExecutableThread {
            encoded_instructions: Rc::new(encoded_instructions),
            references,
            blobs: Rc::new(blobs),
            pre_allocated_addresses: vec![],
        }
    };

    let thread1 = {
        let mut manifest = ManifestBuilder::new()
            .withdraw_from_account(account2, usdc, Decimal::from(2))
            .deposit_batch(account2)
            .build();

        let (instructions, blobs) = manifest.for_intent();

        let prepared_instructions = instructions.prepare_partial().unwrap();
        let encoded_instructions = manifest_encode(&prepared_instructions.inner.0).unwrap();
        let references = prepared_instructions.references;
        let blobs = blobs.prepare_partial().unwrap().blobs_by_hash;
        ExecutableThread {
            encoded_instructions: Rc::new(encoded_instructions),
            references,
            blobs: Rc::new(blobs),
            pre_allocated_addresses: vec![],
        }
    };



    let executable = Executable {
        threads: vec![thread0, thread1],
        context: ExecutionContext {
            intent_hash: TransactionIntentHash::NotToCheck {
                intent_hash: Hash([0u8; Hash::LENGTH])
            },
            epoch_range: None,
            payload_size: 0usize,
            num_of_signature_validations: 1,
            auth_zone_params: AuthZoneParams {
                thread_params: vec![
                    AuthZoneThreadParams {
                        initial_proofs: btreeset!(NonFungibleGlobalId::from_public_key(&key)),
                        virtual_resources: Default::default(),
                    },
                    AuthZoneThreadParams {
                        initial_proofs: btreeset!(NonFungibleGlobalId::from_public_key(&key2)),
                        virtual_resources: Default::default(),
                    },
                ]
            },
            costing_parameters: TransactionCostingParameters {
                tip_percentage: DEFAULT_TIP_PERCENTAGE,
                free_credit_in_xrd: Decimal::ZERO,
                abort_when_loan_repaid: false,
            },
        },
        system: false,
    };

    let receipt = ledger.execute_transaction(executable, execution_config);

    // Assert
    receipt.expect_commit_success();
}

