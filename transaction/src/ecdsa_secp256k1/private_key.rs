use radix_engine_interface::crypto::{EcdsaSecp256k1PublicKey, Hash};
use sbor::rust::vec::Vec;
use secp256k1::{Message, PublicKey, SecretKey};

use super::EcdsaSecp256k1Signature;

pub struct EcdsaSecp256k1PrivateKey(SecretKey);

impl EcdsaSecp256k1PrivateKey {
    pub const LENGTH: usize = 32;

    pub fn public_key(&self) -> EcdsaSecp256k1PublicKey {
        EcdsaSecp256k1PublicKey(PublicKey::from_secret_key_global(&self.0).serialize())
    }

    pub fn sign(&self, msg_hash: &Hash) -> EcdsaSecp256k1Signature {
        let m = Message::from_slice(&msg_hash.0).expect("Hash is always a valid message");
        let signature = secp256k1::SECP256K1.sign_ecdsa_recoverable(&m, &self.0);
        let (recovery_id, signature_data) = signature.serialize_compact();

        let mut buf = [0u8; 65];
        buf[0] = recovery_id.to_i32() as u8;
        buf[1..].copy_from_slice(&signature_data);
        EcdsaSecp256k1Signature(buf)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.secret_bytes().to_vec()
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != EcdsaSecp256k1PrivateKey::LENGTH {
            return Err(());
        }
        Ok(Self(SecretKey::from_slice(slice).map_err(|_| ())?))
    }

    pub fn from_u64(n: u64) -> Result<Self, ()> {
        let mut bytes = [0u8; EcdsaSecp256k1PrivateKey::LENGTH];
        (&mut bytes[EcdsaSecp256k1PrivateKey::LENGTH - 8..EcdsaSecp256k1PrivateKey::LENGTH])
            .copy_from_slice(&n.to_be_bytes());

        Ok(Self(SecretKey::from_slice(&bytes).map_err(|_| ())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::verify_ecdsa_secp256k1;
    use radix_engine_interface::crypto::hash;
    use sbor::rust::str::FromStr;

    #[test]
    fn sign_and_verify() {
        let test_sk = "0000000000000000000000000000000000000000000000000000000000000001";
        let test_pk = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
        let test_message_hash = hash("Test");
        let test_signature = "00eb8dcd5bb841430dd0a6f45565a1b8bdb4a204eb868832cd006f963a89a662813ab844a542fcdbfda4086a83fbbde516214113051b9c8e42a206c98d564d7122";
        let sk = EcdsaSecp256k1PrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
        let pk = EcdsaSecp256k1PublicKey::from_str(test_pk).unwrap();
        let sig = EcdsaSecp256k1Signature::from_str(test_signature).unwrap();

        assert_eq!(sk.public_key(), pk);
        assert_eq!(sk.sign(&test_message_hash), sig);
        assert!(verify_ecdsa_secp256k1(&test_message_hash, &pk, &sig));
    }
}
