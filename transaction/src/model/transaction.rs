use blob_loader::BlobLoader;
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
            "f5b11490064286d67a047fa548b8eeda0cf50d49cb2d9e238194a75ad13685bd",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "8c9bdce63862e3cdff41f28277f92b1da78cc136c8d096d22f3f4ceafc6d3591",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "483891bcb86bbe7031f7f630ac5c39859b20b4700ec29a88f9dce697b4a3777c",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a0500000000000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010020000003011010000000d000000436c656172417574685a6f6e65000000003030000000003021020000000200000091210000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179892400000006e7e991143625fc74e0efcdab561bc8eb886f9d19292879eb4e85cd6be04ae766b4f38787655fc250f03dd4fa8fffa48541c7771c666d31730b712a0afbfc75d02000000912100000002c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee592400000009f18dc52ff45848652cee6c6771a110ba42cf52c6c98140dbb6c04ef99bf88cb794a91d7adac93ca4efa5a407c558ddf756b53d93a7eb6556f1288bfee3554be9240000000979deb1836c32a9df1eb117d93961480ad096366da4ac7a5a5792279131dbd26165fb3dad0b3c5d4b3a9f0a3a033c98af369d13efe29670e3ee5fdebb4c8a877", hex::encode(scrypto_encode(&transaction)));
    }
}
