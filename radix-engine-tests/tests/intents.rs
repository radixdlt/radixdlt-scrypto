use std::rc::Rc;
use radix_common::constants::DEFAULT_TIP_PERCENTAGE;
use radix_common::crypto::{Hash, HasPublicKeyHash};
use radix_common::manifest_args;
use radix_common::math::Decimal;
use radix_common::prelude::{FromPublicKey, manifest_encode, NonFungibleGlobalId, Reference};
use radix_engine::transaction::{ExecutionConfig, SystemOverrides};
use radix_rust::btreeset;
use radix_rust::prelude::IndexSet;
use radix_transactions::builder::ManifestBuilder;
use radix_transactions::model::{AuthZoneParams, Executable, ExecutableIntent, ExecutionContext, TransactionCostingParameters, TransactionManifestV1, TransactionPartialEncode};
use scrypto::prelude::DIVISIBILITY_MAXIMUM;
use scrypto_test::ledger_simulator::LedgerSimulatorBuilder;
use scrypto_test::prelude::LedgerSimulatorResourceExtension;
use radix_common::prelude::ManifestArgs;

fn manifest_to_executable_intent<P: HasPublicKeyHash>(intent_hash: Hash, manifest: TransactionManifestV1, key: &P) -> (ExecutableIntent, IndexSet<Reference>) {
    let (instructions, blobs) = manifest.for_intent();

    let prepared_instructions = instructions.prepare_partial().unwrap();
    let references = prepared_instructions.references;
    let encoded_instructions = manifest_encode(&prepared_instructions.inner.0).unwrap();
    let blobs = blobs.prepare_partial().unwrap().blobs_by_hash;
    let intent = ExecutableIntent {
        intent_hash,
        encoded_instructions: Rc::new(encoded_instructions),
        blobs,
        auth_zone_params:  AuthZoneParams {
            initial_proofs: btreeset!(NonFungibleGlobalId::from_public_key(key)),
            virtual_resources: Default::default(),
        },
    };
    (intent, references)
}

#[test]
fn swap_with_intents_should_work() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (key, _, account) = ledger.new_allocated_account();
    let (key2, _, account2) = ledger.new_allocated_account();
    let (main_key, _, main_account) = ledger.new_allocated_account();
    let btc = ledger.create_fungible_resource(Decimal::from(1000), DIVISIBILITY_MAXIMUM, account);
    let usdc = ledger.create_fungible_resource(Decimal::from(1000), DIVISIBILITY_MAXIMUM, account2);

    let execution_config = {
        let mut execution_config = ExecutionConfig::for_test_transaction();
        execution_config.enable_kernel_trace = false;
        execution_config
    };

    let mut all_references = IndexSet::new();

    let child_intent0 = {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(account2, usdc, Decimal::from(23))
            .take_all_from_worktop(usdc, "usdc")
            .with_name_lookup(|builder, lookup| {
                builder.yield_to_parent(manifest_args!(lookup.bucket("usdc")))
            })
            .assert_worktop_contains(btc, Decimal::from(2))
            .take_all_from_worktop(btc, "btc")
            .with_name_lookup(|builder, lookup| {
                builder.deposit(account2, lookup.bucket("btc"))
            })
            .build();

        let (intent, references) = manifest_to_executable_intent(Hash([0u8; Hash::LENGTH]), manifest, &key2);
        all_references.extend(references);
        intent
    };

    let child_intent1 = {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(account, btc, Decimal::from(2))
            .take_all_from_worktop(btc, "btc")
            .with_name_lookup(|builder, lookup| {
                builder.yield_to_parent(manifest_args!(lookup.bucket("btc")))
            })
            .assert_worktop_contains(usdc, Decimal::from(23))
            .take_all_from_worktop(usdc, "usdc")
            .with_name_lookup(|builder, lookup| {
                builder.deposit(account, lookup.bucket("usdc"))
            })
            .build();

        let (intent, references) = manifest_to_executable_intent(Hash([1u8; Hash::LENGTH]), manifest, &key);
        all_references.extend(references);
        intent
    };

    let parent_intent = {
        let manifest = ManifestBuilder::new()
            .lock_fee(main_account, Decimal::from(10))
            .yield_to_child(child_intent0.intent_hash, manifest_args!(&()))
            .take_all_from_worktop(usdc, "usdc")
            .with_name_lookup(|builder, lookup| {
                builder.yield_to_child(child_intent1.intent_hash, manifest_args!(lookup.bucket("usdc")))
            })
            .take_all_from_worktop(btc, "btc")
            .with_name_lookup(|builder, lookup| {
                builder.yield_to_child(child_intent0.intent_hash, manifest_args!(lookup.bucket("btc")))
            })
            .yield_to_child(child_intent1.intent_hash, manifest_args!(&()))
            .deposit_batch(main_account)
            .build();

        let (intent, references) = manifest_to_executable_intent(Hash([2u8; Hash::LENGTH]), manifest, &main_key);
        all_references.extend(references);
        intent
    };

    let executable = Executable {
        intents: vec![parent_intent, child_intent0, child_intent1],
        references: all_references.into_iter().collect(),
        context: ExecutionContext {
            nullifier_updates: Default::default(),
            payload_size: 0usize,
            num_of_signature_validations: 1,
            costing_parameters: TransactionCostingParameters {
                tip_percentage: DEFAULT_TIP_PERCENTAGE,
                free_credit_in_xrd: Decimal::ZERO,
                abort_when_loan_repaid: false,
            },
            pre_allocated_addresses: vec![],
        },
        system: false,
    };

    // Act
    let receipt = ledger.execute_transaction(executable, execution_config);

    // Assert
    receipt.expect_commit_success();
}
