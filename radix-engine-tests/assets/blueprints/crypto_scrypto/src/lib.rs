use scrypto::prelude::*;

#[blueprint]
mod component_module {
    struct CryptoScrypto {}

    impl CryptoScrypto {
        pub fn bls_verify(
            message: Vec<u8>,
            pub_key: BlsPublicKey,
            signature: BlsSignature,
        ) -> bool {
            CryptoUtils::bls_verify(message, pub_key, signature)
        }

        pub fn keccak_hash(data: Vec<u8>) -> Hash {
            let hash = CryptoUtils::keccak_hash(data);
            hash
        }
    }
}
