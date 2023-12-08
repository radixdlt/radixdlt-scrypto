use super::Bls12381G2Signature;
use crate::internal_prelude::*;
use blst::min_pk::SecretKey;

pub struct Bls12381G1PrivateKey(SecretKey);

impl Bls12381G1PrivateKey {
    pub const LENGTH: usize = 32;

    pub fn public_key(&self) -> Bls12381G1PublicKey {
        Bls12381G1PublicKey(self.0.sk_to_pk().to_bytes())
    }

    pub fn sign_v1(&self, message: &[u8]) -> Bls12381G2Signature {
        let signature = self.0.sign(message, BLS12381_CIPHERSITE_V1, &[]).to_bytes();
        Bls12381G2Signature(signature)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != Bls12381G1PrivateKey::LENGTH {
            return Err(());
        }
        Ok(Self(SecretKey::from_bytes(slice).map_err(|_| ())?))
    }

    pub fn from_u64(n: u64) -> Result<Self, ()> {
        let mut bytes = [0u8; Bls12381G1PrivateKey::LENGTH];
        (&mut bytes[Bls12381G1PrivateKey::LENGTH - 8..Bls12381G1PrivateKey::LENGTH])
            .copy_from_slice(&n.to_be_bytes());

        Ok(Self(SecretKey::from_bytes(&bytes).map_err(|_| ())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::str::FromStr;

    #[test]
    fn sign_and_verify() {
        let test_sk = "408157791befddd702672dcfcfc99da3512f9c0ea818890fcb6ab749580ef2cf";
        let test_pk = "93b1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2";
        let test_message_hash = hash("Test").as_bytes().to_vec();
        let test_signature = "8b84ff5a1d4f8095ab8a80518ac99230ed24a7d1ec90c4105f9c719aa7137ed5d7ce1454d4a953f5f55f3959ab416f3014f4cd2c361e4d32c6b4704a70b0e2e652a908f501acb54ec4e79540be010e3fdc1fbf8e7af61625705e185a71c884f1";
        let sk = Bls12381G1PrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
        let pk = Bls12381G1PublicKey::from_str(test_pk).unwrap();
        let sig = Bls12381G2Signature::from_str(test_signature).unwrap();

        assert_eq!(sk.public_key(), pk);
        assert_eq!(sk.sign_v1(&test_message_hash), sig);
        assert!(verify_bls12381_v1(&test_message_hash, &pk, &sig));
    }

    #[test]
    fn sign_and_verify_supra() {
        // Supra example
        let test_pk = "8a38419cb83c15a92d11243384bea0acd15cbacc24b385b9c577b17272d6ad68bb53c52dbbf79324005528d2d73c2643";
        let test_sk = "5B00CC8C7153F39EF2E6E2FADB1BB95A1F4BF21F43CC5B28EFA9E526FB788C08";
        let test_message_hash = keccak_256_hash("Hello World!");

        assert_eq!(
            test_message_hash,
            Hash::from_str("3ea2f1d0abf3fc66cf29eebb70cbd4e7fe762ef8a09bcc06c8edf641230afec0")
                .unwrap()
        );
        let test_message_hash = test_message_hash.to_vec();

        let test_signature = "82131f69b6699755f830e29d6ed41cbf759591a2ab598aa4e9686113341118d1db900d190436048601791121b5757c341045d4d0c94a95ec31a9ba6205f9b7504de85dadff52874375c58eec6cec28397279de87d5595101e398d31646d345bb";

        let sk = Bls12381G1PrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
        let pk = Bls12381G1PublicKey::from_str(test_pk).unwrap();
        let sig = Bls12381G2Signature::from_str(test_signature).unwrap();

        assert_eq!(sk.public_key(), pk);
        assert_eq!(sk.sign_v1(&test_message_hash), sig);
        assert!(verify_bls12381_v1(&test_message_hash, &pk, &sig));
    }
}
