use radix_common::prelude::*;
use radix_engine::system::system_modules::costing::FeeTable;
use radix_engine::transaction::ExecutionConfig;
use radix_engine_interface::rule;
use scrypto_test::prelude::*;

#[test]
fn test_preview_invalid_direct_access() {
    let mut sim = LedgerSimulatorBuilder::new().build();
    let (public_key, _, _) = sim.new_allocated_account();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_direct_access_method(
            InternalAddress::new_or_panic([0x58; NodeId::LENGTH]),
            "x",
            (),
        )
        .build();

    sim.preview_manifest(
        manifest.clone(),
        vec![],
        0,
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: false,
        },
    );

    sim.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
}

#[test]
fn test_transaction_preview_cost_estimate() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let network = NetworkDefinition::simulator();
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .drop_auth_zone_proofs()
        .build();
    let preview_flags = PreviewFlags {
        use_free_credit: true,
        assume_all_signature_proofs: false,
        skip_epoch_check: false,
        disable_auth: false,
    };
    let (notarized_transaction, preview_intent) = prepare_matching_test_tx_and_preview_intent(
        &mut ledger,
        &network,
        manifest,
        &preview_flags,
    );
    let size_diff = manifest_encode(&notarized_transaction).unwrap().len()
        - manifest_encode(&preview_intent.intent).unwrap().len();

    // Act & Assert: Execute the preview, followed by a normal execution.
    // Ensure that both succeed and that the preview result provides an accurate cost estimate
    let preview_receipt = ledger.preview(preview_intent, &network).unwrap();
    preview_receipt.expect_commit_success();
    let actual_receipt = ledger.execute_transaction(
        notarized_transaction,
        ExecutionConfig::for_notarized_transaction(network.clone())
            .with_kernel_trace(true)
            .with_cost_breakdown(true),
    );
    actual_receipt.expect_commit(true);
    assert_eq!(
        // TODO: better preview payload size estimate?
        preview_receipt
            .fee_summary
            .total_cost()
            .checked_add(
                Decimal::try_from(EXECUTION_COST_UNIT_PRICE_IN_XRD)
                    .unwrap()
                    .checked_mul(FeeTable::latest().validate_tx_payload_cost(size_diff))
                    .unwrap()
            )
            .unwrap()
            .checked_add(
                Decimal::try_from(ARCHIVE_STORAGE_PRICE_IN_XRD)
                    .unwrap()
                    .checked_mul(size_diff)
                    .unwrap()
            )
            .unwrap()
            .checked_add(
                Decimal::try_from(EXECUTION_COST_UNIT_PRICE_IN_XRD)
                    .unwrap()
                    .checked_mul(FeeTable::latest().verify_tx_signatures_cost(2))
                    .unwrap()
            )
            .unwrap(),
        actual_receipt.fee_summary.total_cost(),
    );
}

#[test]
fn test_transaction_preview_without_locking_fee() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let network = NetworkDefinition::simulator();
    let manifest = ManifestBuilder::new()
        // Explicitly don't lock fee from faucet
        .drop_auth_zone_proofs()
        .build();
    let preview_flags = PreviewFlags {
        use_free_credit: true,
        assume_all_signature_proofs: false,
        skip_epoch_check: false,
        disable_auth: false,
    };
    let (_, preview_intent) = prepare_matching_test_tx_and_preview_intent(
        &mut ledger,
        &network,
        manifest,
        &preview_flags,
    );

    // Act
    let preview_receipt = ledger.preview(preview_intent, &network).unwrap();
    let fee_summary = &preview_receipt.fee_summary;
    println!("{:?}", preview_receipt);
    assert!(fee_summary.total_execution_cost_in_xrd.is_positive());
    assert_eq!(fee_summary.total_tipping_cost_in_xrd, dec!("0"));
    assert!(fee_summary.total_storage_cost_in_xrd.is_positive()); // payload cost
    assert_eq!(fee_summary.total_royalty_cost_in_xrd, dec!("0"));
}

#[test]
fn test_assume_all_signature_proofs_flag_method_authorization() {
    // Arrange
    // Create an account component that requires a key auth for withdrawal
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let network = NetworkDefinition::simulator();

    let public_key = Secp256k1PrivateKey::from_u64(99).unwrap().public_key();
    let withdraw_auth = rule!(require(signature(&public_key)));
    let account = ledger.new_account_advanced(OwnerRole::Fixed(withdraw_auth));
    let (_, _, other_account) = ledger.new_allocated_account();

    let preview_flags = PreviewFlags {
        use_free_credit: true,
        assume_all_signature_proofs: true,
        skip_epoch_check: false,
        disable_auth: false,
    };

    // Check method authorization (withdrawal) without a proof in the auth zone
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500)
        .withdraw_from_account(account, XRD, 1)
        .try_deposit_entire_worktop_or_abort(other_account, None)
        .build();

    let (_, preview_intent) = prepare_matching_test_tx_and_preview_intent(
        &mut ledger,
        &network,
        manifest,
        &preview_flags,
    );

    // Act
    let result = ledger.preview(preview_intent, &network);

    // Assert
    result.unwrap().expect_commit_success();
}

#[test]
fn test_preview_no_auth() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let network = NetworkDefinition::simulator();

    let preview_flags = PreviewFlags {
        use_free_credit: true,
        assume_all_signature_proofs: false,
        skip_epoch_check: false,
        disable_auth: true,
    };

    // Everything is possible without auth, even ConsensusManager calls!
    let next_round = 99;
    let manifest = ManifestBuilder::new()
        .call_method(
            CONSENSUS_MANAGER,
            CONSENSUS_MANAGER_NEXT_ROUND_IDENT.to_string(),
            ConsensusManagerNextRoundInput {
                round: Round::of(next_round),
                proposer_timestamp_ms: 100,
                leader_proposal_history: LeaderProposalHistory {
                    gap_round_leaders: (1..next_round).map(|_| 0).collect(),
                    current_leader: 0,
                    is_fallback: false,
                },
            },
        )
        .build();

    let (_, preview_intent) = prepare_matching_test_tx_and_preview_intent(
        &mut ledger,
        &network,
        manifest,
        &preview_flags,
    );

    // Act
    let result = ledger.preview(preview_intent, &network);

    // Assert
    result
        .unwrap()
        .expect_commit_success()
        .state_updates
        .by_node
        .contains_key(CONSENSUS_MANAGER.as_node_id());
}

#[test]
fn notary_key_is_in_initial_proofs_when_notary_as_signatory_is_true() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_account(true);
    let current_epoch = ledger.get_current_epoch();

    // Act
    let receipt = ledger.preview(
        PreviewIntentV1 {
            intent: IntentV1 {
                header: TransactionHeaderV1 {
                    network_id: 0xf2,
                    start_epoch_inclusive: current_epoch,
                    end_epoch_exclusive: current_epoch.after(10).unwrap(),
                    nonce: 10,
                    notary_public_key: public_key.into(),
                    notary_is_signatory: true,
                    tip_percentage: 0,
                },
                instructions: InstructionsV1::from(
                    ManifestBuilder::new()
                        .lock_fee_and_withdraw(account, 10, XRD, 10)
                        .deposit_entire_worktop(account)
                        .build()
                        .instructions,
                ),
                blobs: Default::default(),
                message: Default::default(),
            },
            signer_public_keys: Default::default(),
            flags: PreviewFlags {
                use_free_credit: false,
                assume_all_signature_proofs: false,
                skip_epoch_check: false,
                disable_auth: false,
            },
        },
        &NetworkDefinition::simulator(),
    );

    // Assert
    receipt.expect("Must succeed!").expect_commit_success();
}

#[test]
fn notary_key_is_not_in_initial_proofs_when_notary_as_signatory_is_false() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_account(true);
    let current_epoch = ledger.get_current_epoch();

    // Act
    let receipt = ledger.preview(
        PreviewIntentV1 {
            intent: IntentV1 {
                header: TransactionHeaderV1 {
                    network_id: 0xf2,
                    start_epoch_inclusive: current_epoch,
                    end_epoch_exclusive: current_epoch.after(10).unwrap(),
                    nonce: 10,
                    notary_public_key: public_key.into(),
                    notary_is_signatory: false,
                    tip_percentage: 0,
                },
                instructions: InstructionsV1::from(
                    ManifestBuilder::new()
                        .lock_fee_and_withdraw(account, 10, XRD, 10)
                        .deposit_entire_worktop(account)
                        .build()
                        .instructions,
                ),
                blobs: Default::default(),
                message: Default::default(),
            },
            signer_public_keys: Default::default(),
            flags: PreviewFlags {
                use_free_credit: false,
                assume_all_signature_proofs: false,
                skip_epoch_check: false,
                disable_auth: false,
            },
        },
        &NetworkDefinition::simulator(),
    );

    // Assert
    receipt
        .expect("Must succeed!")
        .expect_specific_rejection(|error| {
            matches!(
                error,
                RejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(
                    RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                        AuthError::Unauthorized(..)
                    ))
                )
            )
        });
}

fn prepare_matching_test_tx_and_preview_intent(
    ledger: &mut DefaultLedgerSimulator,
    network: &NetworkDefinition,
    manifest: TransactionManifestV1,
    flags: &PreviewFlags,
) -> (NotarizedTransactionV1, PreviewIntentV1) {
    let notary_priv_key = Secp256k1PrivateKey::from_u64(2).unwrap();
    let tx_signer_priv_key = Secp256k1PrivateKey::from_u64(3).unwrap();

    let notarized_transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: network.id,
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(99),
            nonce: ledger.next_transaction_nonce(),
            notary_public_key: notary_priv_key.public_key().into(),
            notary_is_signatory: false,
            tip_percentage: 0,
        })
        .manifest(manifest)
        .sign(&tx_signer_priv_key)
        .notarize(&notary_priv_key)
        .build();

    let preview_intent = PreviewIntentV1 {
        intent: notarized_transaction.signed_intent.intent.clone(),
        signer_public_keys: vec![tx_signer_priv_key.public_key().into()],
        flags: flags.clone(),
    };

    (notarized_transaction, preview_intent)
}
