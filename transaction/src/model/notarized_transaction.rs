use sbor::*;
use scrypto::buffer::{scrypto_decode, scrypto_encode};
use scrypto::core::NetworkDefinition;
use scrypto::crypto::{hash, Hash, PublicKey, Signature, SignatureWithPublicKey};
use scrypto::values::*;

use crate::manifest::{compile, CompileError};
use crate::model::TransactionManifest;

// TODO: add versioning of transaction schema

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
#[custom_type_id(ScryptoCustomTypeId)]
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
#[custom_type_id(ScryptoCustomTypeId)]
pub struct TransactionIntent {
    pub header: TransactionHeader,
    pub manifest: TransactionManifest,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
#[custom_type_id(ScryptoCustomTypeId)]
pub struct SignedTransactionIntent {
    pub intent: TransactionIntent,
    pub intent_signatures: Vec<SignatureWithPublicKey>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
#[custom_type_id(ScryptoCustomTypeId)]
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
            "7c36cff285020a292330e2aa2fe8ac67008f2cb1542b389e8e454c080f2721a5",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "2b46356007d8f1a5131b599f5577cba8fe78718e0b6ff78c0ae1bdc0c263f50d",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "c6b1e206b8e76b244a12ab94f49454fb1930bfc0704379f93918886dbbdd0543",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a0500000000000000110e0000004563647361536563703235366b3101000000b12100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f901000940420f00090500000010020000002011010000000d000000436c656172417574685a6f6e65000000002020000000002011020000000e0000004563647361536563703235366b3101000000b241000000014fea00dfce89dbcaf1da77bf0c0e67470afd2a9b4ab28fa22cb7c770a656fc1b77881f8b2a5a48ee93fb1ec2f798e51f59d8194463649a5864b668a8065f0dc70e0000004563647361536563703235366b3101000000b241000000018bfce373b4d7b9da60bc6840a8eafe5e2f120ea31e3db6eb58644fb52cd8a3f41dda36901bf0c9fbf69903afcf73632b3e3ba00b9f2dddbbac1be2d1838209ef110e0000004563647361536563703235366b3101000000b2410000000057e0a0e6ea2719321a8aefd90925a96bc0acd5209e2742716edc895d61fa40685c2af12aef7e7a6f11d829791f274ae1cd637304573c3355e37151486f5e57f4", hex::encode(scrypto_encode(&transaction)));
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
            "8b2c75b1140a1c3d0bf3654da60a5ccb8413adff52dd7bff7784dceada537889",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "c54ae55ae7fb3f08d4df1d51fe26f8bbc7afca0ca32a326741bae19a00654248",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "0362f435691d8ac1ad3cc76c0f323f4eecccd943e140ab3f6733c33e7012a981",
            transaction.hash().to_string()
        );
        assert_eq!("1002000000100200000010020000001009000000070107f20a00000000000000000a64000000000000000a0500000000000000110c00000045646473614564323535313901000000b320000000f381626e41e7027ea431bfe3009e94bdd25a746beec468948d6c3c7c5dc9a54b01000940420f00090500000010020000002011010000000d000000436c656172417574685a6f6e65000000002020000000002011020000000c00000045646473614564323535313902000000b3200000004cb5abf6ad79fbf5abbccafcc269d85cd2651ed4b885b5869f241aedf0a5ba29b4400000006cb126ff30516e19bb0a74ec6e6a28f6d52d41e7145d3cee68dc8f455aeb834f23ad92865f6ac814465916d847dd3d2f501a6f4141590d6542371f898576c80f0c00000045646473614564323535313902000000b3200000007422b9887598068e32c4448a949adb290d0f4e35b9e01b0ee5f1a1e600fe2674b4400000002ddfba87719b3edf3313e9bc739d16398f3c5f5bd213ff54ccb4457dfe89730e876a111b0b8281ca795db613109b3bb1989a5871f8938ba742cbc9cae6623f09110c00000045646473614564323535313901000000b4400000002fa39602b3caec55b3cbbca5c870e508c6819a70914471fd734cffe6705cdfaab235f0e9ce819012616eca5ee4cbc0eee53125e52a02ab5c9041c3df8bcf9501", hex::encode(scrypto_encode(&transaction)));
    }
}
