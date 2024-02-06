use scrypto::*;
use scrypto_test::prelude::*;
use scrypto_unit::*;

#[test]
fn test_crypto_scrypto_verify_bls12381_v1() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish(this_package!());

    let msg1 = hash("Test").to_vec();
    let pk = "93b1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2";
    let msg1_signature = "8b84ff5a1d4f8095ab8a80518ac99230ed24a7d1ec90c4105f9c719aa7137ed5d7ce1454d4a953f5f55f3959ab416f3014f4cd2c361e4d32c6b4704a70b0e2e652a908f501acb54ec4e79540be010e3fdc1fbf8e7af61625705e185a71c884f1";

    let pk = Bls12381G1PublicKey::from_str(pk).unwrap();
    let msg1_signature = Bls12381G2Signature::from_str(msg1_signature).unwrap();

    let msg1_verify: bool = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 500u32)
                .call_function(
                    package_address,
                    "CryptoScrypto",
                    "bls12381_v1_verify",
                    manifest_args!(msg1, pk, msg1_signature,),
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .output(1);

    assert!(msg1_verify);
}
