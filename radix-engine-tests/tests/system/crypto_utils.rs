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
    msg: &str,
    pk: &str,
    sig: &str,
) -> bool {
    let msg = hex::decode(msg).unwrap();
    let pub_key = Bls12381G1PublicKey::from_str(pk).unwrap();
    let signature = Bls12381G2Signature::from_str(sig).unwrap();

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
fn crypto_scrypto_keccak256_hash(
    runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    data: &str,
) -> Hash {
    let data = data.as_bytes().to_vec();

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

    let msg1 = hash("Test").to_string();
    let msg2 = hash("ExpectFailureTest").to_string();
    let pk = "93b1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2";
    let msg1_signature = "8b84ff5a1d4f8095ab8a80518ac99230ed24a7d1ec90c4105f9c719aa7137ed5d7ce1454d4a953f5f55f3959ab416f3014f4cd2c361e4d32c6b4704a70b0e2e652a908f501acb54ec4e79540be010e3fdc1fbf8e7af61625705e185a71c884f1";

    // Act
    let msg1_verify = crypto_scrypto_bls12381_v1_verify(
        &mut test_runner,
        package_address,
        &msg1,
        pk,
        msg1_signature,
    );
    let msg2_verify = crypto_scrypto_bls12381_v1_verify(
        &mut test_runner,
        package_address,
        &msg2,
        pk,
        msg1_signature,
    );

    // Assert
    assert!(msg1_verify);
    assert!(!msg2_verify);
}

#[test]
fn test_crypto_scrypto_keccak256_hash() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let data1 = "Hello Radix";
    let data2 = "xidaR olleH";

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

    let msg = "Important message";

    // Act
    // Get the hash of the message using CryptoScrypto package
    let msg_hash = crypto_scrypto_keccak256_hash(&mut test_runner, package_address, msg);

    let secret_key = Bls12381G1PrivateKey::from_u64(1).unwrap();
    let public_key = secret_key.public_key();

    // Sign the message hash using BLS
    let msg_signature = secret_key.sign_v1(msg_hash.as_slice());

    // Verify the BLS signature using CryptoScrypto package
    let result = crypto_scrypto_bls12381_v1_verify(
        &mut test_runner,
        package_address,
        &msg_hash.to_string(),
        &public_key.to_string(),
        &msg_signature.to_string(),
    );

    // Assert
    assert!(result);
}

#[test]
fn test_crypto_keccak256_100b() {
    let data = "m".repeat(100);
    let _ = keccak256_hash(data);
}
#[test]
fn test_crypto_keccak256_1kb() {
    let data = "m".repeat(1024);
    let _ = keccak256_hash(data);
}
#[test]
fn test_keccak256_100kb() {
    let data = "m".repeat(100 * 1024);
    let _ = keccak256_hash(data);
}
#[test]
fn test_crypto_keccak256_1mb() {
    let data = "m".repeat(1024 * 1024);
    let _ = keccak256_hash(data);
}

#[test]
fn test_crypto_sign_and_verify_100b() {
    let test_sk = "408157791befddd702672dcfcfc99da3512f9c0ea818890fcb6ab749580ef2cf";
    let test_message = vec![0; 100];
    let sk = Bls12381G1PrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
    let pk = sk.public_key();

    let sig = sk.sign_v1(&test_message);
    let _ = verify_bls12381_v1(&test_message, &pk, &sig);
}
#[test]
fn test_crypto_sign_and_verify_1kb() {
    let test_sk = "408157791befddd702672dcfcfc99da3512f9c0ea818890fcb6ab749580ef2cf";
    let test_message = vec![0; 1024];
    let sk = Bls12381G1PrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
    let pk = sk.public_key();

    let sig = sk.sign_v1(&test_message);
    let _ = verify_bls12381_v1(&test_message, &pk, &sig);
}
#[test]
fn test_crypto_sign_and_verify_100kb() {
    let test_sk = "408157791befddd702672dcfcfc99da3512f9c0ea818890fcb6ab749580ef2cf";
    let test_message = vec![0; 100 * 1024];
    let sk = Bls12381G1PrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
    let pk = sk.public_key();

    let sig = sk.sign_v1(&test_message);
    let _ = verify_bls12381_v1(&test_message, &pk, &sig);
}
#[test]
fn test_crypto_sign_and_verify_1mb() {
    let test_sk = "408157791befddd702672dcfcfc99da3512f9c0ea818890fcb6ab749580ef2cf";
    let test_message = vec![0; 1024 * 1024];
    let sk = Bls12381G1PrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
    let pk = sk.public_key();

    let sig = sk.sign_v1(&test_message);
    let _ = verify_bls12381_v1(&test_message, &pk, &sig);
}

#[test]
fn test_crypto_scrypto_keccak256_costing() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    for size in [100usize, 1024, 100 * 1024, 900 * 1024] {
        let data = vec![0u8; size];
        let receipt = test_runner.execute_manifest(
            ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 500u32)
                .call_function(
                    package_address,
                    "CryptoScrypto",
                    "keccak256_hash",
                    manifest_args!(data),
                )
                .build(),
            vec![],
        );
        let _ = receipt.expect_commit_success();
    }
}

#[test]
fn test_crypto_scrypto_verify_bls12381_v1_costing() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let secret_key = Bls12381G1PrivateKey::from_u64(1).unwrap();
    let public_key = secret_key.public_key();

    for size in [100usize, 1024, 100 * 1024, 900 * 1024] {
        let data = vec![0u8; size];
        let signature = secret_key.sign_v1(data.as_slice());
        let receipt = test_runner.execute_manifest(
            ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 500u32)
                .call_function(
                    package_address,
                    "CryptoScrypto",
                    "bls12381_v1_verify",
                    manifest_args!(data, public_key, signature),
                )
                .build(),
            vec![],
        );
        let _ = receipt.expect_commit_success();
    }
}
