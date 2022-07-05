use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::{scrypto_decode, scrypto_encode};
use scrypto::core::Network;
use scrypto::crypto::{hash, EcdsaPublicKey, EcdsaSignature, Hash};

use crate::manifest::{compile, CompileError};
use crate::model::Instruction;

// TODO: add versioning of transaction schema

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct TransactionHeader {
    pub version: u8,
    pub network: Network,
    pub start_epoch_inclusive: u64,
    pub end_epoch_exclusive: u64,
    pub nonce: u64,
    pub notary_public_key: EcdsaPublicKey,
    pub notary_as_signatory: bool,
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

impl TransactionIntent {
    pub fn new(header: TransactionHeader, manifest: &str) -> Result<Self, CompileError> {
        Ok(Self {
            header,
            manifest: compile(manifest)?,
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

    #[test]
    fn construct_sign_and_notarize() {
        // create a key pair
        let sk1 = EcdsaPrivateKey::from_u64(1).unwrap();
        let sk2 = EcdsaPrivateKey::from_u64(2).unwrap();
        let sk_notary = EcdsaPrivateKey::from_u64(3).unwrap();

        // construct
        let intent = TransactionIntent::new(
            TransactionHeader {
                version: 1,
                network: Network::InternalTestnet,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce: 5,
                notary_public_key: sk_notary.public_key(),
                notary_as_signatory: false,
            },
            "CLEAR_AUTH_ZONE;",
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
            "5be3e4fe37d11184239d75bc05642c839131ee0e011082cfa8fc81e274135174",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "a8ecbc46798841f3fa16ca299abc530f96fefe8666bb6c43fac063f10543cd32",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "02024325e8e7a43ead595aa388050455cc19b455d720a9eec6cb23fbbde5f1ca",
            transaction.hash().to_string()
        );
        assert_eq!("10020000001002000000100200000010070000000701110f000000496e7465726e616c546573746e6574000000000a00000000000000000a64000000000000000a0500000000000000912100000002f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9010010010000003011010000000d000000436c656172417574685a6f6e65000000003023020000000200000091210000000279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f8179892400000006cf35fe75e8cf4cc7db93e2d0b5e5f17efe0768cc2eb3db9d1e9d4bb8c6df6d95446cc78c550c68a91217f75266dc8ec14b1c2324637ea49cc99119d782f3a4b02000000912100000002c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee592400000000350a245e2df1143d5a97433cc640601e725fc342d3ba9ebd74052757526695432ff1c321c001ab11f01943a9da312333b78f4bcbadfac89754ec111c2cf5ea1924000000024bd869215c36f4291ea48ac7e1378758bef43a56088446d441f99509cec06f9516089eb7040d1bb9455be59455084c232ecc85becb496cb59b7c156a1206917", hex::encode(scrypto_encode(&transaction)));
    }
}
