use radix_engine_interface::crypto::*;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode};
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::*;
use sbor::*;

use crate::manifest::{compile, CompileError};
use crate::model::TransactionManifest;

// TODO: add versioning of transaction schema

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct TransactionIntent {
    pub header: TransactionHeader,
    pub manifest: TransactionManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct SignedTransactionIntent {
    pub intent: TransactionIntent,
    pub intent_signatures: Vec<SignatureWithPublicKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScryptoSbor)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScryptoSbor)]
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
            "641c247aa64c7f8f6706f365efcb2898b43893f006b748c2de46929756c08f5e",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "fea2075148785705d52a383176e9ded4b0ee8481ae82276bb7dec04c48f4eb05",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "c1dce0e0b3cba17e85575736b25874f2185b7c761863bf52d1b49702ef23b228",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000220001b102f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00080500210220220109002020002022020001b20102115096107f734faceb87b1e3a6d678c3ddd407fb646a219035d79203caa348372be680c53eedc18160bf0dc8ca207c0082d4a35f40a8d4bad036f833c8bbae0001b200f0b7e0b49a44ed7f72cd8dcdbe267ebdde521935ea13273a4d0bee4ec68a7db658e56c9d2821f45ea78856c489c1c9eacf8fbf804eddeb77b4bd901916d4d4c1220001b200610bd7a14a6910490911f0b3de5a7b77669b224ff7ab5ffb62ceac4e2eb443731c3b5cd5d544db4f5671cf2b24b117ebd33122bc40b54ffb749c11e24dfa22c2", hex::encode(scrypto_encode(&transaction).unwrap()));
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
            "9e6a155f408a445b7f5249bfc2df9dbc4f94b78a1b81f170d2dc0f91529cc212",
            transaction.signed_intent.intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "12f65ffad398eb3e927c68787cdc34efa61f57c038798e0cb465c34aed38cb49",
            transaction.signed_intent.hash().unwrap().to_string()
        );
        assert_eq!(
            "69dbbf6de80d2508410a92b2840c913a989a43884a3ce376660a059ec921d1f4",
            transaction.hash().unwrap().to_string()
        );
        assert_eq!("5c2102210221022109070107f20a00000000000000000a64000000000000000a0500000000000000220101b3f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f00080500210220220109002020002022020102b34cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29b4637acc3086579a7951f3339954a7c819082df0f2aefcf5bee6545886a212fb11deb3ea77cd45b20408980f31ce16bffeeb3e28705c601a4ae565e44a9ac120040102b37422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674b486aefe97b661ab550e5c65bb5bffbb27ad8a4b3a3936c9aecc7a308996087f9a624a3a0293aeacd3c3ec489054719e3854aad040e1ec0378013563c5db309106220101b4e224ace8bcd124de7af8da24cd897bc50ac8d26758020ae7735fa2374983be491de610450bc628e8c2e19fe386762f3573382f82607ac040fe72640c8ffb070a", hex::encode(scrypto_encode(&transaction).unwrap()));
    }
}
