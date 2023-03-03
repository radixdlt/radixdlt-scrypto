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
            "81b06974e2e359ed9ceb16ebbcaae7d256f19d6b13bbef6d44698c203e3d14e7"
        );
        assert_eq!(
            transaction.signed_intent.hash().unwrap().to_string(),
            "b00287b46fc52651e5db2913b8886051d4ca6fd851852697fe4e43fd85608737"
        );
        assert_eq!(
            transaction.hash().unwrap().to_string(),
            "d5a09086b67552347a0d34a1729e87a8b75e992be95f1c5817e51484b208c3d0",
        );
        assert_eq!(hex::encode(manifest_encode(&transaction).unwrap()), "4d2102210221022109070107f20a00000000000000000a64000000000000000a050000000000000022000120072102f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f0008050021022022010900202000202202000121012007410080f917243985f64db4fe011a1758c223ed6866400853fede94bc6cef35ca98bb2bca3164e6b4b6f08fdf7d39e067b9c5b3eeb5fdff66e4e095c46baeba12061c000121012007410157551ffc806d3abf40d0be5673b9102fb83bbee9101ee4e674a6ffe1f6d8e85d17f2a41f1092067dd9a2af36c1cb5a46fb904f41e4b8d9698f833c1d6f806dd2220001210120074100f4859a1bd63b649548b0e3dd5c0c52e1446ddec18a40cab19acbd24c9e8bce1655f802e67e6ae8ad4cad34849b17c557d0f95ff660423564315da147ec7f37eb");
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
            "bc01cff3978ff43d881ad01f4a8be3ff21582de55a1631b3384d27b66dd28d8d"
        );
        assert_eq!(
            transaction.signed_intent.hash().unwrap().to_string(),
            "69076ac02d6958a1b07c49382b14e0e037951653774068fa424416138d206fda",
        );
        assert_eq!(
            transaction.hash().unwrap().to_string(),
            "b4924b71bd03aa69e0f796ece997b7fbb9e501dd1298aeec57d802dcac8ef8d3"
        );
        assert_eq!(  hex::encode(manifest_encode(&transaction).unwrap()), "4d2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000220101200720f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f000805002102202201090020200020220201022007204cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba292101200740efca0f6a4e2a782eef27da5bce284e93ca3b993774b1866510a8888b7100162f9f30405ab6ded0cbe45262dc9b7c514d89d610e779e36659c53bcbf83182760501022007207422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe26742101200740de01a711a96684c9e705583680f44729cccab96f5dde370225003382e86e66660ea216d020d1b296ae8137ff072065e850c46fde287772139fabcebfd38cc10e2201012101200740622d0f7f90e7cc3d5550bcb763c3f52a14ee19e6dd9239673682b57c1cfb33e4cffce2b3a4b13d7a4ea987f214d539e2621bd5f9b0cca054d0581e06287d5b0e");
    }
}
