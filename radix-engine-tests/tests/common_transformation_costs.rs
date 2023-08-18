use radix_engine::transaction::CostingParameters;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::types::*;
use radix_engine_interface::blueprints::access_controller::ACCESS_CONTROLLER_CREATE_PROOF_IDENT;
use scrypto_unit::*;
use transaction::prelude::*;
use transaction::validation::NotarizedTransactionValidator;
use transaction::validation::{TransactionValidator, ValidationConfig};

// We run tests in this file to produce common manifest transformation costs for Core Apps, such as
// - Adding a lock_fee instruction, with account protected by single signature/badge, whichever is worse
// - Adding an amount assertion, for fungible/non-fungible, whichever is worse
// - Adding a secp256k1 or ed25519 signature, whichever is worse

#[test]
fn estimate_locking_fee_from_an_account_protected_by_signature() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let network = NetworkDefinition::simulator();
    let (_pk, sk, account) = test_runner.new_virtual_account();

    let manifest1 = ManifestBuilder::new().build();
    let tx1 = create_notarized_transaction(
        &mut test_runner,
        &network,
        manifest1,
        vec![], // no sign
        &sk,    // notarize
    );
    let receipt1 = test_runner.execute_transaction(
        validate_notarized_transaction(&network, &tx1).get_executable_with_free_credit(dec!(100)),
        CostingParameters::default(),
        ExecutionConfig::for_notarized_transaction(),
    );
    println!("\n{:?}", receipt1);

    let manifest2 = ManifestBuilder::new().lock_fee(account, dec!(100)).build();
    let tx2 = create_notarized_transaction(
        &mut test_runner,
        &network,
        manifest2,
        vec![&sk], // sign
        &sk,       // notarize
    );
    let receipt2 = test_runner.execute_transaction(
        validate_notarized_transaction(&network, &tx2).get_executable_with_free_credit(dec!(0)),
        CostingParameters::default(),
        ExecutionConfig::for_notarized_transaction(),
    );
    println!("\n{:?}", receipt2);

    println!(
        "Locking fee from an account protected by signature: {} XRD",
        receipt2
            .fee_summary
            .total_cost()
            .safe_sub(receipt1.fee_summary.total_cost())
            .unwrap()
    );
}

#[test]
fn estimate_locking_fee_from_an_account_protected_by_access_controller() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let network = NetworkDefinition::simulator();
    let (_pk1, sk1, _pk2, _sk2, _pk3, _sk3, _pk4, _sk4, account, access_controller) =
        test_runner.new_virtual_account_with_access_controller();

    let manifest1 = ManifestBuilder::new().build();
    let tx1 = create_notarized_transaction(
        &mut test_runner,
        &network,
        manifest1,
        vec![], // no sign
        &sk1,   // notarize
    );
    let receipt1 = test_runner.execute_transaction(
        validate_notarized_transaction(&network, &tx1).get_executable_with_free_credit(dec!(100)),
        CostingParameters::default(),
        ExecutionConfig::for_notarized_transaction(),
    );
    println!("\n{:?}", receipt1);

    let manifest2 = ManifestBuilder::new()
        .call_method(
            access_controller,
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT,
            manifest_args!(),
        )
        .lock_fee(account, dec!(100))
        .build();
    let tx2 = create_notarized_transaction(
        &mut test_runner,
        &network,
        manifest2,
        vec![&sk1], // sign
        &sk1,       // notarize
    );
    let receipt2 = test_runner.execute_transaction(
        validate_notarized_transaction(&network, &tx2).get_executable_with_free_credit(dec!(0)),
        CostingParameters::default(),
        ExecutionConfig::for_notarized_transaction(),
    );
    println!("\n{:?}", receipt2);

    println!(
        "Locking fee from an account protected by an access controller (1-4): {} XRD",
        receipt2
            .fee_summary
            .total_cost()
            .safe_sub(receipt1.fee_summary.total_cost())
            .unwrap()
    );
}

fn create_notarized_transaction(
    test_runner: &mut DefaultTestRunner,
    network: &NetworkDefinition,
    manifest: TransactionManifestV1,
    signers: Vec<&Secp256k1PrivateKey>,
    notary: &Secp256k1PrivateKey,
) -> NotarizedTransactionV1 {
    let notarized_transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: network.id,
            start_epoch_inclusive: Epoch::zero(),
            end_epoch_exclusive: Epoch::of(99),
            nonce: test_runner.next_transaction_nonce(),
            notary_public_key: notary.public_key().into(),
            notary_is_signatory: false,
            tip_percentage: DEFAULT_TIP_PERCENTAGE,
        })
        .manifest(manifest)
        .multi_sign(&signers)
        .notarize(notary)
        .build();
    notarized_transaction
}

fn validate_notarized_transaction<'a>(
    network: &'a NetworkDefinition,
    transaction: &'a NotarizedTransactionV1,
) -> ValidatedNotarizedTransactionV1 {
    NotarizedTransactionValidator::new(ValidationConfig::default(network.id))
        .validate(transaction.prepare().unwrap())
        .unwrap()
}
