use radix_common::types::PackageAddress;

#[derive(Clone, Copy, Debug)]
pub enum ClientCostingEntry<'a> {
    RunNativeCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
        input_size: usize,
    },
    RunWasmCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
        wasm_execution_units: u32,
    },
    PrepareWasmCode {
        size: usize,
    },
    Bls12381V1Verify {
        size: usize,
    },
    Bls12381V1AggregateVerify {
        sizes: &'a [usize],
    },
    Bls12381V1FastAggregateVerify {
        size: usize,
        keys_cnt: usize,
    },
    Bls12381G2SignatureAggregate {
        signatures_cnt: usize,
    },
    Keccak256Hash {
        size: usize,
    },
    Blake2b256Hash {
        size: usize,
    },
    Ed25519Verify {
        size: usize,
    },
    Secp256k1EcdsaVerify,
    Secp256k1EcdsaKeyRecover,
}
