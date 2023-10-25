mod package_loader;

use package_loader::PackageLoader;
use radix_engine::transaction::CostingParameters;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_bls_signature() {
    //==================
    // Execute locally
    //==================
    use bls_signatures::*;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;
    use rayon::prelude::*;
    use std::time::Instant;

    let mut rng = ChaCha8Rng::seed_from_u64(12);
    const N: usize = 5;
    const MESSAGE_SIZE: usize = 64;

    // Key pairs
    let private_keys: Vec<PrivateKey> = (0..N).map(|_| PrivateKey::generate(&mut rng)).collect();
    let public_keys = private_keys
        .par_iter()
        .map(|pk| pk.public_key().as_bytes())
        .collect::<Vec<_>>();

    // Messages
    let messages: Vec<Vec<u8>> = (0..N)
        .map(|_| (0..MESSAGE_SIZE).map(|_| rng.gen()).collect())
        .collect();

    // Signatures
    let signatures: Vec<Signature> = messages
        .par_iter()
        .zip(private_keys.par_iter())
        .map(|(message, pk)| pk.sign(message))
        .collect();
    let aggregated_signature = aggregate(&signatures)
        .expect("failed to aggregate")
        .as_bytes();

    let start = Instant::now();
    let hashes = messages
        .iter()
        .map(|message| hash(message))
        .collect::<Vec<_>>();
    assert!(verify(
        &Signature::from_bytes(&aggregated_signature).unwrap(),
        &hashes,
        &public_keys
            .iter()
            .map(|bytes| PublicKey::from_bytes(&bytes).unwrap())
            .collect::<Vec<PublicKey>>()
    ));
    println!("Time elapsed: {} microseconds", start.elapsed().as_micros());

    //==================
    // Execute with WASM
    //==================
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("bls"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "BLS",
            "verify",
            manifest_args!(messages, public_keys, aggregated_signature),
        )
        .build();
    let receipt = test_runner.execute_manifest_with_costing_params(
        manifest,
        vec![],
        CostingParameters::default().with_execution_cost_unit_limit(u32::MAX),
    );
    println!("{:?}", receipt);
}
