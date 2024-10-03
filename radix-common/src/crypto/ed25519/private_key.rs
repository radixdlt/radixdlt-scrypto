use super::Ed25519Signature;
use crate::internal_prelude::*;
use ed25519_dalek::{Signer, SigningKey};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
pub struct Ed25519PrivateKey(SigningKey);

impl Zeroize for Ed25519PrivateKey {
    fn zeroize(&mut self) {
        let mut secret_key = self.0.to_bytes();
        secret_key.zeroize();
        self.0 = SigningKey::from_bytes(&secret_key);
    }
}

impl Ed25519PrivateKey {
    pub const LENGTH: usize = 32;

    pub fn public_key(&self) -> Ed25519PublicKey {
        Ed25519PublicKey(self.0.verifying_key().to_bytes())
    }

    pub fn sign(&self, msg_hash: &impl IsHash) -> Ed25519Signature {
        // SHA512 is used here

        Ed25519Signature(self.0.sign(msg_hash.as_ref()).to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != Ed25519PrivateKey::LENGTH {
            return Err(());
        }

        let signing_key = SigningKey::try_from(slice).map_err(|_| ())?;

        Ok(Self(signing_key))
    }

    pub fn from_u64(n: u64) -> Result<Self, ()> {
        let mut bytes = [0u8; Ed25519PrivateKey::LENGTH];
        (&mut bytes[Ed25519PrivateKey::LENGTH - 8..Ed25519PrivateKey::LENGTH])
            .copy_from_slice(&n.to_be_bytes());

        Ok(Self(SigningKey::from_bytes(&bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::str::FromStr;

    #[test]
    fn sign_and_verify() {
        let test_sk = "0000000000000000000000000000000000000000000000000000000000000001";
        let test_pk = "4cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29";
        let test_message_hash = hash("Test");
        let test_signature = "cf0ca64435609b85ab170da339d415bbac87d678dfd505969be20adc6b5971f4ee4b4620c602bcbc34fd347596546675099d696265f4a42a16df343da1af980e";
        let sk = Ed25519PrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
        let pk = Ed25519PublicKey::from_str(test_pk).unwrap();
        let sig = Ed25519Signature::from_str(test_signature).unwrap();

        assert_eq!(sk.public_key(), pk);
        assert_eq!(sk.sign(&test_message_hash), sig);
        assert!(verify_ed25519(&test_message_hash, &pk, &sig));
    }

    #[test]
    fn verify_zeroize() {
        let bytes = "4fd3fb62d6b7a4749f75d56d06b0aea1ec2c2a6986d2bfa975d7891585590fea";
        let mut key = Ed25519PrivateKey::from_bytes(&hex::decode(bytes).unwrap()).unwrap();

        key.zeroize();

        assert_eq!(key.to_bytes(), [0; 32].to_vec());
    }
}
