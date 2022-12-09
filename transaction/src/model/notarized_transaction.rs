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
    pub tip_percentage: u16,
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
            "bb3c6a094e369544f198062028764091b4633da6378df0f1855352d61d3bd63e",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "8668036d946b686f7062aca3b6b0ebc68958d273c72e5623f907c19fb7d026ae",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "c4e44f6913ccb160f1da5f58120f8053d9117e9ee8e8b94cb88633c08b8bfccc",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000110e4563647361536563703235366b3101b102f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f0008050021022011010d436c656172417574685a6f6e65002020002011020e4563647361536563703235366b3101b20009d9cb7f2f6f8b736d0ce8cb6d366716ef96160bebd9dc0660ecf44e3d905b8060bf1cd1351096e528f0ec7071386b043c6a7e0f468ea966688841b19c4b5a0c0e4563647361536563703235366b3101b20114b651e33ca6886d0f3b2bcf668504b4fc1ebf1ae4fe1b3b15b0ae8520c21dbd6cf2a6fdabe6d5e58c716fdd0327fc0a813610ede14d975a28a4195a5b9ed1cb110e4563647361536563703235366b3101b20160dab46382fdec2e4f48843b9d1995dd8610378d8855a67580c79bad77a5f21425a2b425c2a1d5cb78a9b91de54ea732c33ea5982ced9d7ea0fb431055721373", hex::encode(scrypto_encode(&transaction).unwrap()));
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
            "23f13c562ca9b65758cb2c9c4bcbfa9b53332425b15ffa85478da9dbc32683f7",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "a39c1c7050e635c0fea7675fb56f08a5abde5a70683989077dd0fd663310e7f7",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "1cfeae74c450789061c1c5e24638090ffa49aabbd374ac3b8dc0aa8905807f1d",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000110c45646473614564323535313901b3f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f0008050021022011010d436c656172417574685a6f6e65002020002011020c45646473614564323535313902b34cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29b4156c17ecb586e60d37b8f5b62775e8155bfe8b5193e3c578c7d6a0a4333030f80d51569125cd5ad93f3ddf8c7efd156a898679a2e9548e80aa0e3b7b0a5fd7070c45646473614564323535313902b37422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674b4a15125b15101e881b44bd9f196dbc7f893c9e85fd66ef7c0fcf0841bb5ad634ff8369723f5aa18412ef5b710ac3c122c34df8c97db8e01469a1561f16e38830d110c45646473614564323535313901b44572adc28f1d2e8af3713cdedc3f7deb8a4a0e7d9e26113af3ead2ac006099c342ce7917a250748bd09f8c618acac765699e1a270e1116451ebe1e990cf7590a", hex::encode(scrypto_encode(&transaction).unwrap()));
    }
}
