use crate::ecdsa_secp256k1::EcdsaSecp256k1Signature;
use crate::eddsa_ed25519::EddsaEd25519Signature;
use crate::manifest::{compile, CompileError};
use crate::model::TransactionManifest;
use radix_engine_interface::crypto::*;
use radix_engine_interface::data::manifest::*;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::*;
use sbor::*;

// TODO: add versioning of transaction schema

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
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

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct TransactionIntent {
    pub header: TransactionHeader,
    pub manifest: TransactionManifest,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct SignedTransactionIntent {
    pub intent: TransactionIntent,
    pub intent_signatures: Vec<SignatureWithPublicKey>,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NotarizedTransaction {
    pub signed_intent: SignedTransactionIntent,
    pub notary_signature: Signature,
}

/// Represents any natively supported signature.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "signature")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum Signature {
    EcdsaSecp256k1(EcdsaSecp256k1Signature),
    EddsaEd25519(EddsaEd25519Signature),
}

/// Represents any natively supported signature, including public key.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum SignatureWithPublicKey {
    EcdsaSecp256k1 {
        signature: EcdsaSecp256k1Signature,
    },
    EddsaEd25519 {
        public_key: EddsaEd25519PublicKey,
        signature: EddsaEd25519Signature,
    },
}

impl SignatureWithPublicKey {
    pub fn signature(&self) -> Signature {
        match &self {
            SignatureWithPublicKey::EcdsaSecp256k1 { signature } => signature.clone().into(),
            SignatureWithPublicKey::EddsaEd25519 { signature, .. } => signature.clone().into(),
        }
    }
}

impl From<EcdsaSecp256k1Signature> for Signature {
    fn from(signature: EcdsaSecp256k1Signature) -> Self {
        Self::EcdsaSecp256k1(signature)
    }
}

impl From<EddsaEd25519Signature> for Signature {
    fn from(signature: EddsaEd25519Signature) -> Self {
        Self::EddsaEd25519(signature)
    }
}

impl From<EcdsaSecp256k1Signature> for SignatureWithPublicKey {
    fn from(signature: EcdsaSecp256k1Signature) -> Self {
        Self::EcdsaSecp256k1 { signature }
    }
}

impl From<(EddsaEd25519PublicKey, EddsaEd25519Signature)> for SignatureWithPublicKey {
    fn from((public_key, signature): (EddsaEd25519PublicKey, EddsaEd25519Signature)) -> Self {
        Self::EddsaEd25519 {
            public_key,
            signature,
        }
    }
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
        manifest_decode(slice)
    }

    pub fn hash(&self) -> Result<Hash, EncodeError> {
        Ok(hash(self.to_bytes()?))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(self)
    }
}

impl SignedTransactionIntent {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        manifest_decode(slice)
    }

    pub fn hash(&self) -> Result<Hash, EncodeError> {
        Ok(hash(self.to_bytes()?))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(self)
    }
}

impl NotarizedTransaction {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        manifest_decode(slice)
    }

    pub fn hash(&self) -> Result<Hash, EncodeError> {
        Ok(hash(self.to_bytes()?))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        manifest_encode(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ecdsa_secp256k1::EcdsaSecp256k1PrivateKey, eddsa_ed25519::EddsaEd25519PrivateKey};

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

        let intent_hash = intent.hash().unwrap();

        // sign
        let signature1 = sk1.sign(&intent_hash);
        let signature2 = sk2.sign(&intent_hash);
        let signed_intent = SignedTransactionIntent {
            intent,
            intent_signatures: vec![signature1.into(), signature2.into()],
        };

        let signed_intent_hash = signed_intent.hash().unwrap();

        // notarize
        let signature3 = sk_notary.sign(&signed_intent_hash);
        let transaction = NotarizedTransaction {
            signed_intent,
            notary_signature: signature3.into(),
        };

        assert_eq!(
            transaction.signed_intent.intent.hash().unwrap().to_string(),
            "0e73517418d922cbe5f2e07a5e31b3f40b64310ffce0856c10da5cf4a13982c2"
        );
        assert_eq!(
            transaction.signed_intent.hash().unwrap().to_string(),
            "86fa0c6934d05f1be28c0a886713a4da3a46103fda33e29b2b6c7aa1a1db8b95"
        );
        assert_eq!(
            transaction.hash().unwrap().to_string(),
            "9b6427d4b565b2cdb50fc777e1183c587cbf0323086c7b3d3a0c83e02f073516",
        );
        assert_eq!(hex::encode(manifest_encode(&transaction).unwrap()), "4d2102210221022109070107f20a00000000000000000a64000000000000000a050000000000000022000120072102f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f000805002102202201120020200020220200012101200741004ad8fee4917dbb21039f8c3ad2f1bd20fe07e6343dada09898902d7ed2ded8ea58ce8ac8762e0820fbf0eded651bd55b573abe6e965f6fdfec102e4daa163979000121012007410103e6bf0755bb23533f9461b97d94541f6d2aa95c044fc2dd5c8c465541ce81d450a22324d67f56e5adf12fef4266058164cbd164504d519f058ec88b6adec933220001210120074101309d0f289ad800ec340349b633aa855b9f3ef4f27e4fa05b1f1db12b958038cd3e755cc203abbde44d7378fea6ba99262c340cacfe6036ab11d6d9b0777adf85");
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

        let intent_hash = intent.hash().unwrap();

        // sign
        let signature1 = (sk1.public_key(), sk1.sign(&intent_hash));
        let signature2 = (sk2.public_key(), sk2.sign(&intent_hash));
        let signed_intent = SignedTransactionIntent {
            intent,
            intent_signatures: vec![signature1.into(), signature2.into()],
        };

        // notarize
        let signed_intent_hash = hash(signed_intent.to_bytes().unwrap());

        let signature3 = sk_notary.sign(&signed_intent_hash);
        let transaction = NotarizedTransaction {
            signed_intent,
            notary_signature: signature3.into(),
        };

        assert_eq!(
            transaction.signed_intent.intent.hash().unwrap().to_string(),
            "24df74ea45d77fd9a11507d3ce67f36dd988aa94066f0669eb40e26bfd852af9"
        );
        assert_eq!(
            transaction.signed_intent.hash().unwrap().to_string(),
            "8da9be73d777f49529b31af50287db5cd8ff7bb2c954961984267bc27448afcf",
        );
        assert_eq!(
            transaction.hash().unwrap().to_string(),
            "4f1efc84b25ee241eb452d1cee17e626a167a55373a15ac84d5598906c00ff85"
        );
        assert_eq!(hex::encode(manifest_encode(&transaction).unwrap()), "4d2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000220101200720f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f000805002102202201120020200020220201022007204cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29210120074068c249a8e2d64204ed2186e001df9af0b1038ae677c1d9ae0472e74a6e87f61c359c1b1b36db262efb3e8b1b907aa2e448ef866126ce65fefa5afe6f042c540701022007207422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe26742101200740220333daf482f6a73f2984235083e81b5af9997aa8574329f8ed06622ee25cb4783bb7d182767c80b0a5668a34d18aa74d2da7603e9ae7fbced21a7eccfa1103220101210120074078dfb079a4ccad6c0c7e7d19594facade502ca2fff456e950cfd2fd39c00f06f9c09ca7e71fbba5745baec763f28d7855fddd3d27427d3388bd70476b4687308");
    }
}
