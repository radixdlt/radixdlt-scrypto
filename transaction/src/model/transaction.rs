use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::crypto::{EcdsaPublicKey, EcdsaSignature, Hash};

use crate::errors::*;
use crate::manifest::{compile, CompileError};
use crate::model::Instruction;
use crate::signing::Signer;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Network {
    InternalTestnet,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct TransactionHeader {
    pub version: u8,
    pub network: Network,
    pub start_epoch_inclusive: u64,
    pub end_epoch_exclusive: u64,
    pub nonce: u64,
    pub notary_public_key: EcdsaPublicKey,
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
    pub intent_signatures: Vec<TransactionSignature>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Transaction {
    pub signed_intent: SignedTransactionIntent,
    pub notary_signature: TransactionSignature,
}

pub type TransactionSignature = (EcdsaPublicKey, EcdsaSignature);

impl TransactionIntent {
    pub fn new(header: TransactionHeader, manifest: &str) -> Result<Self, CompileError> {
        Ok(Self {
            header,
            manifest: TransactionManifest {
                instructions: compile(manifest)?,
            },
        })
    }

    pub fn hash(&self) -> Hash {
        todo!()
    }

    pub fn sign<S: Signer>(&self, signer: &S) -> TransactionSignature {
        todo!()
    }
}

impl SignedTransactionIntent {
    pub fn hash(&self) -> Hash {
        todo!()
    }

    pub fn notarize<S: Signer>(&self, signer: &S) -> TransactionSignature {
        todo!()
    }
}

impl Transaction {
    pub fn from_slice(slice: &[u8]) -> Result<Transaction, DecodeError> {
        todo!()
    }

    pub fn intent_hash(&self) -> Hash {
        todo!()
    }

    pub fn signed_intent_hash(&self) -> Hash {
        todo!()
    }

    pub fn hash(&self) -> Hash {
        todo!()
    }

    pub fn validate_header(&self, current_epoch: u64) -> Result<(), HeaderValidationError> {
        todo!()
    }

    pub fn validate_signatures(
        &self,
    ) -> Result<(Vec<EcdsaPublicKey>, EcdsaPublicKey), SignatureValidationError> {
        let msg = self.intent_hash();
        // if !EcdsaVerifier::verify(&msg, pk, sig) {
        //     return Err(TransactionValidationError::InvalidSignature);
        // }
        todo!()
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
            },
            "CLEAR_AUTH_ZONE;",
        )
        .unwrap();

        // sign
        let signature1 = intent.sign(&sk1);
        let signature2 = intent.sign(&sk2);
        let signed_intent = SignedTransactionIntent {
            intent,
            intent_signatures: vec![signature1, signature2],
        };

        // notarize
        let signature3 = signed_intent.notarize(&sk_notary);
        let transaction = Transaction {
            signed_intent,
            notary_signature: signature3,
        };

        assert_eq!("", transaction.intent_hash().to_string());
        assert_eq!("", transaction.signed_intent_hash().to_string());
        assert_eq!("", transaction.hash().to_string());
        assert_eq!("", hex::encode(scrypto_encode(&transaction)));
    }
}
