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
            "e8caf1bc39ab01841f89995a2e4b76ceb40ca12fd97101309b6fece3d739abb3"
        );
        assert_eq!(
            transaction.signed_intent.hash().unwrap().to_string(),
            "bac2aeb113b1ae8bd8b0e6940cf729f95115192f4e4b1ec390446f2bd8fd7ce5"
        );
        assert_eq!(
            transaction.hash().unwrap().to_string(),
            "27acc1157b68b81eece7b9622999eaef4581ab3f82c49749e869ece1e8c60f4c",
        );
        assert_eq!(hex::encode(manifest_encode(&transaction).unwrap()), "4d2102210221022109070107f20a00000000000000000a64000000000000000a050000000000000022000120072102f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00080500210220220108002020002022020001210120074101d33e6ecb1414c106b7e3de5c3b2724f298b77cad4f8cbfa34f5f1970c5bc2f291b449fec4cc1cd7757107313e7b23691a140cf212b540cc3b32b02e7087dc0780001210120074100129dff9f0661c6df20a041b246e73c5304456c2ceeb146e2b1dc0ed7525bcf063e31eac0b27a12895b1c2d1b21c729a38cfe84588df6f3dd714f61ac722156bb220001210120074101b9279e19d833b2354040d1ce4289f7a180821c4a1e32a7f8758b5c34250d2e857c6d0254cdd3b13d773c77397490a31e3c4302e8cb6591ff1fcb59191de80044");
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
            "a651deb0b3fab8a4b5d761d4b2e474c5cd0af57af7f06233c39f60f5df853d7e"
        );
        assert_eq!(
            transaction.signed_intent.hash().unwrap().to_string(),
            "210d705dd133bf768a55437d797023bcab8af4dbf14eee1b122df330bd764798",
        );
        assert_eq!(
            transaction.hash().unwrap().to_string(),
            "b5bb9806bc0f561f12d28176847e75300b96e7e4a231c1911b7b067212214b2c"
        );
        assert_eq!(hex::encode(manifest_encode(&transaction).unwrap()), "4d2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000220101200720f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f000805002102202201080020200020220201022007204cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba2921012007402251ab02a805b76f3c33b5aa736877dcf1cdc409eff49d39ce389098f6a7ff226d0f6d406b406fd2953657435633290afbaefecf8eb919be67d6191358347d0d01022007207422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe267421012007402b170c3553afc14417ffdf6d0b4981f370b3ab715b5d687fda0caa49f7b67d85b1c8082305b7e19b2b9d1a8984961faf290684a530dae66e7db132f1093f30092201012101200740b5f47e7c233be29f1daacfe7476c69c8e654062285f0efbe4fd66901d4e6e3b608d9df85b2a95e4e7ca37645f1c7f36727d9170da721b3a752da8121d7612207");
    }
}
