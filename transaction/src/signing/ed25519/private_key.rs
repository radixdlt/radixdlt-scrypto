use super::Ed25519Signature;
use crate::internal_prelude::*;
use ed25519_dalek::{ExpandedSecretKey, PublicKey, SecretKey};

pub struct Ed25519PrivateKey(ExpandedSecretKey);

impl Ed25519PrivateKey {
    pub fn public_key(&self) -> Ed25519PublicKey {
        Ed25519PublicKey(PublicKey::from(&self.0).to_bytes())
    }

    pub fn sign(&self, msg_hash: &impl IsHash) -> Ed25519Signature {
        Ed25519Signature(
            self.0
                .sign(msg_hash.as_ref(), &PublicKey::from(&self.0))
                .to_bytes(),
        )
    }

    pub fn to_scalar_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()[0..32].try_into().unwrap()
    }

    /// Note - this should only be used for non-signing applications such as DH where this "nonce" concept on the
    /// expanded key isn't relevant.
    pub fn from_scalar_bytes_with_empty_nonce(scalar_bytes: &[u8]) -> Result<Self, ()> {
        if scalar_bytes.len() != 32 {
            return Err(());
        }
        let mut expanded_secret_key_bytes = [0u8; 64];
        expanded_secret_key_bytes[0..32].copy_from_slice(scalar_bytes);
        // Note - the unwrap is not safe
        Ok(Self(
            ExpandedSecretKey::from_bytes(expanded_secret_key_bytes.as_slice()).unwrap(),
        ))
    }

    /// These seed bytes are hashed with SHA-512 (and twiddled a little to be valid) to create an [`ExpandedSecretKey`].
    /// See the docs on [`ExpandedSecretKey`] for more information.
    pub fn from_seed_bytes(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != 32 {
            return Err(());
        }
        let secret_key = SecretKey::from_bytes(slice).map_err(|_| ())?;
        let expanded_secret_key = ExpandedSecretKey::from(&secret_key);
        Ok(Self(expanded_secret_key))
    }

    /// The resultant bytes are hashed with SHA-512 (and twiddled a little to be valid) to create an [`ExpandedSecretKey`].
    /// See the docs on [`ExpandedSecretKey`] for more information.
    pub fn from_u64(n: u64) -> Result<Self, ()> {
        let mut bytes = [0u8; 32];
        (&mut bytes[32 - 8..32]).copy_from_slice(&n.to_be_bytes());

        Self::from_seed_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::verify_ed25519;
    use radix_engine_interface::crypto::hash;
    use sbor::rust::str::FromStr;

    #[test]
    fn sign_and_verify() {
        let test_sk = "0000000000000000000000000000000000000000000000000000000000000001";
        let test_pk = "4cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29";
        let test_message_hash = hash("Test");
        let test_signature = "cf0ca64435609b85ab170da339d415bbac87d678dfd505969be20adc6b5971f4ee4b4620c602bcbc34fd347596546675099d696265f4a42a16df343da1af980e";
        let sk = Ed25519PrivateKey::from_seed_bytes(&hex::decode(test_sk).unwrap()).unwrap();
        let pk = Ed25519PublicKey::from_str(test_pk).unwrap();
        let sig = Ed25519Signature::from_str(test_signature).unwrap();

        assert_eq!(sk.public_key(), pk);
        assert_eq!(sk.sign(&test_message_hash), sig);
        assert!(verify_ed25519(&test_message_hash, &pk, &sig));
    }
}
