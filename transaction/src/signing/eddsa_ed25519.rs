use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};
use sbor::rust::vec::Vec;

use scrypto::crypto::*;

pub struct EddsaEd25519PrivateKey(SecretKey);

impl EddsaEd25519PrivateKey {
    pub const LENGTH: usize = 32;

    pub fn public_key(&self) -> EddsaEd25519PublicKey {
        EddsaEd25519PublicKey(PublicKey::from(&self.0).to_bytes())
    }

    pub fn sign(&self, msg: &[u8]) -> EddsaEd25519Signature {
        let keypair = Keypair {
            secret: SecretKey::from_bytes(self.0.as_bytes()).expect("From a valid key bytes"),
            public: PublicKey::from(&self.0),
        };

        // SHA512 is used here

        EddsaEd25519Signature(keypair.sign(msg).to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != EddsaEd25519PrivateKey::LENGTH {
            return Err(());
        }
        Ok(Self(SecretKey::from_bytes(slice).map_err(|_| ())?))
    }

    pub fn from_u64(n: u64) -> Result<Self, ()> {
        let mut bytes = [0u8; EddsaEd25519PrivateKey::LENGTH];
        (&mut bytes[EddsaEd25519PrivateKey::LENGTH - 8..EddsaEd25519PrivateKey::LENGTH])
            .copy_from_slice(&n.to_be_bytes());

        Ok(Self(SecretKey::from_bytes(&bytes).map_err(|_| ())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::verify_eddsa_ed25519;
    use sbor::rust::str::FromStr;

    #[test]
    fn sign_and_verify() {
        let test_sk = "0000000000000000000000000000000000000000000000000000000000000001";
        let test_pk = "4cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29";
        let test_message = "Test";
        let test_signature = "ce993adc51111309a041faa65cbcf1154d21ed0ecdc2d54070bc90b9deb744aa8605b3f686fa178fba21070b4a4678e54eee3486a881e0e328251cd37966de09";
        let sk = EddsaEd25519PrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
        let pk = EddsaEd25519PublicKey::from_str(test_pk).unwrap();
        let sig = EddsaEd25519Signature::from_str(test_signature).unwrap();

        assert_eq!(sk.public_key(), pk);
        assert_eq!(sk.sign(test_message.as_bytes()), sig);
        assert!(verify_eddsa_ed25519(test_message.as_bytes(), &pk, &sig));
    }
}
