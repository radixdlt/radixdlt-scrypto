use radix_common::prelude::*;
use scrypto_test::prelude::*;

#[test]
fn test_transaction_preview_cost_estimate() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().with_cost_breakdown().build();

    let flags = PreviewFlags {
        use_free_credit: false,
        assume_all_signature_proofs: false,
        skip_epoch_check: false,
        disable_auth: false,
    };
    let (notarized_transaction, preview_transaction) =
        prepare_complex_matching_transaction_and_preview_transaction(
            &mut ledger,
            TransactionBuildConfig {
                should_sign: true,
                should_lock_fee: true,
            },
        );
    let preparation_settings = PreparationSettings::latest_ref();
    let raw_notarized_transaction = notarized_transaction.to_raw().unwrap();
    let raw_preview_transaction = preview_transaction.to_raw().unwrap();
    let size_diff = raw_notarized_transaction
        .prepare(preparation_settings)
        .unwrap()
        .get_summary()
        .effective_length
        - PreparedPreviewTransactionV2::prepare(&raw_preview_transaction, preparation_settings)
            .unwrap()
            .transaction_intent
            .get_summary()
            .effective_length;

    // Act & Assert: Execute the preview, followed by a normal execution.
    // Ensure that both succeed and that the preview result provides an accurate cost estimate
    let preview_receipt = ledger.preview_v2(preview_transaction, flags);
    preview_receipt.expect_commit_success();
    let actual_receipt = ledger.execute_notarized_transaction(&raw_notarized_transaction);
    actual_receipt.expect_commit(true);

    println!(
        "{}",
        format_cost_breakdown(
            &preview_receipt.fee_summary,
            preview_receipt.fee_details.as_ref().unwrap()
        )
    );
    println!(
        "{}",
        format_cost_breakdown(
            &actual_receipt.fee_summary,
            actual_receipt.fee_details.as_ref().unwrap()
        )
    );

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
            .unwrap(),
        actual_receipt.fee_summary.total_cost(),
    );
}

#[test]
fn test_transaction_preview_without_locking_fee() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let flags = PreviewFlags {
        use_free_credit: true,
        assume_all_signature_proofs: false,
        skip_epoch_check: false,
        disable_auth: false,
    };

    let (_, preview) = prepare_complex_matching_transaction_and_preview_transaction(
        &mut ledger,
        TransactionBuildConfig {
            should_sign: true,
            should_lock_fee: false,
        },
    );

    // Act
    let receipt = ledger.preview_v2(preview, flags);
    let fee_summary = &receipt.fee_summary;
    println!("{:?}", receipt);
    assert!(fee_summary.total_execution_cost_in_xrd.is_positive());
    assert_eq!(fee_summary.total_tipping_cost_in_xrd, dec!("0"));
    assert!(fee_summary.total_storage_cost_in_xrd.is_positive()); // payload cost
    assert_eq!(fee_summary.total_royalty_cost_in_xrd, dec!("0"));
}

#[test]
fn test_assume_all_signature_proofs_flag_method_authorization() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let flags = PreviewFlags {
        use_free_credit: false,
        assume_all_signature_proofs: true,
        skip_epoch_check: false,
        disable_auth: false,
    };

    let (_, preview) = prepare_complex_matching_transaction_and_preview_transaction(
        &mut ledger,
        TransactionBuildConfig {
            should_sign: false,
            should_lock_fee: true,
        },
    );

    // Act
    let result = ledger.preview_v2(preview, flags);

    // Assert
    result.expect_commit_success();
}

#[test]
fn test_preview_no_auth() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let preview_flags = PreviewFlags {
        use_free_credit: true,
        assume_all_signature_proofs: false,
        skip_epoch_check: false,
        disable_auth: true,
    };

    // Everything is possible without auth, even ConsensusManager calls!
    let next_round = 99;
    let preview = ledger
        .v2_transaction_builder()
        .manifest(
            ManifestBuilder::new_v2()
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
                .build(),
        )
        .build_preview_transaction([]);

    // Act
    let result = ledger.preview_v2(preview, preview_flags);

    // Assert
    result
        .expect_commit_success()
        .state_updates
        .by_node
        .contains_key(CONSENSUS_MANAGER.as_node_id());
}

struct TransactionBuildConfig {
    should_sign: bool,
    should_lock_fee: bool,
}

fn prepare_complex_matching_transaction_and_preview_transaction(
    ledger: &mut DefaultLedgerSimulator,
    TransactionBuildConfig {
        should_sign,
        should_lock_fee,
    }: TransactionBuildConfig,
) -> (NotarizedTransactionV2, PreviewTransactionV2) {
    let network = NetworkDefinition::simulator();
    let (_, subintent_account_key, subintent_account_address) = ledger.new_account(true);
    let (_, notary_key, notary_account_address) = ledger.new_account(true);
    let (_, transaction_account_key, transaction_account_address) = ledger.new_account(true);

    let subintent = TransactionBuilder::new_partial_v2()
        .intent_header(IntentHeaderV2 {
            network_id: network.id,
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(99),
            intent_discriminator: ledger.next_transaction_nonce() as u64,
            min_proposer_timestamp_inclusive: None,
            max_proposer_timestamp_exclusive: None,
        })
        .manifest_builder(|builder| {
            builder
                .withdraw_from_account(subintent_account_address, XRD, 10)
                .take_all_from_worktop(XRD, "xrd")
                .yield_to_parent_with_name_lookup(|lookup| (lookup.bucket("xrd"),))
        })
        .then(|builder| {
            if should_sign {
                builder.sign(&subintent_account_key)
            } else {
                builder
            }
        })
        .build();

    let mut builder = TransactionBuilder::new_v2()
        .transaction_header(TransactionHeaderV2 {
            notary_public_key: notary_key.public_key().into(),
            notary_is_signatory: should_sign,
            tip_basis_points: 0,
        })
        .intent_header(IntentHeaderV2 {
            network_id: network.id,
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(99),
            intent_discriminator: ledger.next_transaction_nonce() as u64,
            min_proposer_timestamp_inclusive: None,
            max_proposer_timestamp_exclusive: None,
        })
        .add_signed_child("child", subintent)
        .manifest_builder(
            |builder| {
                builder
                    .then(|builder| {
                        if should_lock_fee {
                            builder.lock_standard_test_fee(notary_account_address)
                        } else {
                            builder
                        }
                    }) // Auth from notary is signer
                    .yield_to_child("child", ())
                    .deposit_entire_worktop(transaction_account_address)
            }, // Explicit signing required
        );

    let preview_transaction = builder.build_preview_transaction(if should_sign {
        vec![transaction_account_key.public_key().into()]
    } else {
        vec![]
    });

    let transaction = builder
        .sign(&transaction_account_key)
        .notarize(&notary_key)
        .build_minimal();

    (transaction, preview_transaction)
}
