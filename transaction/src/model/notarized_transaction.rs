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
            "fd8e83317cde93f63038e854b9e827f4978983272104f263c944a040446aff29",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "e38b6db4faf1faa16aa4efff05d35f27e78fc3504a061575be600f647ef03eb5",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "d984bb1f866cd2f7bb365dc662b569c07cfac24be9e064a452d4c06a70a74d3d",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c1002100210021009070107f20a00000000000000000a64000000000000000a0500000000000000110e4563647361536563703235366b3101b102f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010022011010d436c656172417574685a6f6e65002020002011020e4563647361536563703235366b3101b2000ed3b59ca36cb1a2bfc630bfd991b0dd31abf50a4ba4062e65c614fadfed95f6273e4bdda215fedab5dee1590b95d692b844b420f6f6f29d3d78297b410f26bc0e4563647361536563703235366b3101b201a68a18e926b8b7e61e3ffdb75fc9eec4a339bb97da705645af9e56799d8fd1f75d16d4737f55de73d7262debbe47ffe314b1c744b85a741ffab7ade48585e6dc110e4563647361536563703235366b3101b2007d8bc9cf9978a3bc2e48225ec99c4c6a8504b22bb93934caf1099d16218894fa389e118bfe56be80003808993219dabd1ec55f2bc246774f6aff4b85002b2cf8", hex::encode(scrypto_encode(&transaction).unwrap()));
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
            "741f0603d809633969a6ce2fe80ebad6dd177de53a4fa6f2b10e43d4ae154e55",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "1bd5c98bef2a44cbd71798d1a25e6d72b8eee06d83f4710ccf12b59b451ba98d",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "eccb735bc2a113f87ba32a74f68b2d240b3840a67f1d43fd735451c7bd59b85e",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c1002100210021009070107f20a00000000000000000a64000000000000000a0500000000000000110c45646473614564323535313901b3f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f00090500000010022011010d436c656172417574685a6f6e65002020002011020c45646473614564323535313902b34cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29b47d834fb981b3130b739502df6731ec7c8faf544a0268ae544f3c9156754505581033310b3566753aea406becba448299d92d969b3cdae720d5898c57a0eb99090c45646473614564323535313902b37422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674b4aea48b3e6f00852cd93857e3b96927d667739c44d6f9ef650b0b3f9f8ad0ced088183aa68a1625f0a8cb049e765efc53549d20d0b552f53335097edab0346908110c45646473614564323535313901b430e3605c854e15282c3b59528310b4aa25ffdb813e8830d754664819b5182cfb9f98a65c28b777cb14c2e8568f39b0ae237e0fd7653bee3d8904abb6fdfd1600", hex::encode(scrypto_encode(&transaction).unwrap()));
    }
}
