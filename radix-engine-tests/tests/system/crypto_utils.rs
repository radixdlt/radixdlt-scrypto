use radix_common::prelude::*;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::vm::NoExtension;
use radix_engine_tests::common::*;
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_transactions::builder::ManifestBuilder;
use scrypto_test::prelude::*;

macro_rules! get_output {
    ($func:ident($($args:tt)*)) => {
        $func($($args)*)
            .expect_commit_success()
            .output(1)
    };
}

macro_rules! get_failure {
    ($func:ident($($args:tt)*)) => {
        $func($($args)*)
            .expect_commit_failure()
            .outcome
            .expect_failure()
            .to_string(NO_NETWORK)
    };
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

fn crypto_scrypto_bls12381_v1_verify(
    runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    msg: Vec<u8>,
    pub_key: Bls12381G1PublicKey,
    signature: Bls12381G2Signature,
) -> TransactionReceipt {
    runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                "bls12381_v1_verify",
                manifest_args!(msg, pub_key, signature),
            )
            .build(),
        vec![],
    )
}

fn crypto_scrypto_bls12381_v1_aggregate_verify(
    runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    msgs: Vec<Vec<u8>>,
    pub_keys: Vec<Bls12381G1PublicKey>,
    signature: Bls12381G2Signature,
) -> TransactionReceipt {
    let pub_keys_msgs: Vec<(Bls12381G1PublicKey, Vec<u8>)> = pub_keys
        .iter()
        .zip(msgs)
        .map(|(pk, sk)| (*pk, sk))
        .collect();

    runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                "bls12381_v1_aggregate_verify",
                manifest_args!(pub_keys_msgs, &signature),
            )
            .build(),
        vec![],
    )
}

fn crypto_scrypto_bls12381_v1_fast_aggregate_verify(
    runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    msg: Vec<u8>,
    pub_keys: Vec<Bls12381G1PublicKey>,
    signature: Bls12381G2Signature,
) -> TransactionReceipt {
    runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                "bls12381_v1_fast_aggregate_verify",
                manifest_args!(msg, &pub_keys, &signature),
            )
            .build(),
        vec![],
    )
}

fn crypto_scrypto_bls12381_g2_signature_aggregate(
    runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    signatures: Vec<Bls12381G2Signature>,
) -> TransactionReceipt {
    runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                "bls12381_g2_signature_aggregate",
                manifest_args!(&signatures),
            )
            .build(),
        vec![],
    )
}

fn crypto_scrypto_keccak256_hash(
    runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    data: Vec<u8>,
) -> TransactionReceipt {
    runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                "keccak256_hash",
                manifest_args!(data),
            )
            .build(),
        vec![],
    )
}

fn crypto_scrypto_blake_2b_256_hash(
    runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    data: Vec<u8>,
) -> TransactionReceipt {
    runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                "blake2b_256_hash",
                manifest_args!(data),
            )
            .build(),
        vec![],
    )
}

#[test]
fn test_crypto_scrypto_verify_bls12381_v1() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

    let msg1 = hash("Test").to_vec();
    let pk = "93b1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2";
    let msg1_signature = "8b84ff5a1d4f8095ab8a80518ac99230ed24a7d1ec90c4105f9c719aa7137ed5d7ce1454d4a953f5f55f3959ab416f3014f4cd2c361e4d32c6b4704a70b0e2e652a908f501acb54ec4e79540be010e3fdc1fbf8e7af61625705e185a71c884f1";

    let pk = Bls12381G1PublicKey::from_str(pk).unwrap();
    let msg1_signature = Bls12381G2Signature::from_str(msg1_signature).unwrap();
    // Act
    let msg1_verify: bool = get_output!(crypto_scrypto_bls12381_v1_verify(
        &mut ledger,
        package_address,
        msg1,
        pk,
        msg1_signature,
    ));

    // Assert
    assert!(msg1_verify);

    // Arrange
    let msg2 = hash("ExpectFailureTest").to_vec();

    // Act
    let msg2_verify: bool = get_output!(crypto_scrypto_bls12381_v1_verify(
        &mut ledger,
        package_address,
        msg2,
        pk,
        msg1_signature,
    ));

    // Assert
    assert!(!msg2_verify);
}

#[test]
fn test_crypto_scrypto_bls12381_g2_signature_aggregate() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

    let (_sks, _pks, _msgs, sigs) = get_aggregate_verify_test_data(10, 10);

    // Aggregate the signature
    let agg_sig_multiple_msgs = Bls12381G2Signature::aggregate(&sigs, true).unwrap();

    // Act
    let agg_sig_from_scrypto =
        crypto_scrypto_bls12381_g2_signature_aggregate(&mut ledger, package_address, sigs)
            .expect_commit_success()
            .output(1);

    // Assert
    assert_eq!(agg_sig_multiple_msgs, agg_sig_from_scrypto);

    // Attempt to aggregate signature from empty input
    let receipt =
        crypto_scrypto_bls12381_g2_signature_aggregate(&mut ledger, package_address, vec![]);

    // Assert
    receipt.expect_commit_failure_containing_error("InputDataEmpty");
}

#[test]
fn test_crypto_scrypto_bls12381_aggregate_verify() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

    let (_sks, pks, msgs, sigs) = get_aggregate_verify_test_data(10, 10);

    // Aggregate the signature
    let agg_sig_multiple_msgs = Bls12381G2Signature::aggregate(&sigs, true).unwrap();

    // Act
    let agg_sig_from_scrypto = get_output!(crypto_scrypto_bls12381_g2_signature_aggregate(
        &mut ledger,
        package_address,
        sigs
    ));

    // Assert
    assert_eq!(agg_sig_multiple_msgs, agg_sig_from_scrypto);

    // Act
    let agg_verify: bool = get_output!(crypto_scrypto_bls12381_v1_aggregate_verify(
        &mut ledger,
        package_address,
        msgs.clone(),
        pks.clone(),
        agg_sig_multiple_msgs,
    ));

    // Assert
    assert!(agg_verify);

    // Arrange
    let mut pks_rev = pks.clone();
    pks_rev.reverse();

    // Act
    // Attempt to verify with reversed public keys order
    let agg_verify_expect_false: bool = get_output!(crypto_scrypto_bls12381_v1_aggregate_verify(
        &mut ledger,
        package_address,
        msgs.clone(),
        pks_rev,
        agg_sig_multiple_msgs,
    ));

    // Attempt to verify signature of empty message vector
    let empty_message_error = get_failure!(crypto_scrypto_bls12381_v1_aggregate_verify(
        &mut ledger,
        package_address,
        vec![],
        pks,
        agg_sig_multiple_msgs,
    ));

    // Attempt to verify signature using empty keys vector
    let empty_keys_error = get_failure!(crypto_scrypto_bls12381_v1_aggregate_verify(
        &mut ledger,
        package_address,
        msgs,
        vec![],
        agg_sig_multiple_msgs,
    ));

    // Assert
    assert!(!agg_verify_expect_false);
    assert!(empty_message_error.contains("InputDataEmpty"));
    assert!(empty_keys_error.contains("InputDataEmpty"));
}

#[test]
fn test_crypto_scrypto_bls12381_fast_aggregate_verify() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

    let sks: Vec<Bls12381G1PrivateKey> = (1..11)
        .map(|i| Bls12381G1PrivateKey::from_u64(i).unwrap())
        .collect();

    // Single message
    let msg = b"One message to sign for all".to_vec();

    let sigs: Vec<Bls12381G2Signature> = sks.iter().map(|sk| sk.sign_v1(&msg)).collect();

    let pks: Vec<Bls12381G1PublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    // Aggregate the signature
    let agg_sig_single_msg = Bls12381G2Signature::aggregate(&sigs, true).unwrap();

    // Act
    let agg_sig_from_scrypto: Bls12381G2Signature = get_output!(
        crypto_scrypto_bls12381_g2_signature_aggregate(&mut ledger, package_address, sigs)
    );

    // Assert
    assert_eq!(agg_sig_single_msg, agg_sig_from_scrypto);

    // Act
    let agg_verify: bool = get_output!(crypto_scrypto_bls12381_v1_fast_aggregate_verify(
        &mut ledger,
        package_address,
        msg.clone(),
        pks.clone(),
        agg_sig_single_msg,
    ));

    // Assert
    assert!(agg_verify);

    // Arrange
    let msg_false = b"Some other message".to_vec();

    // Act
    // Attempt to verify non-matching signature
    let agg_verify_expect_false: bool =
        get_output!(crypto_scrypto_bls12381_v1_fast_aggregate_verify(
            &mut ledger,
            package_address,
            msg_false,
            pks,
            agg_sig_single_msg,
        ));

    // Attempt to verify signature using empty keys vector
    let empty_keys_error = get_failure!(crypto_scrypto_bls12381_v1_fast_aggregate_verify(
        &mut ledger,
        package_address,
        msg,
        vec![],
        agg_sig_single_msg,
    ));

    // Assert
    assert!(!agg_verify_expect_false);
    assert!(empty_keys_error.contains("InputDataEmpty"));
}

#[test]
fn test_crypto_scrypto_keccak256_hash() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

    let data1 = b"Hello Radix".to_vec();
    let data2 = b"xidaR olleH".to_vec();
    let data3: Vec<u8> = vec![]; // empty data

    // Act
    let data1_hash: Hash = get_output!(crypto_scrypto_keccak256_hash(
        &mut ledger,
        package_address,
        data1
    ));
    // Assert
    assert_eq!(
        data1_hash,
        Hash::from_str("415942230ddb029416a4612818536de230d827cbac9646a0b26d9855a4c45587").unwrap()
    );

    // Act
    let data2_hash: Hash = get_output!(crypto_scrypto_keccak256_hash(
        &mut ledger,
        package_address,
        data2
    ));
    // Assert
    assert_ne!(
        data2_hash,
        Hash::from_str("415942230ddb029416a4612818536de230d827cbac9646a0b26d9855a4c45587").unwrap()
    );

    // Act
    let data3_hash: Hash = get_output!(crypto_scrypto_keccak256_hash(
        &mut ledger,
        package_address,
        data3
    ));
    // Assert
    assert_eq!(
        data3_hash,
        Hash::from_str("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap()
    );
}

#[test]
fn test_crypto_scrypto_flow() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

    let msg = b"Important message".to_vec();

    // Act
    // Get the hash of the message using CryptoScrypto package
    let msg_hash: Vec<u8> = {
        let hash: Hash = get_output!(crypto_scrypto_keccak256_hash(
            &mut ledger,
            package_address,
            msg
        ));
        hash
    }
    .as_bytes()
    .to_vec();

    let secret_key = Bls12381G1PrivateKey::from_u64(1).unwrap();
    let public_key = secret_key.public_key();

    // Sign the message hash using BLS
    let msg_signature = secret_key.sign_v1(msg_hash.as_slice());

    // Verify the BLS signature using CryptoScrypto package
    let result: bool = get_output!(crypto_scrypto_bls12381_v1_verify(
        &mut ledger,
        package_address,
        msg_hash,
        public_key,
        msg_signature,
    ));

    // Assert
    assert!(result);
}

#[test]
fn test_crypto_scrypto_blake2b_256_hash() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v2"));

    let data1 = b"Hello Radix".to_vec();
    let data2 = b"xidaR olleH".to_vec();
    let data3: Vec<u8> = vec![]; // empty data

    // Act
    let data1_hash: Hash = get_output!(crypto_scrypto_blake_2b_256_hash(
        &mut ledger,
        package_address,
        data1.clone()
    ));
    // Assert
    assert_eq!(
        data1_hash,
        Hash::from_str("48f1bd08444b5e713db9e14caac2faae71836786ac94d645b00679728202a935").unwrap()
    );
    assert_eq!(data1_hash, blake2b_256_hash(data1));

    // Act
    let data2_hash: Hash = get_output!(crypto_scrypto_blake_2b_256_hash(
        &mut ledger,
        package_address,
        data2
    ));
    // Assert
    assert_ne!(
        data2_hash,
        Hash::from_str("48f1bd08444b5e713db9e14caac2faae71836786ac94d645b00679728202a935").unwrap()
    );

    // Act
    let data3_hash: Hash = get_output!(crypto_scrypto_blake_2b_256_hash(
        &mut ledger,
        package_address,
        data3
    ));
    // Assert
    assert_eq!(
        data3_hash,
        Hash::from_str("0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8").unwrap()
    );
}

fn crypto_scrypto_ed25519_verify(
    runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    msg: Vec<u8>,
    pub_key: Ed25519PublicKey,
    signature: Ed25519Signature,
) -> TransactionReceipt {
    runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                "ed25519_verify",
                manifest_args!(msg, pub_key, signature),
            )
            .build(),
        vec![],
    )
}

fn crypto_scrypto_secp256k1_ecdsa_verify(
    runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    hash: Hash,
    pub_key: Secp256k1PublicKey,
    signature: Secp256k1Signature,
) -> TransactionReceipt {
    runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                "secp256k1_ecdsa_verify",
                manifest_args!(hash, pub_key, signature),
            )
            .build(),
        vec![],
    )
}

fn crypto_scrypto_secp256k1_ecdsa_verify_and_key_recover(
    runner: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    hash: Hash,
    signature: Secp256k1Signature,
    compressed: bool,
) -> TransactionReceipt {
    runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                if compressed {
                    "secp256k1_ecdsa_verify_and_key_recover"
                } else {
                    "secp256k1_ecdsa_verify_and_key_recover_uncompressed"
                },
                manifest_args!(hash, signature),
            )
            .build(),
        vec![],
    )
}

#[test]
fn test_crypto_scrypto_verify_ed25519() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v2"));

    let msg1 = hash("Test").to_vec();
    let pk = "4cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29";
    let msg1_signature = "cf0ca64435609b85ab170da339d415bbac87d678dfd505969be20adc6b5971f4ee4b4620c602bcbc34fd347596546675099d696265f4a42a16df343da1af980e";

    let pk = Ed25519PublicKey::from_str(pk).unwrap();
    let msg1_signature = Ed25519Signature::from_str(msg1_signature).unwrap();
    // Act
    let msg1_verify: bool = get_output!(crypto_scrypto_ed25519_verify(
        &mut ledger,
        package_address,
        msg1,
        pk,
        msg1_signature,
    ));

    // Assert
    assert!(msg1_verify);

    // Arrange
    let msg2 = hash("ExpectFailureTest").to_vec();

    // Act
    let msg2_verify: bool = get_output!(crypto_scrypto_ed25519_verify(
        &mut ledger,
        package_address,
        msg2,
        pk,
        msg1_signature,
    ));

    // Assert
    assert!(!msg2_verify);
}

#[test]
fn test_crypto_scrypto_verify_secp256k1_ecdsa() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v2"));

    let hash1 = hash("Test");
    let pk = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
    let hash1_signature = "00eb8dcd5bb841430dd0a6f45565a1b8bdb4a204eb868832cd006f963a89a662813ab844a542fcdbfda4086a83fbbde516214113051b9c8e42a206c98d564d7122";

    let pk = Secp256k1PublicKey::from_str(pk).unwrap();
    let hash1_signature = Secp256k1Signature::from_str(hash1_signature).unwrap();
    // Act
    let msg1_verify: bool = get_output!(crypto_scrypto_secp256k1_ecdsa_verify(
        &mut ledger,
        package_address,
        hash1,
        pk,
        hash1_signature,
    ));

    // Assert
    assert!(msg1_verify);

    // Arrange
    let hash2 = hash("ExpectFailureTest");

    // Act
    let msg2_verify: bool = get_output!(crypto_scrypto_secp256k1_ecdsa_verify(
        &mut ledger,
        package_address,
        hash2,
        pk,
        hash1_signature,
    ));

    // Assert
    assert!(!msg2_verify);
}

#[test]
fn test_crypto_scrypto_key_recover_secp256k1_ecdsa() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v2"));

    let hash1 = hash("Test");
    let pk = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
    let hash1_signature = "00eb8dcd5bb841430dd0a6f45565a1b8bdb4a204eb868832cd006f963a89a662813ab844a542fcdbfda4086a83fbbde516214113051b9c8e42a206c98d564d7122";

    let pk = Secp256k1PublicKey::from_str(pk).unwrap();
    let hash1_signature = Secp256k1Signature::from_str(hash1_signature).unwrap();

    // Act
    let pk_recovered1: Secp256k1PublicKey =
        get_output!(crypto_scrypto_secp256k1_ecdsa_verify_and_key_recover(
            &mut ledger,
            package_address,
            hash1,
            hash1_signature,
            true
        ));
    let pk_recovered2: [u8; 65] =
        get_output!(crypto_scrypto_secp256k1_ecdsa_verify_and_key_recover(
            &mut ledger,
            package_address,
            hash1,
            hash1_signature,
            false
        ));

    // Assert
    assert_eq!(pk, pk_recovered1);
    assert_eq!(
        secp256k1::PublicKey::from_slice(pk.as_ref())
            .unwrap()
            .serialize_uncompressed(),
        pk_recovered2
    );

    // Test for key recovery error
    let invalid_signature = "01cd8dcd5bb841430dd0a6f45565a1b8bdb4a204eb868832cd006f963a89a662813ab844a542fcdbfda4086a83fbbde516214113051b9c8e42a206c98d564d7122";
    let invalid_signature = Secp256k1Signature::from_str(invalid_signature).unwrap();

    let key_recovery_error = get_failure!(crypto_scrypto_secp256k1_ecdsa_verify_and_key_recover(
        &mut ledger,
        package_address,
        hash1,
        invalid_signature,
        true
    ));

    // Assert
    assert!(key_recovery_error.contains("Secp256k1KeyRecoveryError"));
}

fn bls12381_invalid_signature_aggregate<F>(protocol: ProtocolVersion, assert_receipt: F)
where
    F: Fn(TransactionReceiptV1),
{
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.from_bootstrap_to(protocol))
        .build();

    let package_address = ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

    let sig_not_in_group = "8b84ff5a1d4f8095ab8a80518ac99230ed24a7d1ec90c4105f9c719aa7137ed5d7ce1454d4a953f5f55f3959ab416f3014f4cd2c361e4d32c6b4704a70b0e2e652a908f501acb54ec4e79540be010e3fdc1fbf8e7af61625705e185a71c884f0";
    let sig_not_in_group = Bls12381G2Signature::from_str(sig_not_in_group).unwrap();
    let sig_valid = "82131f69b6699755f830e29d6ed41cbf759591a2ab598aa4e9686113341118d1db900d190436048601791121b5757c341045d4d0c94a95ec31a9ba6205f9b7504de85dadff52874375c58eec6cec28397279de87d5595101e398d31646d345bb";
    let sig_valid = Bls12381G2Signature::from_str(sig_valid).unwrap();

    let sigs_with_invalid_first = vec![sig_not_in_group, sig_valid];

    // Act
    let receipt = crypto_scrypto_bls12381_g2_signature_aggregate(
        &mut ledger,
        package_address,
        sigs_with_invalid_first,
    );

    // Assert
    assert_receipt(receipt);

    let sigs_with_valid_not_first = vec![sig_valid, sig_not_in_group];

    // Act
    let receipt = crypto_scrypto_bls12381_g2_signature_aggregate(
        &mut ledger,
        package_address,
        sigs_with_valid_not_first,
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::BlsError(
               msg
            )) if msg == "BlsError(\"BLST_POINT_NOT_IN_GROUP\")"
        )
    });
}

#[test]
fn bls12381_invalid_signature_aggregate_bottlenose() {
    bls12381_invalid_signature_aggregate(ProtocolVersion::Bottlenose, |receipt| {
        receipt.expect_commit_success();
    });
}

#[test]
fn bls12381_invalid_signature_aggregate_cuttlefish() {
    bls12381_invalid_signature_aggregate(ProtocolVersion::Cuttlefish, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::BlsError(
                   msg
                )) if msg == "BlsError(\"BLST_POINT_NOT_IN_GROUP\")"
            )
        });
    });
}

// Tests in this submodule are used to estimate costs units for Crypto Utils methods
#[cfg(feature = "resource_tracker")]
mod costing_tests {
    use super::*;

    #[test]
    fn test_crypto_scrypto_keccak256_costing() {
        let mut ledger = LedgerSimulatorBuilder::new().build();

        let package_address =
            ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

        for size in [
            100usize,
            200,
            500,
            1024,
            10 * 1024,
            20 * 1024,
            50 * 1024,
            100 * 1024,
            200 * 1024,
            500 * 1024,
            900 * 1024,
        ] {
            let data = vec![0u8; size];
            let _hash = crypto_scrypto_keccak256_hash(&mut ledger, package_address, data);
        }
    }

    #[test]
    fn test_crypto_scrypto_verify_bls12381_v1_costing() {
        let mut ledger = LedgerSimulatorBuilder::new().build();

        let package_address =
            ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

        let secret_key = Bls12381G1PrivateKey::from_u64(1).unwrap();
        let public_key = secret_key.public_key();

        for size in [
            100usize,
            200,
            500,
            1024,
            10 * 1024,
            20 * 1024,
            50 * 1024,
            100 * 1024,
            200 * 1024,
            500 * 1024,
            900 * 1024,
        ] {
            let data = vec![0u8; size];
            let signature = secret_key.sign_v1(data.as_slice());
            let _ = crypto_scrypto_bls12381_v1_verify(
                &mut ledger,
                package_address,
                data,
                public_key,
                signature,
            );
        }
    }

    #[test]
    fn test_crypto_scrypto_bls12381_g2_signature_aggregate_costing() {
        let mut ledger = LedgerSimulatorBuilder::new().build();

        let package_address =
            ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

        for cnt in [1, 2, 5, 10, 20, 50, 100] {
            let sks: Vec<Bls12381G1PrivateKey> = (1..(cnt + 1))
                .map(|i| Bls12381G1PrivateKey::from_u64(i).unwrap())
                .collect();

            // Single message
            let msg = b"One message to sign for all".to_vec();

            let sigs: Vec<Bls12381G2Signature> = sks.iter().map(|sk| sk.sign_v1(&msg)).collect();

            // Act
            let _ =
                crypto_scrypto_bls12381_g2_signature_aggregate(&mut ledger, package_address, sigs);
        }
    }

    #[test]
    fn test_crypto_scrypto_bls12381_v1_aggregate_verify_costing() {
        let mut ledger = LedgerSimulatorBuilder::new().build();

        let package_address =
            ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

        for msg_size in [100usize, 200, 500, 1024, 10 * 1024, 20 * 1024] {
            for cnt in [1u32, 2, 5, 10, 20] {
                let (_sks, pks, msgs, sigs) = get_aggregate_verify_test_data(cnt, msg_size);

                let agg_sig_multiple_msgs = Bls12381G2Signature::aggregate(&sigs, true).unwrap();

                let _ = crypto_scrypto_bls12381_v1_aggregate_verify(
                    &mut ledger,
                    package_address,
                    msgs,
                    pks,
                    agg_sig_multiple_msgs,
                );
            }
        }
    }

    #[test]
    fn test_crypto_scrypto_bls12381_v1_aggregate_verify_costing_2() {
        let mut ledger = LedgerSimulatorBuilder::new().build();

        let package_address =
            ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

        for (cnt, msg_size) in [
            (1, 100 * 1024),
            (2, 50 * 1024),
            (5, 20 * 1024),
            (10, 10 * 1024),
            (100, 1024),
            (1024, 100),
        ] {
            let (_sks, pks, msgs, sigs) = get_aggregate_verify_test_data(cnt, msg_size);
            let agg_sig = Bls12381G2Signature::aggregate(&sigs, true).unwrap();

            let _ = crypto_scrypto_bls12381_v1_aggregate_verify(
                &mut ledger,
                package_address,
                msgs,
                pks,
                agg_sig,
            );
        }

        // 1x 99kB and 1000x1B
        let (mut sks1, mut pks1, mut msgs1, mut sigs1) =
            get_aggregate_verify_test_data(1, 99 * 1024);
        let (mut sks2, mut pks2, mut msgs2, mut sigs2) = get_aggregate_verify_test_data(1000, 1);
        sks1.append(&mut sks2);
        pks1.append(&mut pks2);
        msgs1.append(&mut msgs2);
        sigs1.append(&mut sigs2);
        let agg_sig = Bls12381G2Signature::aggregate(&sigs1, true).unwrap();

        let _ = crypto_scrypto_bls12381_v1_aggregate_verify(
            &mut ledger,
            package_address,
            msgs1,
            pks1,
            agg_sig,
        );

        // 1x 90kB and 10 x 1kB
        let (mut sks1, mut pks1, mut msgs1, mut sigs1) =
            get_aggregate_verify_test_data(1, 90 * 1024);
        let (mut sks2, mut pks2, mut msgs2, mut sigs2) = get_aggregate_verify_test_data(10, 1024);
        sks1.append(&mut sks2);
        pks1.append(&mut pks2);
        msgs1.append(&mut msgs2);
        sigs1.append(&mut sigs2);
        let agg_sig = Bls12381G2Signature::aggregate(&sigs1, true).unwrap();

        let _ = crypto_scrypto_bls12381_v1_aggregate_verify(
            &mut ledger,
            package_address,
            msgs1,
            pks1,
            agg_sig,
        );
    }

    #[test]
    fn test_crypto_scrypto_bls12381_v1_fast_aggregate_verify_costing() {
        let mut ledger = LedgerSimulatorBuilder::new().build();

        let package_address =
            ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v1"));

        for msg_size in [100usize, 200, 500, 1024, 10 * 1024, 20 * 1024] {
            for cnt in [1u8, 2, 5, 10, 20, 50, 100] {
                let sks: Vec<Bls12381G1PrivateKey> = (1..(cnt + 1))
                    .map(|i| Bls12381G1PrivateKey::from_u64(i.into()).unwrap())
                    .collect();

                // Single message
                let msg: Vec<u8> = vec![cnt; msg_size];

                let sigs: Vec<Bls12381G2Signature> =
                    sks.iter().map(|sk| sk.sign_v1(&msg)).collect();

                let pks: Vec<Bls12381G1PublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

                let agg_sig_single_msg = Bls12381G2Signature::aggregate(&sigs, true).unwrap();

                let _ = crypto_scrypto_bls12381_v1_fast_aggregate_verify(
                    &mut ledger,
                    package_address,
                    msg,
                    pks,
                    agg_sig_single_msg,
                );
            }
        }
    }

    #[test]
    fn test_crypto_scrypto_blake2b_256_costing() {
        let mut ledger = LedgerSimulatorBuilder::new().build();

        let package_address =
            ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v2"));

        for size in [
            100usize,
            200,
            500,
            1024,
            10 * 1024,
            20 * 1024,
            50 * 1024,
            100 * 1024,
            200 * 1024,
            500 * 1024,
            900 * 1024,
        ] {
            let data = vec![0u8; size];
            let _hash = crypto_scrypto_blake_2b_256_hash(&mut ledger, package_address, data);
        }
    }

    #[test]
    fn test_crypto_scrypto_verify_ed25519_costing() {
        let mut ledger = LedgerSimulatorBuilder::new().build();

        let package_address =
            ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v2"));

        let secret_key = Ed25519PrivateKey::from_u64(1).unwrap();
        let public_key = secret_key.public_key();

        for size in [
            100usize,
            200,
            500,
            1024,
            10 * 1024,
            20 * 1024,
            50 * 1024,
            100 * 1024,
            200 * 1024,
            500 * 1024,
            900 * 1024,
        ] {
            let data = vec![0u8; size];
            let signature = secret_key.sign(&data);
            let _ = crypto_scrypto_ed25519_verify(
                &mut ledger,
                package_address,
                data,
                public_key,
                signature,
            );
        }
    }

    #[test]
    fn test_crypto_scrypto_verify_secp256k1_ecdsa_costing() {
        let mut ledger = LedgerSimulatorBuilder::new().build();

        let package_address =
            ledger.publish_package_simple(PackageLoader::get("crypto_scrypto_v2"));

        let secret_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let public_key = secret_key.public_key();

        for size in 0..10 {
            let data: Vec<u8> = vec![size as u8; size * 10];
            let data_hash = hash(data);
            let signature = secret_key.sign(&data_hash);
            let _ = crypto_scrypto_secp256k1_ecdsa_verify(
                &mut ledger,
                package_address,
                data_hash,
                public_key,
                signature,
            );
        }
    }
}
