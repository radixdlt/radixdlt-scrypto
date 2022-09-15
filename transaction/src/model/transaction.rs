use sbor::*;
use scrypto::buffer::{scrypto_decode, scrypto_encode};
use scrypto::core::NetworkDefinition;
use scrypto::crypto::{hash, Hash, PublicKey, Signature, SignatureWithPublicKey};

use crate::manifest::{compile, CompileError};
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
            "671a87cacf3f359ed6f368c50684fe963567a345eea7382ad931dd8a09d30e5a",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "95299e1b74664150ae319ccade62cc0ed605548c65115f25272c6e4269182f21",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "bcfc92958a504627cfa04b8b1dc9804c5e3a039e5231759258b3be4c6d6e740a",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a0500000000000000110e0000004563647361536563703235366b3101000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010020000003011010000000d000000436c656172417574685a6f6e65000000003030000000003011020000000e0000004563647361536563703235366b310100000092410000000132e68b38e908177113142e58aee6453c34615d1e6d8c48530d5748f6367e27925c55a01c7735fdeda44928a7d015a0e48203f4a39834e73412d150dff092abe70e0000004563647361536563703235366b3101000000924100000000144cbd023cc482c4a39dca0e2d3f2a61bc765c9bbd72e75cf10484a7a3ddf1457fc8bebef15f703cb67e9818e40954a6081f0338e34f17730133050149d93468110e0000004563647361536563703235366b3101000000924100000000245d5ac8983efbf1f4aaf9f369a571d8bdfaf07f1173299998d043252183a1ac7ab0428724dd94e195bdf0092c3e34f78814a7300cbf2ab41131f9c4da69b8ab", hex::encode(scrypto_encode(&transaction)));
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
            "08e1fc53bc3542c9e641bb6335c375bffcbc7bf86c96feecff7b6689568c9d1a",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "d96f9285e8001ebb38cf676aee8009ec471afd7660f1229e512d30790d6e2b06",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "5c4bae2e3713a711c513a096c45a06d695d39188c77d0f2d0d1283cfa6a026a7",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a0500000000000000110c000000456464736145643235353139010000009320000000f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f00090500000010020000003011010000000d000000436c656172417574685a6f6e65000000003030000000003011020000000c0000004564647361456432353531390200000093200000004cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba299440000000c5a8fc87ec5d839b6b9914aeb320a8f6d758e25de9a8ae737f526a9d79df9b179e991fdf877f54ca38ad6177c34ea7cca04b4ffac627d3a224ef095121b7f0070c0000004564647361456432353531390200000093200000007422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674944000000079ffb153e8b19103725e2897dabf6214b5b0c189d285d9dcf4c3785bcc952540966821b07ce5cc4972c47148d4dd26087f6161054a8dd600ba933ea789b3d808110c000000456464736145643235353139010000009440000000b17f1ddea31beeb62266f450a4cdb7d8f2810941bddcf6270cad1b23208160e5c12e2952e9fa5f810d57c1b6a9c15bb9413aeb6f21bfb803c70fc15bef488e02", hex::encode(scrypto_encode(&transaction)));
    }
}
