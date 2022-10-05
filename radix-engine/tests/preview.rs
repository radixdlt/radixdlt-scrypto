use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig};
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::*;
use transaction::signing::EcdsaSecp256k1PrivateKey;
use transaction::validation::{NotarizedTransactionValidator, TestIntentHashManager};
use transaction::validation::{TransactionValidator, ValidationConfig};

#[test]
fn test_transaction_preview_cost_estimate() {
    // Arrange
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut substate_store);
    let network = NetworkDefinition::simulator();
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .clear_auth_zone()
        .build();
    let preview_flags = PreviewFlags {
        unlimited_loan: true,
        assume_all_signature_proofs: false,
    };
    let (validated_transaction, preview_intent) =
        prepare_test_tx_and_preview_intent(&test_runner, &network, manifest, &preview_flags);

    // Act & Assert: Execute the preview, followed by a normal execution.
    // Ensure that both succeed and that the preview result provides an accurate cost estimate
    let preview_result = test_runner.execute_preview(preview_intent, &network);
    let preview_receipt = preview_result.unwrap().receipt;
    preview_receipt.expect_commit_success();

    let receipt = test_runner.execute_transaction(
        &validated_transaction,
        &FeeReserveConfig::standard(),
        &ExecutionConfig::standard(),
    );
    receipt.expect_commit_success();

    assert_eq!(
        preview_receipt.execution.fee_summary.cost_unit_consumed,
        receipt.execution.fee_summary.cost_unit_consumed
    );
}

#[test]
fn test_assume_all_signature_proofs_flag_create_proof_instruction() {
    // Arrange
    // Create an account component that requires a key auth for withdrawal
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut substate_store);
    let network = NetworkDefinition::simulator();

    let public_key = EcdsaSecp256k1PrivateKey::from_u64(99).unwrap().public_key();
    let withdraw_auth = rule!(require(NonFungibleAddress::from_public_key(&public_key)));
    let account = test_runner.new_account_with_auth_rule(&withdraw_auth);
    let (_, _, other_account) = test_runner.new_account();

    let preview_flags = PreviewFlags {
        unlimited_loan: true,
        assume_all_signature_proofs: true,
    };

    // Manually create a proof via a manifest instruction and push it to the auth zone
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .create_proof_from_auth_zone_by_ids(
            &BTreeSet::from([NonFungibleAddress::from_public_key(&public_key).non_fungible_id()]),
            ECDSA_TOKEN,
            |builder, proof_id| {
                builder
                    .push_to_auth_zone(proof_id)
                    .lock_fee(10.into(), account)
                    .withdraw_from_account(RADIX_TOKEN, account)
                    .call_method(
                        other_account,
                        "deposit_batch",
                        args!(Expression::entire_worktop()),
                    )
            },
        )
        .build();

    let (_, preview_intent) =
        prepare_test_tx_and_preview_intent(&test_runner, &network, manifest, &preview_flags);

    // Act
    let result = test_runner.execute_preview(preview_intent, &network);

    // Assert
    result.unwrap().receipt.expect_commit_success();
}

#[test]
fn test_assume_all_signature_proofs_flag_method_authorization() {
    // Arrange
    // Create an account component that requires a key auth for withdrawal
    let mut substate_store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut substate_store);
    let network = NetworkDefinition::simulator();

    let public_key = EcdsaSecp256k1PrivateKey::from_u64(99).unwrap().public_key();
    let withdraw_auth = rule!(require(NonFungibleAddress::from_public_key(&public_key)));
    let account = test_runner.new_account_with_auth_rule(&withdraw_auth);
    let (_, _, other_account) = test_runner.new_account();

    let preview_flags = PreviewFlags {
        unlimited_loan: true,
        assume_all_signature_proofs: true,
    };

    // Check method authorization (withdrawal) without a proof in the auth zone
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), account)
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method(
            other_account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();

    let (_, preview_intent) =
        prepare_test_tx_and_preview_intent(&test_runner, &network, manifest, &preview_flags);

    // Act
    let result = test_runner.execute_preview(preview_intent, &network);

    // Assert
    result.unwrap().receipt.expect_commit_success();
}

fn prepare_test_tx_and_preview_intent(
    test_runner: &TestRunner<TypedInMemorySubstateStore>,
    network: &NetworkDefinition,
    manifest: TransactionManifest,
    flags: &PreviewFlags,
) -> (Executable, PreviewIntent) {
    let notary_priv_key = EcdsaSecp256k1PrivateKey::from_u64(2).unwrap();
    let tx_signer_priv_key = EcdsaSecp256k1PrivateKey::from_u64(3).unwrap();

    let notarized_transaction = TransactionBuilder::new()
        .header(TransactionHeader {
            version: 1,
            network_id: network.id,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 99,
            nonce: test_runner.next_transaction_nonce(),
            notary_public_key: notary_priv_key.public_key().into(),
            notary_as_signatory: false,
            cost_unit_limit: 10_000_000,
            tip_percentage: 0,
        })
        .manifest(manifest)
        .sign(&tx_signer_priv_key)
        .notarize(&notary_priv_key)
        .build();

    let validator = NotarizedTransactionValidator::new(ValidationConfig {
        network_id: network.id,
        current_epoch: 1,
        max_cost_unit_limit: 10_000_000,
        min_tip_percentage: 0,
    });

    let validated_transaction = validator
        .validate(notarized_transaction.clone(), &TestIntentHashManager::new())
        .unwrap();

    let preview_intent = PreviewIntent {
        intent: notarized_transaction.signed_intent.intent.clone(),
        signer_public_keys: vec![tx_signer_priv_key.public_key().into()],
        flags: flags.clone(),
    };

    (validated_transaction, preview_intent)
}
