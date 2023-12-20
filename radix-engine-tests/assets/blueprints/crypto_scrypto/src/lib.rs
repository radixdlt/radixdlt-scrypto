use scrypto::prelude::*;

#[blueprint]
mod component_module {
    struct CryptoScrypto {}

    impl CryptoScrypto {
        pub fn bls12381_v1_verify(
            message: Vec<u8>,
            pub_key: Bls12381G1PublicKey,
            signature: Bls12381G2Signature,
        ) -> bool {
            CryptoUtils::bls12381_v1_verify(message, pub_key, signature)
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
