use scrypto::prelude::*;

#[blueprint]
mod component_module {
    // TODO: remove below
    const CRYPTO_UTILS_BLUEPRINT: &str = "CryptoUtils";
    //const CRYPTO_UTILS_BLS_VERIFY_IDENT: &str = "bls_verify";
    const CRYPTO_UTILS_KECCAK_HASH_IDENT: &str = "keccak_hash";

    struct CryptoScrypto {}

    impl CryptoScrypto {
        pub fn keccak_hash(data: Vec<u8>) -> Hash {
            #[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
            struct CryptoUtilsKeccakHashInput {
                pub data: Vec<u8>,
            }

            let rtn = ScryptoVmV1Api::blueprint_call(
                CRYPTO_UTILS_PACKAGE,
                CRYPTO_UTILS_BLUEPRINT,
                CRYPTO_UTILS_KECCAK_HASH_IDENT,
                scrypto_encode(&CryptoUtilsKeccakHashInput { data }).unwrap(),
            );
            let hash: Hash = scrypto_decode(&rtn).unwrap();
            hash
        }
    }
}
