use scrypto::prelude::*;

#[blueprint]
mod component_module {
    struct CryptoScrypto {}

    impl CryptoScrypto {
        pub fn bls_verify(msg_hash: Hash, pub_key: BlsPublicKey, signature: BlsSignature) -> bool {
            CryptoUtils::bls_verify(msg_hash, pub_key, signature)
        }

        pub fn keccak_hash(data: Vec<u8>) -> Hash {
            let hash = CryptoUtils::keccak_hash(data);
            hash
        }
    }
}
