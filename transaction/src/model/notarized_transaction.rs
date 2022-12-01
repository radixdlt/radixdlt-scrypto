use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::crypto::{hash, Hash, PublicKey, Signature, SignatureWithPublicKey};
use radix_engine_interface::data::{scrypto_decode, scrypto_encode};
use radix_engine_interface::scrypto;
use sbor::*;

use crate::manifest::{compile, CompileError};
use crate::model::TransactionManifest;

// TODO: add versioning of transaction schema

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct TransactionHeader {
    pub version: u8,
    pub network_id: u8,
    pub start_epoch_inclusive: u64,
    pub end_epoch_exclusive: u64,
    pub nonce: u64,
    pub notary_public_key: PublicKey,
    pub notary_as_signatory: bool,
    pub cost_unit_limit: u32,
    pub tip_percentage: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct TransactionIntent {
    pub header: TransactionHeader,
    pub manifest: TransactionManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct SignedTransactionIntent {
    pub intent: TransactionIntent,
    pub intent_signatures: Vec<SignatureWithPublicKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct NotarizedTransaction {
    pub signed_intent: SignedTransactionIntent,
    pub notary_signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentCreationError {
    CompileErr(CompileError),
    ConfigErr(IntentConfigError),
}

impl From<CompileError> for IntentCreationError {
    fn from(compile_error: CompileError) -> Self {
        IntentCreationError::CompileErr(compile_error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentConfigError {
    MismatchedNetwork { expected: u8, actual: u8 },
}

impl TransactionIntent {
    pub fn new(
        network: &NetworkDefinition,
        header: TransactionHeader,
        manifest: &str,
        blobs: Vec<Vec<u8>>,
    ) -> Result<Self, IntentCreationError> {
        if network.id != header.network_id {
            return Err(IntentCreationError::ConfigErr(
                IntentConfigError::MismatchedNetwork {
                    expected: network.id,
                    actual: header.network_id,
                },
            ));
        }
        Ok(Self {
            header,
            manifest: compile(manifest, &network, blobs)?,
        })
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn hash(&self) -> Result<Hash, EncodeError> {
        Ok(hash(self.to_bytes()?))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        scrypto_encode(self)
    }
}

impl SignedTransactionIntent {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn hash(&self) -> Result<Hash, EncodeError> {
        Ok(hash(self.to_bytes()?))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        scrypto_encode(self)
    }
}

impl NotarizedTransaction {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn hash(&self) -> Result<Hash, EncodeError> {
        Ok(hash(self.to_bytes()?))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        scrypto_encode(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signing::*;

    #[test]
    fn construct_sign_and_notarize_ecdsa_secp256k1() {
        // create a key pair
        let sk1 = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();
        let sk2 = EcdsaSecp256k1PrivateKey::from_u64(2).unwrap();
        let sk_notary = EcdsaSecp256k1PrivateKey::from_u64(3).unwrap();

        // construct
        let intent = TransactionIntent::new(
            &NetworkDefinition::simulator(),
            TransactionHeader {
                version: 1,
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce: 5,
                notary_public_key: sk_notary.public_key().into(),
                notary_as_signatory: false,
                cost_unit_limit: 1_000_000,
                tip_percentage: 5,
            },
            "CLEAR_AUTH_ZONE;",
            Vec::new(),
        )
        .unwrap();

        // sign
        let signature1 = sk1.sign(&intent.to_bytes().unwrap());
        let signature2 = sk2.sign(&intent.to_bytes().unwrap());
        let signed_intent = SignedTransactionIntent {
            intent,
            intent_signatures: vec![signature1.into(), signature2.into()],
        };

        // notarize
        let signature3 = sk_notary.sign(&signed_intent.to_bytes().unwrap());
        let transaction = NotarizedTransaction {
            signed_intent,
            notary_signature: signature3.into(),
        };

        assert_eq!(
            "1ae49f0b12d335f84b0e05a47f4e127a9245efc1b6ce566deac2f79e0ebd3f94",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "671f8366b34e0c043800962d31932946837eb0cb7eee9c6f49454c9d8a8f75f3",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "7cb5877fdcd0a3fb45dfa2431a242811cea6be89e109aac1f387dcec1c24d0a5",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000110e4563647361536563703235366b3101b102f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000021022011010d436c656172417574685a6f6e65002020002011020e4563647361536563703235366b3101b201c8cc0ea2965b0c5666225ec3d7635e54ee65ee5fde2830363eaaaeb1e19396705ede4148a064584d4bd26e145e7f67fca787ec11efcd44f3ef685a8944b3ac910e4563647361536563703235366b3101b201a2a080439b5d33259167e5fd79c5c8f68f4d0868d5afc661b4538e03d0deafcf3a694d4ff265af24178372b0e713d28544a3342c0650a384ef68583b11c62d4b110e4563647361536563703235366b3101b2015bedaecad414dcd4681b0663eb93895eb9e442b47661a7347a895342c54a5ccd425015f41673b13314697a19a656dc9217d9c94a46e42dd21707123d92c01c15", hex::encode(scrypto_encode(&transaction).unwrap()));
    }

    #[test]
    fn construct_sign_and_notarize_eddsa_ed25519() {
        // create a key pair
        let sk1 = EddsaEd25519PrivateKey::from_u64(1).unwrap();
        let sk2 = EddsaEd25519PrivateKey::from_u64(2).unwrap();
        let sk_notary = EddsaEd25519PrivateKey::from_u64(3).unwrap();

        // construct
        let intent = TransactionIntent::new(
            &NetworkDefinition::simulator(),
            TransactionHeader {
                version: 1,
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce: 5,
                notary_public_key: sk_notary.public_key().into(),
                notary_as_signatory: false,
                cost_unit_limit: 1_000_000,
                tip_percentage: 5,
            },
            "CLEAR_AUTH_ZONE;",
            Vec::new(),
        )
        .unwrap();

        // sign
        let signature1 = (sk1.public_key(), sk1.sign(&intent.to_bytes().unwrap()));
        let signature2 = (sk2.public_key(), sk2.sign(&intent.to_bytes().unwrap()));
        let signed_intent = SignedTransactionIntent {
            intent,
            intent_signatures: vec![signature1.into(), signature2.into()],
        };

        // notarize
        let signature3 = sk_notary.sign(&signed_intent.to_bytes().unwrap());
        let transaction = NotarizedTransaction {
            signed_intent,
            notary_signature: signature3.into(),
        };

        assert_eq!(
            "0e002f0b4246f1af55140f8032fefc7c4a8749c090505e79e17f9aa5010f614b",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "c776f5a53ebe12755c8f9998c125a19d4ef40c35f128cbebbf1e4e7b501d55dd",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "d6be41682e90bc0f10bfa60cb7a00ead041a5933a2899c70ada9ee7915a7238c",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000110c45646473614564323535313901b3f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f00090500000021022011010d436c656172417574685a6f6e65002020002011020c45646473614564323535313902b34cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29b47914004ce773a9b5f669ef6210b63f6d9e8c786b5130086af20e4975229ac86abea4768a943b15cf52f834307cb6ea8cd3e3ac20db2e7d5222d11374379db80c0c45646473614564323535313902b37422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674b4b01bf541dcf2c24d6ef3acff8a292dff6ad190702a021a00bdc6edc350f57379ab42f23d78e8b46f2f3cd8d67b520db92df84f574b970d71e12e9a3d988ba60a110c45646473614564323535313901b4e67861c104585a777b2d804a9942e501cc0d3fa6089a227c4138f68259c1a8b1d83c5a944a06677ab5da68acfe5cab1a83534ee48b2c1167b7ed6168983d5d0b", hex::encode(scrypto_encode(&transaction).unwrap()));
    }
}
