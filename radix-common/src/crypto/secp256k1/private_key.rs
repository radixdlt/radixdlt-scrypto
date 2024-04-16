use super::Secp256k1Signature;
use crate::internal_prelude::*;
use ::secp256k1::{All, Message, PublicKey, Secp256k1, SecretKey};
use zeroize::{DefaultIsZeroes, Zeroize};

lazy_static::lazy_static! {
    pub(crate) static ref SECP256K1_CTX: Secp256k1<All> = secp256k1::Secp256k1::new();
}

#[derive(Copy, Clone)]
pub struct SecretKeyWrapper(SecretKey);
impl Default for SecretKeyWrapper {
    fn default() -> Self {
        let mut data = [0u8; secp256k1::constants::SECRET_KEY_SIZE];
        data[secp256k1::constants::SECRET_KEY_SIZE - 1] = 1;
        Self(SecretKey::from_slice(&data).unwrap())
    }
}
impl DefaultIsZeroes for SecretKeyWrapper {}

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct Secp256k1PrivateKey(SecretKeyWrapper);

impl Secp256k1PrivateKey {
    pub const LENGTH: usize = secp256k1::constants::SECRET_KEY_SIZE;

    pub fn public_key(&self) -> Secp256k1PublicKey {
        Secp256k1PublicKey(PublicKey::from_secret_key(&SECP256K1_CTX, &self.0 .0).serialize())
    }

    pub fn sign(&self, msg_hash: &impl IsHash) -> Secp256k1Signature {
        let m =
            Message::from_digest_slice(msg_hash.as_ref()).expect("Hash is always a valid message");
        let signature = SECP256K1_CTX.sign_ecdsa_recoverable(&m, &self.0 .0);
        let (recovery_id, signature_data) = signature.serialize_compact();

        let mut buf = [0u8; Secp256k1Signature::LENGTH];
        buf[0] = recovery_id.to_i32() as u8;
        buf[1..].copy_from_slice(&signature_data);
        Secp256k1Signature(buf)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0 .0.secret_bytes().to_vec()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    pub fn from_hex(s: &str) -> Result<Self, ()> {
        hex::decode(s)
            .map_err(|_| ())
            .and_then(|v| Self::from_bytes(&v))
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != Secp256k1PrivateKey::LENGTH {
            return Err(());
        }
        Ok(Self(SecretKeyWrapper(
            SecretKey::from_slice(slice).map_err(|_| ())?,
        )))
    }

    pub fn from_u64(n: u64) -> Result<Self, ()> {
        let mut bytes = [0u8; Secp256k1PrivateKey::LENGTH];
        (&mut bytes[Secp256k1PrivateKey::LENGTH - 8..Secp256k1PrivateKey::LENGTH])
            .copy_from_slice(&n.to_be_bytes());

        Ok(Self(SecretKeyWrapper(
            SecretKey::from_slice(&bytes).map_err(|_| ())?,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::str::FromStr;

    #[test]
    fn sign_and_verify() {
        let test_sk = "0000000000000000000000000000000000000000000000000000000000000001";
        let test_pk = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
        let test_message_hash = hash("Test");
        let test_signature = "00eb8dcd5bb841430dd0a6f45565a1b8bdb4a204eb868832cd006f963a89a662813ab844a542fcdbfda4086a83fbbde516214113051b9c8e42a206c98d564d7122";
        let sk = Secp256k1PrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
        let pk = Secp256k1PublicKey::from_str(test_pk).unwrap();
        let sig = Secp256k1Signature::from_str(test_signature).unwrap();

        assert_eq!(sk.public_key(), pk);
        assert_eq!(sk.sign(&test_message_hash), sig);
        assert!(verify_secp256k1(&test_message_hash, &pk, &sig));
    }

    #[test]
    fn default_value() {
        let key: SecretKeyWrapper = SecretKeyWrapper::default();
        assert_eq!(
            key.0.secret_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1
            ]
        );
    }

    #[test]
    fn verify_zeroize() {
        let bytes = "4fd3fb62d6b7a4749f75d56d06b0aea1ec2c2a6986d2bfa975d7891585590fea";
        let mut key = Secp256k1PrivateKey::from_bytes(&hex::decode(bytes).unwrap()).unwrap();
        key.zeroize();

        assert_eq!(
            key.0 .0.secret_bytes(),
            SecretKeyWrapper::default().0.secret_bytes()
        );
    }
}
