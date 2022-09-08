use blob_loader::BlobLoader;
use sbor::*;
use scrypto::buffer::{scrypto_decode, scrypto_encode};
use scrypto::core::NetworkDefinition;
use scrypto::crypto::{hash, Hash, PublicKey, Signature, SignatureWithPublicKey};

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
    pub notary_public_key: PublicKey,
    pub notary_as_signatory: bool,
    pub cost_unit_limit: u32,
    pub tip_percentage: u32,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct TransactionManifest {
    pub instructions: Vec<Instruction>,
    pub blobs: Vec<Vec<u8>>,
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
    fn construct_sign_and_notarize_ecdsa() {
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
                notary_public_key: sk_notary.public_key().into(),
                notary_as_signatory: false,
                cost_unit_limit: 1_000_000,
                tip_percentage: 5,
            },
            "CLEAR_AUTH_ZONE;",
            &InMemoryBlobLoader::default(),
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
            "a9785847e3d454452fba1a5eef56682ad46e92e9b830fa4efd19930808b41d32",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "019b9cbecff5a33735b4e866d363e8173a0ff17bbd88608935c9c43bd2839ee1",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "a0cd1dc05398294918e50c4a3df6020d51f09e72c1edfb715c23ac3f60d2f25f",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a05000000000000001105000000456364736101000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010020000003011010000000d000000436c656172417574685a6f6e65000000003030000000003011020000000500000045636473610100000092410000000017c228442a5b312fd580d4c2936a6b044dcc363734becc74473dff6f799387ca0901054dc526a21a4eb779f5fdaa82fea3cf214913c5b69fd63c087e61d6154905000000456364736101000000924100000001aff7b2bffa056704776f6e27a36a14f2e5f357004fbdd1d649bc99c7c3d66ba273b6e63fa4239e87f048ca61f510655302ab5231e36d7c994f2180956a588b771105000000456364736101000000924100000000a335aeb9b8ed658c6945441fbcaca6557adb3181285ccdb44ceb0d76627ec3994912277d7637e2d2c51c27b6b9519cb272646ee4a3721054e57c7414c12b47a8", hex::encode(scrypto_encode(&transaction)));
    }

    #[test]
    fn construct_sign_and_notarize_ed25519() {
        // create a key pair
        let sk1 = Ed25519PrivateKey::from_u64(1).unwrap();
        let sk2 = Ed25519PrivateKey::from_u64(2).unwrap();
        let sk_notary = Ed25519PrivateKey::from_u64(3).unwrap();

        // construct
        let intent = TransactionIntent::new(
            &NetworkDefinition::local_simulator(),
            TransactionHeader {
                version: 1,
                network_id: NetworkDefinition::local_simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce: 5,
                notary_public_key: sk_notary.public_key().into(),
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
            intent_signatures: vec![signature1.into(), signature2.into()],
        };

        // notarize
        let signature3 = sk_notary.sign(&signed_intent.to_bytes());
        let transaction = NotarizedTransaction {
            signed_intent,
            notary_signature: signature3.into(),
        };

        assert_eq!(
            "938e5062ad7d881a7143154690a2db0ee831e10fbc0ca3f9080ec207c49a1e29",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "3da3d6ecc2b1217e1a30244f762ec5ae0b0c9670597d08e786fb26f5247909fa",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "9c2308b119cac8193a7959027e3f532365f68fdff97141d4627048507e339764",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a0500000000000000110700000045643235353139010000009320000000f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f00090500000010020000003011010000000d000000436c656172417574685a6f6e650000000030300000000030110200000007000000456432353531390200000093200000004cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba299440000000495ee3df6061866b80f02e247b60960072eaa15c78fad375032831e4244e73a4ccf14b216c6b75b616a1a0745a933c7d0b910a3f2462e6fc5789b7a8f017b70e07000000456432353531390200000093200000007422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674944000000047a0fea579b22e5c04786d80ae275cce09ef2408bf8a5190fe607cd68f78498640ef570b263143ba16afe93dbad0dc6e69b7977087c7bbe8a76f023c86d13007110700000045643235353139010000009440000000fdf5c189ad7a7f751c5cde08a050d3e846189f27037cf9eebab612da965c4278e5d4071586c968c78c0ca4c05c32a8449e7046cbed25a9a3ba3997b17dae4c0c", hex::encode(scrypto_encode(&transaction)));
    }
}
