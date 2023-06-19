use crate::model::*;
use crate::signing::Signer;

use super::manifest_builder::TransactionManifestV1;

pub struct TransactionBuilder {
    manifest: Option<TransactionManifestV1>,
    header: Option<TransactionHeaderV1>,
    message: Option<MessageV1>,
    intent_signatures: Vec<SignatureWithPublicKeyV1>,
    notary_signature: Option<SignatureV1>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            manifest: None,
            header: None,
            message: None,
            intent_signatures: vec![],
            notary_signature: None,
        }
    }

    pub fn manifest(mut self, manifest: TransactionManifestV1) -> Self {
        self.manifest = Some(manifest);
        self
    }

    pub fn header(mut self, header: TransactionHeaderV1) -> Self {
        self.header = Some(header);
        self
    }

    pub fn message(mut self, message: MessageV1) -> Self {
        self.message = Some(message);
        self
    }

    pub fn sign<S: Signer>(mut self, signer: &S) -> Self {
        let intent = self.transaction_intent();
        let prepared = intent.prepare().expect("Intent could be prepared");
        self.intent_signatures
            .push(signer.sign_with_public_key(&prepared.intent_hash()));
        self
    }

    pub fn signer_signatures(mut self, sigs: Vec<SignatureWithPublicKeyV1>) -> Self {
        self.intent_signatures.extend(sigs);
        self
    }

    pub fn notarize<S: Signer>(mut self, signer: &S) -> Self {
        let signed_intent = self.signed_transaction_intent();
        let prepared = signed_intent
            .prepare()
            .expect("Signed intent could be prepared");
        self.notary_signature = Some(
            signer
                .sign_with_public_key(&prepared.signed_intent_hash())
                .signature(),
        );
        self
    }

    pub fn notary_signature(mut self, signature: SignatureV1) -> Self {
        self.notary_signature = Some(signature);
        self
    }

    pub fn build(&self) -> NotarizedTransactionV1 {
        NotarizedTransactionV1 {
            signed_intent: self.signed_transaction_intent(),
            notary_signature: NotarySignatureV1(
                self.notary_signature.clone().expect("Not notarized"),
            ),
        }
    }

    fn transaction_intent(&self) -> IntentV1 {
        let (instructions, blobs) = self
            .manifest
            .clone()
            .expect("Manifest not specified")
            .for_intent();
        IntentV1 {
            header: self.header.clone().expect("Header not specified"),
            instructions,
            blobs,
            message: self.message.clone().unwrap_or_default(),
        }
    }

    fn signed_transaction_intent(&self) -> SignedIntentV1 {
        let intent = self.transaction_intent();
        SignedIntentV1 {
            intent,
            intent_signatures: IntentSignaturesV1 {
                signatures: self
                    .intent_signatures
                    .clone()
                    .into_iter()
                    .map(|sig| IntentSignatureV1(sig))
                    .collect(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use radix_engine_common::types::Epoch;
    use radix_engine_interface::network::NetworkDefinition;

    use super::*;
    use crate::builder::*;
    use crate::signing::secp256k1::Secp256k1PrivateKey;

    #[test]
    fn notary_as_signatory() {
        let private_key = Secp256k1PrivateKey::from_u64(1).unwrap();

        let transaction = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: Epoch::zero(),
                end_epoch_exclusive: Epoch::of(100),
                nonce: 5,
                notary_public_key: private_key.public_key().into(),
                notary_is_signatory: true,
                tip_percentage: 5,
            })
            .manifest(ManifestBuilder::new().clear_auth_zone().build())
            .notarize(&private_key)
            .build();

        let prepared = transaction.prepare().unwrap();
        assert_eq!(
            prepared
                .signed_intent
                .intent
                .header
                .inner
                .notary_is_signatory,
            true
        );
    }
}
