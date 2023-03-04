use crate::model::*;
use crate::signing::Signer;
use radix_engine_interface::crypto::hash;
use radix_engine_interface::data::manifest::manifest_encode;

pub struct TransactionBuilder {
    manifest: Option<TransactionManifest>,
    header: Option<TransactionHeader>,
    intent_signatures: Vec<SignatureWithPublicKey>,
    notary_signature: Option<Signature>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            manifest: None,
            header: None,
            intent_signatures: vec![],
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
        let intent_payload = manifest_encode(&intent).unwrap();
        let intent_payload_hash = hash(intent_payload);
        self.intent_signatures
            .push(signer.sign(&intent_payload_hash));
        self
    }

    pub fn signer_signatures(mut self, sigs: Vec<SignatureWithPublicKey>) -> Self {
        self.intent_signatures.extend(sigs);
        self
    }

    pub fn notarize<S: Signer>(mut self, signer: &S) -> Self {
        let signed_intent = self.signed_transaction_intent();
        let signed_intent_payload = manifest_encode(&signed_intent).unwrap();
        let signed_intent_payload_hash = hash(signed_intent_payload);
        self.notary_signature = Some(signer.sign(&signed_intent_payload_hash).signature());
        self
    }

    pub fn notary_signature(mut self, signature: Signature) -> Self {
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
    use radix_engine_interface::network::NetworkDefinition;

    use super::*;
    use crate::builder::*;
    use crate::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

    #[test]
    fn notary_as_signatory() {
        let private_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap();

        let transaction = TransactionBuilder::new()
            .header(TransactionHeader {
                version: 1,
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce: 5,
                notary_public_key: private_key.public_key().into(),
                notary_as_signatory: true,
                cost_unit_limit: 1_000_000,
                tip_percentage: 5,
            })
            .manifest(ManifestBuilder::new().clear_auth_zone().build())
            .notarize(&private_key)
            .build();

        let bytes = transaction.to_bytes().unwrap();
        NotarizedTransaction::from_slice(&bytes).unwrap();
    }
}
