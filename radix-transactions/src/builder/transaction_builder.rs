use crate::internal_prelude::*;
use crate::model::*;
use crate::signing::Signer;

//====================================
// This file contains:
// * TransactionV1Builder (with alias TransactionBuilder), which creates:
//   - TransactionIntentV1
//   - SignedTransactionIntentV1
//   - NotarizedTransactionV1
// * PartialTransactionV2Builder, which creates:
//   - SubintentV2
//   - SignedPartialTransactionV2
// * TransactionV2Builder, which creates:
//   - TransactionIntentV2
//   - SignedTransactionIntentV2
//   - NotarizedTransactionV2
//====================================
pub type TransactionBuilder = TransactionV1Builder;

pub struct TransactionV1Builder {
    manifest: Option<TransactionManifestV1>,
    header: Option<TransactionHeaderV1>,
    message: Option<MessageV1>,
    intent_signatures: Vec<SignatureWithPublicKeyV1>,
    notary_signature: Option<SignatureV1>,
}

impl TransactionV1Builder {
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
        let prepared = intent
            .prepare(PreparationSettings::latest_ref())
            .expect("Intent could be prepared");
        self.intent_signatures
            .push(signer.sign_with_public_key(&prepared.transaction_intent_hash()));
        self
    }

    pub fn multi_sign<S: Signer>(mut self, signers: &[&S]) -> Self {
        let intent = self.transaction_intent();
        let prepared = intent
            .prepare(PreparationSettings::latest_ref())
            .expect("Intent could be prepared");
        for signer in signers {
            self.intent_signatures
                .push(signer.sign_with_public_key(&prepared.transaction_intent_hash()));
        }
        self
    }

    pub fn signer_signatures(mut self, sigs: Vec<SignatureWithPublicKeyV1>) -> Self {
        self.intent_signatures.extend(sigs);
        self
    }

    pub fn notarize<S: Signer>(mut self, signer: &S) -> Self {
        let signed_intent = self.signed_transaction_intent();
        let prepared = signed_intent
            .prepare(PreparationSettings::latest_ref())
            .expect("Signed intent could be prepared");
        self.notary_signature = Some(
            signer
                .sign_with_public_key(&prepared.signed_transaction_intent_hash())
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

    pub fn into_manifest(self) -> TransactionManifestV1 {
        self.manifest.expect("No manifest")
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
            message: self.message.clone().unwrap_or(MessageV1::None),
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

pub struct PartialTransactionV2Builder {
    children: IndexMap<String, (SubintentHash, SignedPartialTransactionV2)>,
    intent_header: Option<IntentHeaderV2>,
    message: Option<MessageV2>,
    manifest: Option<SubintentManifestV2>,
    // Cached once created
    intent: Option<SubintentV2>,
    prepared_intent: Option<PreparedSubintentV2>,
    intent_signatures: Vec<IntentSignatureV1>,
}

impl PartialTransactionV2Builder {
    pub fn add_signed_child(
        mut self,
        name: impl AsRef<str>,
        signed_partial_transaction: SignedPartialTransactionV2,
    ) -> Self {
        if self.manifest.is_some() {
            panic!("Call add_signed_child before calling manifest or manifest_builder");
        }

        let prepared = signed_partial_transaction
            .prepare(PreparationSettings::latest_ref())
            .expect("Child signed partial transation could not be prepared");
        let hash = prepared.subintent_hash();
        let name = name.as_ref();
        let replaced = self
            .children
            .insert(name.to_string(), (hash, signed_partial_transaction));
        if replaced.is_some() {
            panic!("Child with name {name} already exists");
        }
        self
    }

    /// You should call `add_signed_child` first.
    pub fn manifest_builder(
        self,
        build_manifest: impl FnOnce(SubintentManifestV2Builder) -> SubintentManifestV2Builder,
    ) -> Self {
        let mut manifest_builder = SubintentManifestV2Builder::new_typed();
        for (child_name, (hash, _)) in self.children.iter() {
            manifest_builder = manifest_builder.register_child(child_name, *hash);
        }
        self.manifest(build_manifest(manifest_builder).build())
    }

    pub fn manifest(mut self, manifest: SubintentManifestV2) -> Self {
        self.manifest = Some(manifest);
        self
    }

    pub fn message(mut self, message: MessageV2) -> Self {
        self.message = Some(message);
        self
    }

    pub fn intent_header(mut self, intent_header: IntentHeaderV2) -> Self {
        self.intent_header = Some(intent_header);
        self
    }

    pub fn create_subintent(&mut self) -> &SubintentV2 {
        if self.intent.is_none() {
            let (instructions, blobs, children) = self
                .manifest
                .as_ref()
                .expect("Manifest must be provided")
                .clone()
                .for_intent();
            self.intent = Some(SubintentV2 {
                intent_core: IntentCoreV2 {
                    header: self
                        .intent_header
                        .as_ref()
                        .expect("Intent Header must be provided")
                        .clone(),
                    blobs,
                    message: self
                        .message
                        .as_ref()
                        .expect("Message must be provided")
                        .clone(),
                    children,
                    instructions,
                },
            });
        }
        self.intent.as_ref().unwrap()
    }

    pub fn create_prepared_subintent(&mut self) -> &PreparedSubintentV2 {
        if self.prepared_intent.is_none() {
            let prepared = self
                .create_subintent()
                .prepare(PreparationSettings::latest_ref())
                .expect("Expected that subintent could be prepared");
            self.prepared_intent = Some(prepared);
        }
        self.prepared_intent.as_ref().unwrap()
    }

    pub fn subintent_hash(&mut self) -> SubintentHash {
        self.create_prepared_subintent().subintent_hash()
    }

    pub fn sign<S: Signer>(mut self, signer: &S) -> Self {
        let hash = self.subintent_hash();
        self.intent_signatures
            .push(IntentSignatureV1(signer.sign_with_public_key(&hash)));
        self
    }

    pub fn multi_sign<S: Signer>(mut self, signers: &[&S]) -> Self {
        let hash = self.subintent_hash();
        for signer in signers {
            self.intent_signatures
                .push(IntentSignatureV1(signer.sign_with_public_key(&hash)));
        }
        self
    }

    pub fn build(mut self) -> SignedPartialTransactionV2 {
        // Ensure subintent has been created
        self.create_subintent();

        // Now assemble the signed partial transaction
        let mut aggregated_subintents = vec![];
        let mut aggregated_subintent_signatures = vec![];
        for (_name, (_hash, child_partial_transaction)) in self.children {
            let SignedPartialTransactionV2 {
                partial_transaction,
                root_intent_signatures,
                subintent_signatures,
            } = child_partial_transaction;
            aggregated_subintents.push(partial_transaction.root_intent);
            aggregated_subintents.extend(partial_transaction.subintents.0);
            aggregated_subintent_signatures.push(root_intent_signatures);
            aggregated_subintent_signatures.extend(subintent_signatures.by_subintent);
        }
        SignedPartialTransactionV2 {
            partial_transaction: PartialTransactionV2 {
                root_intent: self
                    .intent
                    .expect("Expected intent to already be compiled by now"),
                subintents: SubintentsV2(aggregated_subintents),
            },
            root_intent_signatures: IntentSignaturesV2 {
                signatures: self.intent_signatures,
            },
            subintent_signatures: MultipleIntentSignaturesV2 {
                by_subintent: aggregated_subintent_signatures,
            },
        }
    }
}

/// A builder for building a NotarizedTransactionV2.
/// In future, we may make this into a state-machine style builder.
pub struct TransactionV2Builder {
    children: IndexMap<String, (SubintentHash, SignedPartialTransactionV2)>,
    transaction_header: Option<TransactionHeaderV2>,
    intent_header: Option<IntentHeaderV2>,
    message: Option<MessageV2>,
    manifest: Option<TransactionManifestV2>,
    // Temporarily cached once created
    intent_and_subintent_signatures: Option<(TransactionIntentV2, MultipleIntentSignaturesV2)>,
    prepared_intent: Option<PreparedTransactionIntentV2>,
    intent_signatures: Vec<IntentSignatureV1>,
    signed_intent: Option<SignedTransactionIntentV2>,
    prepared_signed_intent: Option<PreparedSignedTransactionIntentV2>,
    notary_signature: Option<NotarySignatureV1>,
}

impl TransactionV2Builder {
    pub fn add_signed_child(
        mut self,
        name: impl AsRef<str>,
        signed_partial_transaction: SignedPartialTransactionV2,
    ) -> Self {
        if self.manifest.is_some() {
            panic!("Call add_signed_child before calling manifest or manifest_builder");
        }

        let prepared = signed_partial_transaction
            .prepare(PreparationSettings::latest_ref())
            .expect("Child signed partial transation could not be prepared");
        let hash = prepared.subintent_hash();
        let name = name.as_ref();
        let replaced = self
            .children
            .insert(name.to_string(), (hash, signed_partial_transaction));
        if replaced.is_some() {
            panic!("Child with name {name} already exists");
        }
        self
    }

    /// If the intent has any children, you should call `add_signed_child` first.
    pub fn manifest_builder(
        self,
        build_manifest: impl FnOnce(TransactionManifestV2Builder) -> TransactionManifestV2Builder,
    ) -> Self {
        let mut manifest_builder = TransactionManifestV2Builder::new_typed();
        for (child_name, (hash, _)) in self.children.iter() {
            manifest_builder = manifest_builder.register_child(child_name, *hash);
        }
        self.manifest(build_manifest(manifest_builder).build())
    }

    pub fn manifest(mut self, manifest: TransactionManifestV2) -> Self {
        self.manifest = Some(manifest);
        self
    }

    pub fn message(mut self, message: MessageV2) -> Self {
        self.message = Some(message);
        self
    }

    pub fn intent_header(mut self, intent_header: IntentHeaderV2) -> Self {
        self.intent_header = Some(intent_header);
        self
    }

    pub fn create_intent_and_subintent_signatures(&mut self) -> &TransactionIntentV2 {
        if self.intent_and_subintent_signatures.is_none() {
            let (instructions, blobs, child_hashes) = self
                .manifest
                .as_ref()
                .expect("Manifest must be provided")
                .clone()
                .for_intent();
            let subintents = core::mem::take(&mut self.children);

            let mut aggregated_subintents = vec![];
            let mut aggregated_subintent_signatures = vec![];
            for (_name, (_hash, child_partial_transaction)) in subintents {
                let SignedPartialTransactionV2 {
                    partial_transaction,
                    root_intent_signatures,
                    subintent_signatures,
                } = child_partial_transaction;
                aggregated_subintents.push(partial_transaction.root_intent);
                aggregated_subintents.extend(partial_transaction.subintents.0);
                aggregated_subintent_signatures.push(root_intent_signatures);
                aggregated_subintent_signatures.extend(subintent_signatures.by_subintent);
            }
            let intent = TransactionIntentV2 {
                root_header: self
                    .transaction_header
                    .as_ref()
                    .expect("Transaction Header must be provided")
                    .clone(),
                root_intent_core: IntentCoreV2 {
                    header: self
                        .intent_header
                        .as_ref()
                        .expect("Intent Header must be provided")
                        .clone(),
                    blobs,
                    message: self
                        .message
                        .as_ref()
                        .expect("Message must be provided")
                        .clone(),
                    children: child_hashes,
                    instructions,
                },
                subintents: SubintentsV2(aggregated_subintents),
            };
            let subintent_signatures = MultipleIntentSignaturesV2 {
                by_subintent: aggregated_subintent_signatures,
            };
            self.intent_and_subintent_signatures = Some((intent, subintent_signatures));
        }
        &self.intent_and_subintent_signatures.as_ref().unwrap().0
    }

    pub fn create_prepared_intent(&mut self) -> &PreparedTransactionIntentV2 {
        if self.prepared_intent.is_none() {
            let prepared = self
                .create_intent_and_subintent_signatures()
                .prepare(PreparationSettings::latest_ref())
                .expect("Expected that the intent could be prepared");
            self.prepared_intent = Some(prepared);
        }
        self.prepared_intent.as_ref().unwrap()
    }

    pub fn intent_hash(&mut self) -> TransactionIntentHash {
        self.create_prepared_intent().transaction_intent_hash()
    }

    pub fn sign<S: Signer>(mut self, signer: &S) -> Self {
        let hash = self.intent_hash();
        self.intent_signatures
            .push(IntentSignatureV1(signer.sign_with_public_key(&hash)));
        self
    }

    pub fn multi_sign<S: Signer>(mut self, signers: &[&S]) -> Self {
        let hash = self.intent_hash();
        for signer in signers {
            self.intent_signatures
                .push(IntentSignatureV1(signer.sign_with_public_key(&hash)));
        }
        self
    }

    pub fn add_signature(mut self, signature: SignatureWithPublicKeyV1) -> Self {
        self.intent_signatures.push(IntentSignatureV1(signature));
        self
    }

    pub fn create_signed_transaction_intent(&mut self) -> &SignedTransactionIntentV2 {
        if self.signed_intent.is_none() {
            self.create_intent_and_subintent_signatures();
            let (root_intent, subintent_signatures) =
                mem::take(&mut self.intent_and_subintent_signatures).unwrap();
            let signatures = mem::take(&mut self.intent_signatures);
            let signed_intent = SignedTransactionIntentV2 {
                root_intent,
                root_intent_signatures: IntentSignaturesV1 { signatures },
                subintent_signatures,
            };
            self.signed_intent = Some(signed_intent);
        }
        self.signed_intent.as_ref().unwrap()
    }

    pub fn create_prepared_signed_transaction_intent(
        &mut self,
    ) -> &PreparedSignedTransactionIntentV2 {
        if self.prepared_intent.is_none() {
            let prepared = self
                .create_signed_transaction_intent()
                .prepare(PreparationSettings::latest_ref())
                .expect("Expected that signed intent could be prepared");
            self.prepared_signed_intent = Some(prepared);
        }
        self.prepared_signed_intent.as_ref().unwrap()
    }

    pub fn notarize<S: Signer>(mut self, signer: &S) -> Self {
        let hash = self
            .create_prepared_signed_transaction_intent()
            .signed_transaction_intent_hash();
        self.notary_signature = Some(NotarySignatureV1(
            signer.sign_with_public_key(&hash).signature(),
        ));
        self
    }

    pub fn notary_signature(mut self, signature: SignatureV1) -> Self {
        self.notary_signature = Some(NotarySignatureV1(signature));
        self
    }

    pub fn build(self) -> NotarizedTransactionV2 {
        NotarizedTransactionV2 {
            signed_intent: self.signed_intent.expect("Expected signed intent to exist"),
            notary_signature: self
                .notary_signature
                .expect("Expected notary signature to exist"),
        }
    }
}

#[cfg(test)]
mod tests {
    use radix_common::network::NetworkDefinition;
    use radix_common::types::Epoch;

    use super::*;
    use crate::builder::*;
    use crate::internal_prelude::Secp256k1PrivateKey;

    #[test]
    #[allow(deprecated)]
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
            .manifest(ManifestBuilder::new().drop_auth_zone_proofs().build())
            .notarize(&private_key)
            .build();

        let prepared = transaction
            .prepare(PreparationSettings::latest_ref())
            .unwrap();
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
