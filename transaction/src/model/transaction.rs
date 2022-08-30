use blob_loader::BlobLoader;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::{scrypto_decode, scrypto_encode};
use scrypto::core::NetworkDefinition;
use scrypto::crypto::{hash, EcdsaPublicKey, EcdsaSignature, Hash};

use crate::manifest::{blob_loader, compile, CompileError};
use crate::model::Instruction;

// TODO: add versioning of transaction schema

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct TransactionHeader {
    pub version: u8,
    pub network_id: u8,
    pub start_epoch_inclusive: u64,
    pub end_epoch_exclusive: u64,
    pub nonce: u64,
    pub notary_public_key: EcdsaPublicKey,
    pub notary_as_signatory: bool,
    pub cost_unit_limit: u32,
    pub tip_percentage: u32,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct TransactionManifest {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct TransactionIntent {
    pub header: TransactionHeader,
    pub manifest: TransactionManifest,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct SignedTransactionIntent {
    pub intent: TransactionIntent,
    pub intent_signatures: Vec<(EcdsaPublicKey, EcdsaSignature)>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct NotarizedTransaction {
    pub signed_intent: SignedTransactionIntent,
    pub notary_signature: EcdsaSignature,
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
    pub fn new<T: BlobLoader>(
        network: &NetworkDefinition,
        header: TransactionHeader,
        manifest: &str,
        blob_loader: &T,
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
            manifest: compile(manifest, &network, blob_loader)?,
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
    use blob_loader::InMemoryBlobLoader;
    use scrypto::buffer::scrypto_encode;
    use scrypto::core::NetworkDefinition;

    #[test]
    fn construct_sign_and_notarize() {
        // create a key pair
        let sk1 = EcdsaPrivateKey::from_u64(1).unwrap();
        let sk2 = EcdsaPrivateKey::from_u64(2).unwrap();
        let sk_notary = EcdsaPrivateKey::from_u64(3).unwrap();

        // construct
        let intent = TransactionIntent::new(
            &NetworkDefinition::local_simulator(),
            TransactionHeader {
                version: 1,
                network_id: NetworkDefinition::local_simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce: 5,
                notary_public_key: sk_notary.public_key(),
                notary_as_signatory: false,
                cost_unit_limit: 1_000_000,
                tip_percentage: 5,
            },
            "CLEAR_AUTH_ZONE;",
            &InMemoryBlobLoader::default(),
        )
        .unwrap();

        // sign
        let signature1 = (sk1.public_key(), sk1.sign(&intent.to_bytes()));
        let signature2 = (sk2.public_key(), sk2.sign(&intent.to_bytes()));
        let signed_intent = SignedTransactionIntent {
            intent,
            intent_signatures: vec![signature1, signature2],
        };

        // notarize
        let signature3 = sk_notary.sign(&signed_intent.to_bytes());
        let transaction = NotarizedTransaction {
            signed_intent,
            notary_signature: signature3,
        };

        assert_eq!(
            "2949b45e2dd2b47b8a0cee87aa1a62395bdf59e6c5d0078900f1743c1969120d",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "ebcafd203a5c768fa32fdae77ea8d4f151616367108aeba7efa3f9320816b6cf",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "76c5f5b4600ba65b533f216f2ccaf580a8d08f1a938685f581b8214eeefb1c11",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a0500000000000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010010000003011010000000d000000436c656172417574685a6f6e65000000003021020000000200000091210000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f817989240000000350ab31365dae6e336fc8b5e5bf1e01712b934b18408d75754ddaea1f6ba283f07fb1e6b629a39400384cadb57a8c68aebabd004fd40e4a9d88359ae0e7ddb9c02000000912100000002c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee592400000000032e58635c3ca75b2447944eb5882fe4a1811123b9db585def5d11766814c463fa326a87e44b135ad108bf49a5a2a41b64ab01a2899e36910ddeaaf3d6730f29240000000e107bfa1fef13a6b210db368ce60cc002bd00184840b0ce7f335481a8292d28d0760ec74e189dc9d9ece00389b70935006d1624695ce6b47412a0beb2badac9b", hex::encode(scrypto_encode(&transaction)));
    }
}
