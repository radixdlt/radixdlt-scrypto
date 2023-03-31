use super::EddsaEd25519Signature;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};
use radix_engine_interface::crypto::EddsaEd25519PublicKey;
use radix_engine_interface::crypto::Hash;
use sbor::rust::vec::Vec;

pub struct EddsaEd25519PrivateKey(SecretKey);

impl EddsaEd25519PrivateKey {
    pub const LENGTH: usize = 32;

    pub fn public_key(&self) -> EddsaEd25519PublicKey {
        EddsaEd25519PublicKey(PublicKey::from(&self.0).to_bytes())
    }

    pub fn sign(&self, msg_hash: &Hash) -> EddsaEd25519Signature {
        let keypair = Keypair {
            secret: SecretKey::from_bytes(self.0.as_bytes()).expect("From a valid key bytes"),
            public: PublicKey::from(&self.0),
        };

        // SHA512 is used here

        EddsaEd25519Signature(keypair.sign(&msg_hash.0).to_bytes())
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
    use radix_engine_interface::crypto::hash;
    use sbor::rust::str::FromStr;

    #[test]
    fn sign_and_verify() {
        let test_sk = "0000000000000000000000000000000000000000000000000000000000000001";
        let test_pk = "4cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29";
        let test_message_hash = hash("Test");
        let test_signature = "cf0ca64435609b85ab170da339d415bbac87d678dfd505969be20adc6b5971f4ee4b4620c602bcbc34fd347596546675099d696265f4a42a16df343da1af980e";
        let sk = EddsaEd25519PrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
        let pk = EddsaEd25519PublicKey::from_str(test_pk).unwrap();
        let sig = EddsaEd25519Signature::from_str(test_signature).unwrap();

        assert_eq!(sk.public_key(), pk);
        assert_eq!(sk.sign(&test_message_hash), sig);
        assert!(verify_eddsa_ed25519(&test_message_hash, &pk, &sig));
    }
}
