use radix_engine_interface::crypto::{hash, Hash, PublicKey, Signature, SignatureWithPublicKey};
use radix_engine_interface::data::{scrypto_decode, scrypto_encode};
use radix_engine_interface::node::NetworkDefinition;
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
            "16592e5f77fbfa03d5833a7b47d871ce8bbb438ed66d86ddb78b456aa681f1f7",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "0dcb1ff303fc094ed5febb3927daa96f52c4a67fb58b310ac38223d35a7df46d",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "90aad0161af8979caf53b04f9452aeeebaa7b690fb2987eec55b7fe80361215f",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000220e4563647361536563703235366b3101b102f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f0008050021022022010d436c656172417574685a6f6e65002020002022020e4563647361536563703235366b3101b20084a54d128053bda2cd4ce2d64cfd45ed76bcb3cb997b062ba1c2473bab956dcb0e025d5506eb688e5f85fb23056e2df281de45b0db8729e8ac501b81a61066bb0e4563647361536563703235366b3101b200309ba8b862321e3d6913121f4b18703d1818b05ebd53274b25afd442ad074897156bf45732bbea34901d402c5110afd441add7201b5d5ea66cb634f29cb1cadf220e4563647361536563703235366b3101b200c9a9f179f06ca876d83e1c234e1415df619e6ad0dde778b4171ac2d5a02e26733ccc250ed0c11481aaffba0e8e0632157d1e84fbc1292977629bb23c697e8a2b", hex::encode(scrypto_encode(&transaction).unwrap()));
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
            "d340c3caa58e19bcba9312a3757d05023dbf3e0802cea24ccb75ad74a7cffa1e",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "3cdbebbec336a4f3d8d5c729dff4731a8a08ddf21b9dd9f8b3ce57fa1b961951",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "760b52f8c16016ce64d1b56291e2b467017e149d349d7e2c2faed08a416bb476",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000220c45646473614564323535313901b3f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f0008050021022022010d436c656172417574685a6f6e65002020002022020c45646473614564323535313902b34cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29b429ed4b2edd178462d0a7d5234c7637c4fcc0692b3d034a440445afd7c4c2e2938dcc20cbf88256014ac5c15f39bc96bbea7ed7b4d1f30bce738a6621d6dedc030c45646473614564323535313902b37422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674b4c672284361e5df92ce6efbae7c0ed57d09c2018c236461b1873c7ea59393c40b4beda2c8a0c204ad2e705dc83eef5c149f2bff9b11258f6c43e964557add2b07220c45646473614564323535313901b4e38fa93e61941ef6d2efba5d74a03b1bac7cdda956c915a1232562662330bfda57d152f1a24853b078bb7561632119dcb5a70cb87b5893c09189d9847b96870d", hex::encode(scrypto_encode(&transaction).unwrap()));
    }
}
