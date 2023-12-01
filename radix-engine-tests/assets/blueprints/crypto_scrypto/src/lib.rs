use scrypto::prelude::*;

#[blueprint]
mod component_module {
    struct CryptoScrypto {}

    impl CryptoScrypto {
        pub fn bls_verify(msg_hash: Hash, pub_key: BlsPublicKey, signature: BlsSignature) -> bool {
            let rtn = ScryptoVmV1Api::blueprint_call(
                CRYPTO_UTILS_PACKAGE,
                CRYPTO_UTILS_BLUEPRINT,
                CRYPTO_UTILS_BLS_VERIFY_IDENT,
                scrypto_encode(&CryptoUtilsBlsVerifyInput {
                    msg_hash,
                    pub_key,
                    signature,
                })
                .unwrap(),
            );
            let result: bool = scrypto_decode(&rtn).unwrap();
            result
        }

        pub fn keccak_hash(data: Vec<u8>) -> Hash {
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
