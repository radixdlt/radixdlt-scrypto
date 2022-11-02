use sbor::*;
use scrypto::buffer::{scrypto_decode, scrypto_encode};
use scrypto::core::NetworkDefinition;
use scrypto::crypto::{hash, Hash, PublicKey, Signature, SignatureWithPublicKey};
use scrypto::data::*;

use crate::manifest::{compile, CompileError};
use crate::model::TransactionManifest;

// TODO: add versioning of transaction schema

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
#[custom_type_id(ScryptoCustomTypeId)]
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
#[custom_type_id(ScryptoCustomTypeId)]
pub struct TransactionIntent {
    pub header: TransactionHeader,
    pub manifest: TransactionManifest,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct SignedTransactionIntent {
    pub intent: TransactionIntent,
    pub intent_signatures: Vec<SignatureWithPublicKey>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
#[custom_type_id(ScryptoCustomTypeId)]
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
    use scrypto::core::NetworkDefinition;

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
            "4244f375d5e286539d191a56d56e7c531eeab7a7c006a2e68c132c7682b8482e",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "007c23e9f3154f626660548f8b6aef5c8117de44827ee94ebdc6caa98c81696e",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "6431780b2732ece77718e1cc501eeede9aa8396fbd02bbab7fdaae57300dfa4f",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a0500000000000000110e0000004563647361536563703235366b3101000000b102f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010020000002011010000000d000000436c656172417574685a6f6e65000000002020000000002011020000000e0000004563647361536563703235366b3101000000b20193cc16da8ef9b6a21ba9bed2184af0a12e631cad187b320ea4656f3282147dbf361d52eb4bee19f38533f30289a77ee3658c002c81fd94c1d8f467a68051508c0e0000004563647361536563703235366b3101000000b201ebe5a13e205bc0315ee3f7e5edc7eb8b34b3847168bcdedef2e5014083702dfd7ba9a886e246a26f14fa971839b64ae9d84d26c555f6caa48327c989ceab41c6110e0000004563647361536563703235366b3101000000b2008626f0e20403ea7c2b7e9f034072298f04522b36b20261072b79ce579353864f0af9cecde0d891424bd7c2d5a9827f6ff9f7e99b2837b6fa44a79e1bc48b5d46", hex::encode(scrypto_encode(&transaction)));
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
            "bfb53a342a53b460984065af0ba6cd9ef13d44f5097c08ff25e3bd4c1942aea3",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "813d5e57f62450b7242b6b5438988f34e898c4f4d98b3abc4c90e24611e08fc5",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "2681b37f92ea0a34736098c862109eebd33aab7f36bf0482325dd2559bceba04",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a0500000000000000110c00000045646473614564323535313901000000b3f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f00090500000010020000002011010000000d000000436c656172417574685a6f6e65000000002020000000002011020000000c00000045646473614564323535313902000000b34cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29b469635e8d3a776d33daee09ac1544ee078a3853433addde53965f14880a6ca4eb205f5c55e6e64e856249083c0ad7d0b8e8613c9ce55e0d3753d5ae571bc2db040c00000045646473614564323535313902000000b37422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674b439255e6f3df15871aa96e5ef1f9042a9c7215a0817613707bd6d31675e5f0cadc1c1d3ffd8714c7768aac99c480df244b703277f2981c58a4eb571a09b608506110c00000045646473614564323535313901000000b42f3602c8afbaf698cdff70a5b8568d3ab9f6172f9a7bf0bec4101c8de32369a6fe838ef8b89ed2240aa26a2043e3afa68ee7119cb770ea90cc26ccff1100c50e", hex::encode(scrypto_encode(&transaction)));
    }
}
