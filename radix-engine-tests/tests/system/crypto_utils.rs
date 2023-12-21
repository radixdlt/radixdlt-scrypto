use radix_engine::types::*;
use radix_engine::vm::NoExtension;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use radix_engine_tests::common::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[cfg(test)]
fn crypto_scrypto_bls12381_v1_verify(
    runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    msg: Vec<u8>,
    pub_key: Bls12381G1PublicKey,
    signature: Bls12381G2Signature,
) -> bool {
    let receipt = runner.execute_manifest(
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
    );
    let result = receipt.expect_commit_success();
    result.output(1)
}

#[cfg(test)]
fn crypto_scrypto_bls12381_v1_aggregate_verify(
    runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    msgs: Vec<Vec<u8>>,
    pub_keys: Vec<Bls12381G1PublicKey>,
    signature: Bls12381G2Signature,
) -> bool {
    let receipt = runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                "bls12381_v1_aggregate_verify",
                manifest_args!(msgs, pub_keys, signature),
            )
            .build(),
        vec![],
    );
    let result = receipt.expect_commit_success();
    result.output(1)
}

#[cfg(test)]
fn crypto_scrypto_bls12381_v1_fast_aggregate_verify(
    runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    msg: Vec<u8>,
    pub_keys: Vec<Bls12381G1PublicKey>,
    signature: Bls12381G2Signature,
) -> bool {
    let receipt = runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                "bls12381_v1_fast_aggregate_verify",
                manifest_args!(msg, pub_keys, signature),
            )
            .build(),
        vec![],
    );
    let result = receipt.expect_commit_success();
    result.output(1)
}

#[cfg(test)]
fn crypto_scrypto_bls12381_g2_signature_aggregate(
    runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    signatures: Vec<Bls12381G2Signature>,
) -> Bls12381G2Signature {
    let receipt = runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(runner.faucet_component(), 500u32)
            .call_function(
                package_address,
                "CryptoScrypto",
                "bls12381_g2_signature_aggregate",
                manifest_args!(signatures),
            )
            .build(),
        vec![],
    );
    let result = receipt.expect_commit_success();
    result.output(1)
}

#[cfg(test)]
fn crypto_scrypto_keccak256_hash(
    runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    data: Vec<u8>,
) -> Hash {
    let receipt = runner.execute_manifest(
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
    );
    let result = receipt.expect_commit_success();
    result.output(1)
}

#[test]
fn test_crypto_scrypto_verify_bls12381_v1() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let msg1 = hash("Test").to_vec();
    let msg2 = hash("ExpectFailureTest").to_vec();
    let pk = "93b1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2";
    let msg1_signature = "8b84ff5a1d4f8095ab8a80518ac99230ed24a7d1ec90c4105f9c719aa7137ed5d7ce1454d4a953f5f55f3959ab416f3014f4cd2c361e4d32c6b4704a70b0e2e652a908f501acb54ec4e79540be010e3fdc1fbf8e7af61625705e185a71c884f1";

    let pk = Bls12381G1PublicKey::from_str(pk).unwrap();
    let msg1_signature = Bls12381G2Signature::from_str(msg1_signature).unwrap();
    // Act
    let msg1_verify = crypto_scrypto_bls12381_v1_verify(
        &mut test_runner,
        package_address,
        msg1,
        pk,
        msg1_signature,
    );
    let msg2_verify = crypto_scrypto_bls12381_v1_verify(
        &mut test_runner,
        package_address,
        msg2,
        pk,
        msg1_signature,
    );

    // Assert
    assert!(msg1_verify);
    assert!(!msg2_verify);
}

#[test]
fn test_crypto_scrypto_bls12381_aggregate_verify() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let sks: Vec<Bls12381G1PrivateKey> = (1..11)
        .map(|i| Bls12381G1PrivateKey::from_u64(i).unwrap())
        .collect();

    // Multiple messages
    let msgs: Vec<Vec<u8>> = (1u8..11).map(|i| vec![i; 10]).collect();

    let sigs: Vec<Bls12381G2Signature> = sks
        .iter()
        .zip(msgs.clone())
        .map(|(sk, msg)| sk.sign_v1(&msg))
        .collect();

    let pks: Vec<Bls12381G1PublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    // Aggregate the signature
    let agg_sig_multiple_msgs = Bls12381G2Signature::aggregate(&sigs).unwrap();

    // Act
    let agg_sig_from_scrypto =
        crypto_scrypto_bls12381_g2_signature_aggregate(&mut test_runner, package_address, sigs);
    let agg_verify = crypto_scrypto_bls12381_v1_aggregate_verify(
        &mut test_runner,
        package_address,
        msgs.clone(),
        pks.clone(),
        agg_sig_multiple_msgs,
    );

    let mut pks_rev = pks.clone();
    pks_rev.reverse();

    // Attempt to verify with reversed public keys order
    let agg_verify_expect_false = crypto_scrypto_bls12381_v1_aggregate_verify(
        &mut test_runner,
        package_address,
        msgs,
        pks_rev,
        agg_sig_multiple_msgs,
    );

    // Assert
    assert_eq!(agg_sig_multiple_msgs, agg_sig_from_scrypto);
    assert!(agg_verify);
    assert!(!agg_verify_expect_false);
}

#[test]
fn test_crypto_scrypto_bls12381_fast_aggregate_verify() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let sks: Vec<Bls12381G1PrivateKey> = (1..11)
        .map(|i| Bls12381G1PrivateKey::from_u64(i).unwrap())
        .collect();

    // Single message
    let msg = b"One message to sign for all".to_vec();

    let sigs: Vec<Bls12381G2Signature> = sks.iter().map(|sk| sk.sign_v1(&msg)).collect();

    let pks: Vec<Bls12381G1PublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

    // Aggregate the signature
    let agg_sig_single_msg = Bls12381G2Signature::aggregate(&sigs).unwrap();

    // Act
    let agg_sig_from_scrypto =
        crypto_scrypto_bls12381_g2_signature_aggregate(&mut test_runner, package_address, sigs);
    let agg_verify = crypto_scrypto_bls12381_v1_fast_aggregate_verify(
        &mut test_runner,
        package_address,
        msg,
        pks.clone(),
        agg_sig_single_msg,
    );

    let msg_false = b"Some other message".to_vec();

    // Attempt to verify non-matching signature
    let agg_verify_expect_false = crypto_scrypto_bls12381_v1_fast_aggregate_verify(
        &mut test_runner,
        package_address,
        msg_false,
        pks,
        agg_sig_single_msg,
    );

    // Assert
    assert_eq!(agg_sig_single_msg, agg_sig_from_scrypto);
    assert!(agg_verify);
    assert!(!agg_verify_expect_false);
}

#[test]
fn test_crypto_scrypto_keccak256_hash() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let data1 = b"Hello Radix".to_vec();
    let data2 = b"xidaR olleH".to_vec();

    // Act
    let data1_hash = crypto_scrypto_keccak256_hash(&mut test_runner, package_address, data1);
    let data2_hash = crypto_scrypto_keccak256_hash(&mut test_runner, package_address, data2);

    // Assert
    assert_eq!(
        data1_hash,
        Hash::from_str("415942230ddb029416a4612818536de230d827cbac9646a0b26d9855a4c45587").unwrap()
    );
    assert_ne!(
        data2_hash,
        Hash::from_str("415942230ddb029416a4612818536de230d827cbac9646a0b26d9855a4c45587").unwrap()
    );
}

#[test]
fn test_crypto_scrypto_flow() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let msg = b"Important message".to_vec();

    // Act
    // Get the hash of the message using CryptoScrypto package
    let msg_hash = crypto_scrypto_keccak256_hash(&mut test_runner, package_address, msg)
        .as_bytes()
        .to_vec();

    let secret_key = Bls12381G1PrivateKey::from_u64(1).unwrap();
    let public_key = secret_key.public_key();

    // Sign the message hash using BLS
    let msg_signature = secret_key.sign_v1(msg_hash.as_slice());

    // Verify the BLS signature using CryptoScrypto package
    let result = crypto_scrypto_bls12381_v1_verify(
        &mut test_runner,
        package_address,
        msg_hash,
        public_key,
        msg_signature,
    );

    // Assert
    assert!(result);
}
