use scrypto::{buffer::scrypto_encode, crypto::*};

use crate::{model::*, signing::Signer};

pub struct TransactionBuilder {
    manifest: Option<TransactionManifest>,
    header: Option<TransactionHeader>,
    intent_signatures: Vec<(EcdsaPublicKey, EcdsaSignature)>,
    notary_signature: Option<EcdsaSignature>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            manifest: None,
            header: None,
            intent_signatures: Vec::new(),
            notary_signature: None,
        }
    }

    pub fn manifest(mut self, manifest: TransactionManifest) -> Self {
        self.manifest = Some(manifest);
        self
    }

    pub fn header(mut self, header: TransactionHeader) -> Self {
        self.header = Some(header);
        self
    }

    pub fn sign<S: Signer>(mut self, signer: &S) -> Self {
        let intent = self.transaction_intent();
        let intent_payload = scrypto_encode(&intent);
        self.intent_signatures
            .push((signer.public_key(), signer.sign(&intent_payload)));
        self
    }

    pub fn signer_signatures(mut self, signatures: Vec<(EcdsaPublicKey, EcdsaSignature)>) -> Self {
        self.intent_signatures.extend(signatures);
        self
    }

    pub fn notarize<S: Signer>(mut self, signer: &S) -> Self {
        let signed_intent = self.signed_transaction_intent();
        let signed_intent_payload = scrypto_encode(&signed_intent);
        self.notary_signature = Some(signer.sign(&signed_intent_payload));
        self
    }

    pub fn notary_signature(mut self, signature: EcdsaSignature) -> Self {
        self.notary_signature = Some(signature);
        self
    }

    pub fn build(&self) -> NotarizedTransaction {
        NotarizedTransaction {
            signed_intent: self.signed_transaction_intent(),
            notary_signature: self.notary_signature.clone().expect("Not notarized"),
        }
    }

    fn transaction_intent(&self) -> TransactionIntent {
        TransactionIntent {
            manifest: self.manifest.clone().expect("Manifest not specified"),
            header: self.header.clone().expect("Header not specified"),
        }
    }

    fn signed_transaction_intent(&self) -> SignedTransactionIntent {
        let intent = self.transaction_intent();
        SignedTransactionIntent {
            intent,
            intent_signatures: self.intent_signatures.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use scrypto::core::Network;

    use super::*;
    use crate::builder::*;
    use crate::signing::*;

    #[test]
    fn notary_as_signatory() {
        let private_key = EcdsaPrivateKey::from_u64(1).unwrap();

        let transaction = TransactionBuilder::new()
            .header(TransactionHeader {
                version: 1,
                network_id: Network::LocalSimulator.get_id(),
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce: 5,
                notary_public_key: private_key.public_key(),
                notary_as_signatory: true,
                cost_unit_limit: 1_000_000,
                tip_percentage: 5,
            })
            .manifest(
                ManifestBuilder::new(Network::LocalSimulator)
                    .clear_auth_zone()
                    .build(),
            )
            .notarize(&private_key)
            .build();

        let bytes = transaction.to_bytes();
        NotarizedTransaction::from_slice(&bytes).unwrap();
    }
}
