use radix_engine_lib::core::NetworkDefinition;
use sbor::*;
use scrypto::buffer::{scrypto_decode, scrypto_encode};
use scrypto::crypto::{hash, Hash, PublicKey, Signature, SignatureWithPublicKey};

use crate::manifest::{compile, CompileError};
use crate::model::TransactionManifest;

// TODO: add versioning of transaction schema

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
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

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct TransactionIntent {
    pub header: TransactionHeader,
    pub manifest: TransactionManifest,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct SignedTransactionIntent {
    pub intent: TransactionIntent,
    pub intent_signatures: Vec<SignatureWithPublicKey>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
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

    pub fn hash(&self) -> Hash {
        hash(self.to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        scrypto_encode(self)
    }
}

impl SignedTransactionIntent {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn hash(&self) -> Hash {
        hash(self.to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        scrypto_encode(self)
    }
}

impl NotarizedTransaction {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        scrypto_decode(slice)
    }

    pub fn hash(&self) -> Hash {
        hash(self.to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        scrypto_encode(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signing::*;
    use scrypto::buffer::scrypto_encode;

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
        let signature1 = sk1.sign(&intent.to_bytes());
        let signature2 = sk2.sign(&intent.to_bytes());
        let signed_intent = SignedTransactionIntent {
            intent,
            intent_signatures: vec![signature1.into(), signature2.into()],
        };

        // notarize
        let signature3 = sk_notary.sign(&signed_intent.to_bytes());
        let transaction = NotarizedTransaction {
            signed_intent,
            notary_signature: signature3.into(),
        };

        assert_eq!(
            "a864aea51bacfd73df654c381ccc8119f32e12135667ec947eb4832448ebbb0a",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "0e3d98321860675881cedcaca53c14607dc1b53775fb329587d154cef785a118",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "e9368400ca20971deb8427642e645443a3ad8394fbb822be05943f7c42f9eda5",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a0500000000000000110e0000004563647361536563703235366b3101000000b12100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010020000003011010000000d000000436c656172417574685a6f6e65000000003030000000003011020000000e0000004563647361536563703235366b3101000000b24100000000938eef99236cc8471627a28ec966f8789bf2caf0014986486104f454766e8f2072592efd93bc66fb48410aa850bbcf50d0fc5ee1fd4ba007238d3c6691f98b040e0000004563647361536563703235366b3101000000b2410000000072e91c612e078e3791beb348fa3a79f46635972c8ffb191fd326d5e329564d814bd11112b044bf68edd991d4087a8a5ea4e62074113ce40ad85defec358c152f110e0000004563647361536563703235366b3101000000b241000000009cb5883f6cf96feca008971afed8f98e0d88816f0bca34323f8e991931155a8f1982a7612be361693a2421d4326319863a5b8583a90036bbf3e00965593ec872", hex::encode(scrypto_encode(&transaction)));
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
        let signature1 = (sk1.public_key(), sk1.sign(&intent.to_bytes()));
        let signature2 = (sk2.public_key(), sk2.sign(&intent.to_bytes()));
        let signed_intent = SignedTransactionIntent {
            intent,
            intent_signatures: vec![signature1.into(), signature2.into()],
        };

        // notarize
        let signature3 = sk_notary.sign(&signed_intent.to_bytes());
        let transaction = NotarizedTransaction {
            signed_intent,
            notary_signature: signature3.into(),
        };

        assert_eq!(
            "31a892ae803a61d090267f07770a93bd2a5c9f78e4e2d99cfb89bb350f0b9798",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "0c7356256071ee2245aeaa3c80e8887701eedde21f68df24aa0ffb4db3e8aad9",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "5d30b015b0cef8a1ac3990f23c91145a61c56fa3317f804c2afebd3894fd3950",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a0500000000000000110c00000045646473614564323535313901000000b320000000f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f00090500000010020000003011010000000d000000436c656172417574685a6f6e65000000003030000000003011020000000c00000045646473614564323535313902000000b3200000004cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29b44000000031861eacaf5ec46409a1bf3a5885cffa17b552881b518b616b189ce0793e92de06174c0bcfb523673351191e143e0df6404721fb2bb835565c061ef87aa0050a0c00000045646473614564323535313902000000b3200000007422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674b440000000a38144ea7a267a62af4d76731cd26cd883e6881a9c89820544748d863f9c69aa5526e8686329c080379f69a4424e48ffd9bb49fc9f4c61e9624a178d76f7e80b110c00000045646473614564323535313901000000b4400000002227e05350e08b7d091de60867e0b4cc19f8fc4eee32fe3178c55cee0abab3a2ee9f6d542b2ea32a9f43f2bcf78b2641c3bc801878dc6a5593d66e0e6c356706", hex::encode(scrypto_encode(&transaction)));
    }
}
