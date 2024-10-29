#![cfg(feature = "std")]

use radix_common::data::scrypto::*;
use radix_common::prelude::*;
use radix_engine::updates::*;
use radix_transactions::manifest::decompile;
use radix_transactions::prelude::*;
use sbor::representations::SerializationParameters;
use scrypto_test::prelude::*;
use std::path::PathBuf;

#[test]
fn transaction_previews_do_no_contains_debug_information() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .deposit_entire_worktop(account)
            .build(),
        vec![pk.into()],
        0,
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: true,
        },
    );

    // Assert
    assert!(
        receipt.debug_information.is_none(),
        "Debug information is available in a preview receipt"
    );
}

#[test]
fn executing_transactions_with_debug_information_outputs_the_detailed_cost_breakdown() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.execute_manifest_with_execution_config(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .deposit_entire_worktop(account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pk)],
        ExecutionConfig::for_debug_transaction(),
    );

    // Assert
    assert!(
        receipt.debug_information.is_some(),
        "Debug information is not available when it should."
    );
}

#[test]
fn assert_flamegraph_of_faucet_lock_fee_method() {
    run_faucet_lock_fee(AlignerExecutionMode::Assert);
}

#[test]
#[ignore = "Run this test to update the generated files"]
fn update_flamegraph_of_faucet_lock_fee_method() {
    run_faucet_lock_fee(AlignerExecutionMode::Write);
}

fn run_faucet_lock_fee(mode: AlignerExecutionMode) {
    create_flamegraph_and_detailed_breakdown(
        "faucet-lock-fee",
        |_| {
            (
                ManifestBuilder::new().lock_fee_from_faucet().build(),
                Default::default(),
            )
        },
        mode,
    );
}

#[test]
fn assert_flamegraph_of_faucet_lock_fee_and_free_xrd_method() {
    run_faucet_lock_fee_and_free_xrd(AlignerExecutionMode::Assert);
}

#[test]
#[ignore = "Run this test to update the generated files"]
fn update_flamegraph_of_faucet_lock_fee_and_free_xrd_method() {
    run_faucet_lock_fee_and_free_xrd(AlignerExecutionMode::Write);
}

fn run_faucet_lock_fee_and_free_xrd(mode: AlignerExecutionMode) {
    create_flamegraph_and_detailed_breakdown(
        "faucet-lock-fee-and-free-xrd",
        |ledger| {
            let (pk, _, account) = ledger.new_account(false);
            (
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .get_free_xrd_from_faucet()
                    .deposit_entire_worktop(account)
                    .build(),
                vec![pk.into()],
            )
        },
        mode,
    );
}

fn create_flamegraph_and_detailed_breakdown<F>(title: &str, callback: F, mode: AlignerExecutionMode)
where
    F: FnOnce(&mut DefaultLedgerSimulator) -> (TransactionManifestV1, Vec<PublicKey>),
{
    let network_definition = NetworkDefinition::simulator();
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_cost_breakdown()
        .with_custom_protocol(|builder| {
            builder
                .configure_babylon(|_| BabylonSettings::test_minimal())
                .from_bootstrap_to(ProtocolVersion::Bottlenose)
        })
        .build();
    let (manifest, signers) = callback(&mut ledger);
    let string_manifest = decompile(&manifest, &network_definition).expect("Can't fail!");
    let receipt = ledger.execute_manifest_with_execution_config(
        manifest,
        signers
            .into_iter()
            .map(|pk| NonFungibleGlobalId::from_public_key(&pk)),
        ExecutionConfig::for_debug_transaction(),
    );
    receipt.expect_commit_success();
    let root_path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("flamegraphs");
    let mut folder =
        FolderContentAligner::new(root_path, mode, AlignerFolderMode::ExpectOtherContent);
    let svg_bytes = receipt
        .generate_execution_breakdown_flamegraph_svg_bytes(title, &network_definition)
        .expect("Must succeed");
    folder.put_file(format!("{}.svg", title), svg_bytes);
    let json_contents = to_json(
        &receipt
            .debug_information
            .as_ref()
            .unwrap()
            .detailed_execution_cost_breakdown,
        &network_definition,
    );
    folder.put_file(format!("{}.json", title), json_contents);
    folder.put_file(format!("{}.rtm", title), string_manifest);
}

pub fn to_json<S: ScryptoEncode + ScryptoDescribe>(
    value: &S,
    network_definition: &NetworkDefinition,
) -> String {
    let encoder = AddressBech32Encoder::new(network_definition);

    let (local_type_id, schema) = generate_full_schema_from_single_type::<S, ScryptoCustomSchema>();
    let schema = schema.fully_update_and_into_latest_version();

    let context = ScryptoValueDisplayContext::with_optional_bech32(Some(&encoder));
    let payload = scrypto_encode(&value).unwrap();
    let raw_payload = ScryptoRawPayload::new_from_valid_slice(&payload);
    let serializable = raw_payload.serializable(SerializationParameters::WithSchema {
        mode: representations::SerializationMode::Natural,
        custom_context: context,
        schema: &schema,
        type_id: local_type_id,
        depth_limit: SCRYPTO_SBOR_V1_MAX_DEPTH,
    });

    serde_json::to_string_pretty(&serializable).unwrap()
}
