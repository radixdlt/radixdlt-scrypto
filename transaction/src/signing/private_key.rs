use sbor::rust::vec::Vec;
use secp256k1::{Message, PublicKey, SecretKey};

use scrypto::crypto::*;

pub struct EcdsaPrivateKey(SecretKey);

impl EcdsaPrivateKey {
    pub const LENGTH: usize = 32;

    pub fn public_key(&self) -> EcdsaPublicKey {
        EcdsaPublicKey(PublicKey::from_secret_key_global(&self.0))
    }

    pub fn sign(&self, msg: &[u8]) -> EcdsaSignature {
        let h = hash(msg);
        let m = Message::from_slice(&h.0).expect("The slice is a valid hash");
        EcdsaSignature(self.0.sign_ecdsa(m))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.secret_bytes().to_vec()
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != EcdsaPrivateKey::LENGTH {
            return Err(());
        }
        Ok(Self(SecretKey::from_slice(slice).map_err(|_| ())?))
    }

    pub fn from_u64(n: u64) -> Result<Self, ()> {
        let mut bytes = [0u8; EcdsaPrivateKey::LENGTH];
        (&mut bytes[EcdsaPrivateKey::LENGTH - 8..EcdsaPrivateKey::LENGTH])
            .copy_from_slice(&n.to_be_bytes());

        Ok(Self(SecretKey::from_slice(&bytes).map_err(|_| ())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::str::FromStr;
    use scrypto::{
        crypto::Hash,
        prelude::ECDSA_TOKEN,
        resource::{NonFungibleAddress, NonFungibleId},
    };

    #[test]
    fn sign_and_verify() {
        // From Babylon Wallet PoC
        let test_sk = "0000000000000000000000000000000000000000000000000000000000000001";
        let test_pk = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
        let test_message = "{\"a\":\"banan\"}";
        let test_hash = "c43a1e3a7e822c97004267324ba8df88d114ab3e019d0e85eccb1ff8592d6d36";
        let test_signature = "403b07ead5e7513064163c23590444d72b2db0fc14a08a9312f483578ed8e1aa317b54b96124b2bb31775ae6a62ae4107ea0549199343243dc19d0df36261d51";
        let sk = EcdsaPrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
        let pk = EcdsaPublicKey::from_str(test_pk).unwrap();
        let hash = Hash::from_str(test_hash).unwrap();
        let sig = EcdsaSignature::from_str(test_signature).unwrap();

        assert_eq!(sk.public_key(), pk);
        assert_eq!(scrypto::crypto::hash(test_message), hash);
        assert_eq!(sk.sign(test_message.as_bytes()), sig);
        assert!(EcdsaVerifier::verify(test_message.as_bytes(), &pk, &sig));
    }

    #[test]
    fn test_non_fungible_address_codec() {
        let expected = "030000000000000000000000000000000000000000000000000005031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f";
        let private_key = EcdsaPrivateKey::from_bytes(&[1u8; 32]).unwrap();
        let public_key = private_key.public_key();
        let auth_address =
            NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::from_bytes(public_key.to_vec()));
        let s1 = auth_address.to_string();
        let auth_address2 = NonFungibleAddress::from_str(&s1).unwrap();
        let s2 = auth_address2.to_string();
        assert_eq!(s1, expected);
        assert_eq!(s2, expected);
    }
}
