use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::{scrypto_decode, scrypto_encode};
use scrypto::crypto::{hash, EcdsaPublicKey, EcdsaSignature, Hash};

use crate::manifest::{compile, CompileError};
use crate::model::Instruction;

// TODO: add versioning of transaction schema

// TODO: we may be able to squeeze network identifier into the other fields, like the `v` byte in signature.
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

    pub fn hash(&self) -> Hash {
        hash(self.to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        scrypto_encode(self)
    }
}

impl SignedTransactionIntent {
    pub fn hash(&self) -> Hash {
        hash(self.to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        scrypto_encode(self)
    }
}

impl NotarizedTransaction {
    pub fn from_slice(slice: &[u8]) -> Result<NotarizedTransaction, DecodeError> {
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
            "f63d14e41c4e7d39a5ee34882e7c29f36cf79715abaa3c68acdcb39ae00e314b",
            transaction.signed_intent.intent.hash().to_string()
        );
        assert_eq!(
            "b3b2b605b279035311ae4e55d28e9ac8fd0d43b4e9514ffd71a3eb9b1bb320c4",
            transaction.signed_intent.hash().to_string()
        );
        assert_eq!(
            "449b2b1b4a9f1830e4aed079429f24bd8621cb4433638db95f8abc409053eaaf",
            transaction.hash().to_string()
        );
        assert_eq!("10020000001002000000100200000010060000000701110f000000496e7465726e616c546573746e6574000000000a00000000000000000a64000000000000000a05000000000000009141000000045ecbe4d1a6330a44c8f7ef951d4bf165e6c6b721efada985fb41661bc6e7fd6c8734640c4998ff7e374b06ce1a64a2ecd82ab036384fb83d9a79b127a27d503210010000003011010000000d000000436c656172417574685a6f6e6500000000302302000000020000009141000000046b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c2964fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f593400000004b29b0c7e1d5be1b7bd1b8385f8ea6e8f8be257dd6f0802ae4fc0d6611c81c4e61b61c1ed55fc5787f375b117a2af26ba93badeff0e2c03c6b09fb4f72c2e182020000009141000000047cf27b188d034f7e8a52380304b51ac3c08969e277f21b35a60b48fc4766997807775510db8ed040293d9ac69f7430dbba7dade63ce982299e04b79d227873d1934000000093386b4f7f412ca9282d66c3807193e73c13731de8bd3e671ce0e02233b6d1a433202177d67e12eec24ae93fe7b28e996db226622c35e248e54f631f50ba0f7b934000000082254bf709be96e80dd06c9642a54988814b88731f0612423ff8549bc512284cce60c6c5716c5429c8cb0e51085ac266483b85c26b19b4c1deacf225672a8f6e", hex::encode(scrypto_encode(&transaction)));
    }
}
