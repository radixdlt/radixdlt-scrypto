use radix_engine_tests::prelude::*;

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
            .to_string()
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
    runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    msg: Vec<u8>,
    pub_key: Bls12381G1PublicKey,
    signature: Bls12381G2Signature,
) -> TransactionReceiptV1 {
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
    runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    msgs: Vec<Vec<u8>>,
    pub_keys: Vec<Bls12381G1PublicKey>,
    signature: Bls12381G2Signature,
) -> TransactionReceiptV1 {
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
                manifest_args!(pub_keys_msgs, signature),
            )
            .build(),
        vec![],
    )
}

fn crypto_scrypto_bls12381_v1_fast_aggregate_verify(
    runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    msg: Vec<u8>,
    pub_keys: Vec<Bls12381G1PublicKey>,
    signature: Bls12381G2Signature,
) -> TransactionReceiptV1 {
    runner.execute_manifest(
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
    )
}

fn crypto_scrypto_bls12381_g2_signature_aggregate(
    runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    signatures: Vec<Bls12381G2Signature>,
) -> TransactionReceiptV1 {
    runner.execute_manifest(
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
    )
}

fn crypto_scrypto_keccak256_hash(
    runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    data: Vec<u8>,
) -> TransactionReceiptV1 {
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

#[test]
fn test_crypto_scrypto_verify_bls12381_v1() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let msg1 = hash("Test").to_vec();
    let pk = "93b1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2";
    let msg1_signature = "8b84ff5a1d4f8095ab8a80518ac99230ed24a7d1ec90c4105f9c719aa7137ed5d7ce1454d4a953f5f55f3959ab416f3014f4cd2c361e4d32c6b4704a70b0e2e652a908f501acb54ec4e79540be010e3fdc1fbf8e7af61625705e185a71c884f1";

    let pk = Bls12381G1PublicKey::from_str(pk).unwrap();
    let msg1_signature = Bls12381G2Signature::from_str(msg1_signature).unwrap();
    // Act
    let msg1_verify: bool = get_output!(crypto_scrypto_bls12381_v1_verify(
        &mut test_runner,
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
        &mut test_runner,
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
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let (_sks, _pks, _msgs, sigs) = get_aggregate_verify_test_data(10, 10);

    // Aggregate the signature
    let agg_sig_multiple_msgs = Bls12381G2Signature::aggregate(&sigs).unwrap();

    // Act
    let agg_sig_from_scrypto =
        crypto_scrypto_bls12381_g2_signature_aggregate(&mut test_runner, package_address, sigs)
            .expect_commit_success()
            .output(1);

    // Assert
    assert_eq!(agg_sig_multiple_msgs, agg_sig_from_scrypto);

    // Attempt to aggregate signature from empty input
    let error_message =
        crypto_scrypto_bls12381_g2_signature_aggregate(&mut test_runner, package_address, vec![])
            .expect_commit_failure()
            .outcome
            .expect_failure()
            .to_string();

    // Assert
    assert!(error_message.contains("InputDataEmpty"));
}

#[test]
fn test_crypto_scrypto_bls12381_aggregate_verify() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let (_sks, pks, msgs, sigs) = get_aggregate_verify_test_data(10, 10);

    // Aggregate the signature
    let agg_sig_multiple_msgs = Bls12381G2Signature::aggregate(&sigs).unwrap();

    // Act
    let agg_sig_from_scrypto = get_output!(crypto_scrypto_bls12381_g2_signature_aggregate(
        &mut test_runner,
        package_address,
        sigs
    ));

    // Assert
    assert_eq!(agg_sig_multiple_msgs, agg_sig_from_scrypto);

    // Act
    let agg_verify: bool = get_output!(crypto_scrypto_bls12381_v1_aggregate_verify(
        &mut test_runner,
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
        &mut test_runner,
        package_address,
        msgs.clone(),
        pks_rev,
        agg_sig_multiple_msgs,
    ));

    // Attempt to verify signature of empty message vector
    let empty_message_error = get_failure!(crypto_scrypto_bls12381_v1_aggregate_verify(
        &mut test_runner,
        package_address,
        vec![],
        pks,
        agg_sig_multiple_msgs,
    ));

    // Attempt to verify signature using empty keys vector
    let empty_keys_error = get_failure!(crypto_scrypto_bls12381_v1_aggregate_verify(
        &mut test_runner,
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
    let agg_sig_from_scrypto: Bls12381G2Signature = get_output!(
        crypto_scrypto_bls12381_g2_signature_aggregate(&mut test_runner, package_address, sigs)
    );

    // Assert
    assert_eq!(agg_sig_single_msg, agg_sig_from_scrypto);

    // Act
    let agg_verify: bool = get_output!(crypto_scrypto_bls12381_v1_fast_aggregate_verify(
        &mut test_runner,
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
            &mut test_runner,
            package_address,
            msg_false,
            pks,
            agg_sig_single_msg,
        ));

    // Attempt to verify signature using empty keys vector
    let empty_keys_error = get_failure!(crypto_scrypto_bls12381_v1_fast_aggregate_verify(
        &mut test_runner,
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
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let data1 = b"Hello Radix".to_vec();
    let data2 = b"xidaR olleH".to_vec();
    let data3: Vec<u8> = vec![]; // empty data

    // Act
    let data1_hash: Hash = get_output!(crypto_scrypto_keccak256_hash(
        &mut test_runner,
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
        &mut test_runner,
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
        &mut test_runner,
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
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    let msg = b"Important message".to_vec();

    // Act
    // Get the hash of the message using CryptoScrypto package
    let msg_hash: Vec<u8> = {
        let hash: Hash = get_output!(crypto_scrypto_keccak256_hash(
            &mut test_runner,
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
        &mut test_runner,
        package_address,
        msg_hash,
        public_key,
        msg_signature,
    ));

    // Assert
    assert!(result);
}

#[test]
fn test_crypto_scrypto_keccak256_costing() {
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

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
        let _hash = crypto_scrypto_keccak256_hash(&mut test_runner, package_address, data);
    }
}

#[test]
fn test_crypto_scrypto_verify_bls12381_v1_costing() {
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

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
            &mut test_runner,
            package_address,
            data,
            public_key,
            signature,
        );
    }
}

#[test]
fn test_crypto_scrypto_bls12381_g2_signature_aggregate_costing() {
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    for cnt in [1, 2, 5, 10, 20, 50, 100] {
        let sks: Vec<Bls12381G1PrivateKey> = (1..(cnt + 1))
            .map(|i| Bls12381G1PrivateKey::from_u64(i).unwrap())
            .collect();

        // Single message
        let msg = b"One message to sign for all".to_vec();

        let sigs: Vec<Bls12381G2Signature> = sks.iter().map(|sk| sk.sign_v1(&msg)).collect();

        // Act
        let _ =
            crypto_scrypto_bls12381_g2_signature_aggregate(&mut test_runner, package_address, sigs);
    }
}

#[test]
fn test_crypto_scrypto_bls12381_v1_aggregate_verify_costing() {
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    for msg_size in [100usize, 200, 500, 1024, 10 * 1024, 20 * 1024] {
        for cnt in [1u32, 2, 5, 10, 20] {
            let (_sks, pks, msgs, sigs) = get_aggregate_verify_test_data(cnt, msg_size);

            let agg_sig_multiple_msgs = Bls12381G2Signature::aggregate(&sigs).unwrap();

            let _ = crypto_scrypto_bls12381_v1_aggregate_verify(
                &mut test_runner,
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
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    for (cnt, msg_size) in [
        (1, 100 * 1024),
        (2, 50 * 1024),
        (5, 20 * 1024),
        (10, 10 * 1024),
        (100, 1024),
        (1024, 100),
    ] {
        let (_sks, pks, msgs, sigs) = get_aggregate_verify_test_data(cnt, msg_size);
        let agg_sig = Bls12381G2Signature::aggregate(&sigs).unwrap();

        let _ = crypto_scrypto_bls12381_v1_aggregate_verify(
            &mut test_runner,
            package_address,
            msgs,
            pks,
            agg_sig,
        );
    }

    // 1x 99kB and 1000x1B
    let (mut sks1, mut pks1, mut msgs1, mut sigs1) = get_aggregate_verify_test_data(1, 99 * 1024);
    let (mut sks2, mut pks2, mut msgs2, mut sigs2) = get_aggregate_verify_test_data(1000, 1);
    sks1.append(&mut sks2);
    pks1.append(&mut pks2);
    msgs1.append(&mut msgs2);
    sigs1.append(&mut sigs2);
    let agg_sig = Bls12381G2Signature::aggregate(&sigs1).unwrap();

    let _ = crypto_scrypto_bls12381_v1_aggregate_verify(
        &mut test_runner,
        package_address,
        msgs1,
        pks1,
        agg_sig,
    );

    // 1x 90kB and 10 x 1kB
    let (mut sks1, mut pks1, mut msgs1, mut sigs1) = get_aggregate_verify_test_data(1, 90 * 1024);
    let (mut sks2, mut pks2, mut msgs2, mut sigs2) = get_aggregate_verify_test_data(10, 1024);
    sks1.append(&mut sks2);
    pks1.append(&mut pks2);
    msgs1.append(&mut msgs2);
    sigs1.append(&mut sigs2);
    let agg_sig = Bls12381G2Signature::aggregate(&sigs1).unwrap();

    let _ = crypto_scrypto_bls12381_v1_aggregate_verify(
        &mut test_runner,
        package_address,
        msgs1,
        pks1,
        agg_sig,
    );
}

#[test]
fn test_crypto_scrypto_bls12381_v1_fast_aggregate_verify_costing() {
    let mut test_runner = TestRunnerBuilder::new().build();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("crypto_scrypto"));

    for msg_size in [100usize, 200, 500, 1024, 10 * 1024, 20 * 1024] {
        for cnt in [1u8, 2, 5, 10, 20, 50, 100] {
            let sks: Vec<Bls12381G1PrivateKey> = (1..(cnt + 1))
                .map(|i| Bls12381G1PrivateKey::from_u64(i.into()).unwrap())
                .collect();

            // Single message
            let msg: Vec<u8> = vec![cnt; msg_size];

            let sigs: Vec<Bls12381G2Signature> = sks.iter().map(|sk| sk.sign_v1(&msg)).collect();

            let pks: Vec<Bls12381G1PublicKey> = sks.iter().map(|sk| sk.public_key()).collect();

            let agg_sig_single_msg = Bls12381G2Signature::aggregate(&sigs).unwrap();

            let _ = crypto_scrypto_bls12381_v1_fast_aggregate_verify(
                &mut test_runner,
                package_address,
                msg,
                pks,
                agg_sig_single_msg,
            );
        }
    }
}
