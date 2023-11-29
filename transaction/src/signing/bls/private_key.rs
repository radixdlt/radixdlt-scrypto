use super::BlsSignature;
use crate::internal_prelude::*;
use blst::min_pk::SecretKey;

pub struct BlsPrivateKey(SecretKey);

impl BlsPrivateKey {
    pub const LENGTH: usize = 32;

    pub fn public_key(&self) -> BlsPublicKey {
        BlsPublicKey(self.0.sk_to_pk().to_bytes())
    }

    pub fn sign(&self, msg_hash: &impl IsHash) -> BlsSignature {
        let signature = self.0.sign(msg_hash.as_ref(), BLS_SCHEME, &[]).to_bytes();
        BlsSignature(signature)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != BlsPrivateKey::LENGTH {
            return Err(());
        }
        Ok(Self(SecretKey::from_bytes(slice).map_err(|_| ())?))
    }

    pub fn from_u64(n: u64) -> Result<Self, ()> {
        let mut bytes = [0u8; BlsPrivateKey::LENGTH];
        (&mut bytes[BlsPrivateKey::LENGTH - 8..BlsPrivateKey::LENGTH])
            .copy_from_slice(&n.to_be_bytes());

        Ok(Self(SecretKey::from_bytes(&bytes).map_err(|_| ())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::verify_bls;
    use radix_engine_interface::crypto::hash;
    use sbor::rust::str::FromStr;

    #[test]
    fn sign_and_verify() {
        let test_sk = "408157791befddd702672dcfcfc99da3512f9c0ea818890fcb6ab749580ef2cf";
        let test_pk = "93b1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2";
        let test_message_hash = hash("Test");
        let test_signature = "a2ba96a1fc1e698b7688e077f171fbd7fe99c6bbf240b1421a08e3faa5d6b55523a18b8c77fba5830181dfec716edc3d18a8657bcadd0a83e3cafdad33998d10417f767c536b26b98df41d67ab416c761ad55438f23132a136fc82eb7b290571";
        let sk = BlsPrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
        let pk = BlsPublicKey::from_str(test_pk).unwrap();
        let sig = BlsSignature::from_str(test_signature).unwrap();

        assert_eq!(sk.public_key(), pk);
        assert_eq!(sk.sign(&test_message_hash), sig);
        assert!(verify_bls(&test_message_hash, &pk, &sig));
    }
}
