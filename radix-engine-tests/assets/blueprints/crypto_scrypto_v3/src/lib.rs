use scrypto::prelude::*;

#[blueprint]
mod component_module {
    struct CryptoScrypto {}

    impl CryptoScrypto {
        pub fn bls12381_v1_verify_min_sig(
            message: Vec<u8>,
            pub_key: Bls12381G2PublicKey,
            signature: Bls12381G1Signature,
        ) -> bool {
            CryptoUtils::bls12381_v1_verify_min_sig(message, pub_key, signature)
        }
    }
}
