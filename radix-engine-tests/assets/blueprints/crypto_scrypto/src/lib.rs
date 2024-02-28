use scrypto::prelude::*;

#[blueprint]
mod component_module {
    struct CryptoScrypto {
        pub_key: Bls12381G1PublicKey,
    }

    impl CryptoScrypto {
        pub fn bls12381_v1_verify(
            message: Vec<u8>,
            pub_key: Bls12381G1PublicKey,
            signature: Bls12381G2Signature,
        ) -> bool {
            CryptoUtils::bls12381_v1_verify(message, pub_key, signature)
        }

        pub fn bls12381_v1_aggregate_verify(
            pub_keys_msgs: Vec<(Bls12381G1PublicKey, Vec<u8>)>,
            signature: Bls12381G2Signature,
        ) -> bool {
            CryptoUtils::bls12381_v1_aggregate_verify(pub_keys_msgs, signature)
        }

        pub fn bls12381_v1_fast_aggregate_verify(
            message: Vec<u8>,
            pub_keys: Vec<Bls12381G1PublicKey>,
            signature: Bls12381G2Signature,
        ) -> bool {
            CryptoUtils::bls12381_v1_fast_aggregate_verify(message, pub_keys, signature)
        }

        pub fn bls12381_g2_signature_aggregate(
            signatures: Vec<Bls12381G2Signature>,
        ) -> Bls12381G2Signature {
            CryptoUtils::bls12381_g2_signature_aggregate(signatures)
        }

        pub fn keccak256_hash(data: Vec<u8>) -> Hash {
            let hash = CryptoUtils::keccak256_hash(data);
            hash
        }
    }
}
