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

/// A builder for a [`SignedPartialTransactionV2`].
///
/// This should be used in the following order:
///
/// * Configure the root subintent:
///   * Set the [`intent_header`][Self::intent_header]
///   * Optionally, set the [`message`][Self::message]
///   * Optionally, add one or more signed partial transaction children [`add_signed_child`][Self::add_signed_child].
///     These can themseleves be created with the [`PartialTransactionV2Builder`] methods [`build_with_names()`][PartialTransactionV2Builder::build_with_names] or
///     [`build`][PartialTransactionV2Builder::build].
///   * Set the manifest with [`manifest_builder`][Self::manifest_builder].
/// * Sign the root subintent zero or more times:
///   * Use methods [`sign`][Self::sign] or [`multi_sign`][Self::multi_sign] [`add_signature`][Self::add_signature]
/// * Build:
///   * Use method [`build_and_validate`][Self::build_and_validate], [`build`][Self::build],
///     [`build_with_names_and_validate`][Self::build_with_names_and_validate] or [`build_with_names`][Self::build_with_names]
///
/// The error messages aren't great if used out of order.
/// In future, this may become a state-machine style builder, to catch more errors at compile time.
#[derive(Default)]
pub struct PartialTransactionV2Builder {
    child_partial_transactions: IndexMap<
        String,
        (
            SubintentHash,
            SignedPartialTransactionV2,
            TransactionObjectNames,
        ),
    >,
    root_subintent_header: Option<IntentHeaderV2>,
    root_subintent_message: Option<MessageV2>,
    root_subintent_manifest: Option<SubintentManifestV2>,
    // Cached once created
    root_subintent: Option<(SubintentV2, ManifestObjectNames)>,
    prepared_root_subintent: Option<PreparedSubintentV2>,
    root_subintent_signatures: Vec<IntentSignatureV1>,
}

impl PartialTransactionV2Builder {
    pub fn new() -> Self {
        Default::default()
    }

    /// When used with the [`manifest_builder`][Self::manifest_builder] method, the provided name and hash
    /// are provided automatically via [`use_child`][ManifestBuilder::use_child] at the start of manifest creation.
    ///
    /// When used with the [`manifest`][Self::manifest] method, the provided name is simply ignored - names
    /// are returned from the provided manifest.
    pub fn add_signed_child(
        mut self,
        name: impl AsRef<str>,
        signed_partial_transaction: impl Into<(SignedPartialTransactionV2, TransactionObjectNames)>,
    ) -> Self {
        if self.root_subintent_manifest.is_some() {
            panic!("Call add_signed_child before calling manifest or manifest_builder");
        }
        let (signed_partial_transaction, object_names) = signed_partial_transaction.into();

        let prepared = signed_partial_transaction
            .prepare(PreparationSettings::latest_ref())
            .expect("Child signed partial transation could not be prepared");
        let hash = prepared.subintent_hash();
        let name = name.as_ref();
        let replaced = self.child_partial_transactions.insert(
            name.to_string(),
            (hash, signed_partial_transaction, object_names),
        );
        if replaced.is_some() {
            panic!("Child with name {name} already exists");
        }
        self
    }

    /// If the intent has any children, you should call [`add_signed_child`][Self::add_signed_child] first.
    /// These children will get added to the manifest for you, with the corresponding names.
    pub fn manifest_builder(
        mut self,
        build_manifest: impl FnOnce(SubintentManifestV2Builder) -> SubintentManifestV2Builder,
    ) -> Self {
        let mut manifest_builder = SubintentManifestV2Builder::new_typed();
        for (child_name, (hash, _, _)) in self.child_partial_transactions.iter() {
            manifest_builder = manifest_builder.use_child(child_name, *hash);
        }
        self.root_subintent_manifest = Some(build_manifest(manifest_builder).build());
        self
    }

    /// Panics:
    /// * If called with a manifest which references different children to those provided by [`add_signed_child`][Self::add_signed_child].
    pub fn manifest(mut self, manifest: SubintentManifestV2) -> Self {
        let known_subintent_hashes: IndexSet<_> = self
            .child_partial_transactions
            .values()
            .map(|(hash, _, _)| ChildSubintentSpecifier { hash: *hash })
            .collect();
        if &manifest.children != &known_subintent_hashes {
            panic!(
                "The manifest's children hashes do not match those provided by `add_signed_child`"
            );
        }
        self.root_subintent_manifest = Some(manifest);
        self
    }

    pub fn message(mut self, message: MessageV2) -> Self {
        self.root_subintent_message = Some(message);
        self
    }

    pub fn intent_header(mut self, intent_header: IntentHeaderV2) -> Self {
        self.root_subintent_header = Some(intent_header);
        self
    }

    pub fn create_subintent(&mut self) -> &SubintentV2 {
        if self.root_subintent.is_none() {
            let (instructions, blobs, children, names) = self
                .root_subintent_manifest
                .take()
                .expect("Manifest must be provided before this action is performed")
                .for_intent_with_names();
            let subintent = SubintentV2 {
                intent_core: IntentCoreV2 {
                    header: self
                        .root_subintent_header
                        .take()
                        .expect("Intent Header must be provided before this action is performed"),
                    blobs,
                    message: self.root_subintent_message.take().unwrap_or_default(),
                    children,
                    instructions,
                },
            };
            self.root_subintent = Some((subintent, names));
        }
        &self.root_subintent.as_ref().unwrap().0
    }

    pub fn create_prepared_subintent(&mut self) -> &PreparedSubintentV2 {
        if self.prepared_root_subintent.is_none() {
            let prepared = self
                .create_subintent()
                .prepare(PreparationSettings::latest_ref())
                .expect("Expected that subintent could be prepared");
            self.prepared_root_subintent = Some(prepared);
        }
        self.prepared_root_subintent.as_ref().unwrap()
    }

    pub fn subintent_hash(&mut self) -> SubintentHash {
        self.create_prepared_subintent().subintent_hash()
    }

    pub fn sign<S: Signer>(mut self, signer: &S) -> Self {
        let hash = self.subintent_hash();
        self.root_subintent_signatures
            .push(IntentSignatureV1(signer.sign_with_public_key(&hash)));
        self
    }

    pub fn multi_sign<S: Signer>(mut self, signers: &[&S]) -> Self {
        let hash = self.subintent_hash();
        for signer in signers {
            self.root_subintent_signatures
                .push(IntentSignatureV1(signer.sign_with_public_key(&hash)));
        }
        self
    }

    pub fn add_signature(mut self, signature: SignatureWithPublicKeyV1) -> Self {
        self.root_subintent_signatures
            .push(IntentSignatureV1(signature));
        self
    }

    /// Builds the [`SignedPartialTransactionV2`], and returns the names used for manifest variables in
    /// the root and non-root subintents.
    ///
    /// Unlike [`TransactionV2Builder`], `build_with_names()` does not validate the partial transaction, to save
    /// lots duplicate work when building a full transaction from layers of partial transaction. If you wish,
    /// you can opt into validation with [`build_with_names_and_validate()`][Self::build_with_names_and_validate].
    pub fn build_with_names(mut self) -> (SignedPartialTransactionV2, TransactionObjectNames) {
        // Ensure subintent has been created
        self.create_subintent();

        // Now assemble the signed partial transaction
        let mut aggregated_subintents = vec![];
        let mut aggregated_subintent_signatures = vec![];
        let mut aggregated_subintent_object_names = vec![];
        for (_name, (_hash, child_partial_transaction, object_names)) in
            self.child_partial_transactions
        {
            let SignedPartialTransactionV2 {
                partial_transaction,
                root_subintent_signatures: root_intent_signatures,
                non_root_subintent_signatures: subintent_signatures,
            } = child_partial_transaction;
            let TransactionObjectNames {
                root_intent: root_intent_names,
                subintents: subintent_names,
            } = object_names;
            aggregated_subintents.push(partial_transaction.root_subintent);
            aggregated_subintents.extend(partial_transaction.non_root_subintents.0);
            aggregated_subintent_signatures.push(root_intent_signatures);
            aggregated_subintent_signatures.extend(subintent_signatures.by_subintent);
            aggregated_subintent_object_names.push(root_intent_names);
            aggregated_subintent_object_names.extend(subintent_names);
        }
        let (root_intent, root_intent_names) = self
            .root_subintent
            .expect("Expected intent to already be compiled by now");
        let signed_partial_transaction = SignedPartialTransactionV2 {
            partial_transaction: PartialTransactionV2 {
                root_subintent: root_intent,
                non_root_subintents: NonRootSubintentsV2(aggregated_subintents),
            },
            root_subintent_signatures: IntentSignaturesV2 {
                signatures: self.root_subintent_signatures,
            },
            non_root_subintent_signatures: NonRootSubintentSignaturesV2 {
                by_subintent: aggregated_subintent_signatures,
            },
        };
        let object_names = TransactionObjectNames {
            root_intent: root_intent_names,
            subintents: aggregated_subintent_object_names,
        };
        (signed_partial_transaction, object_names)
    }

    /// Builds and validates the [`SignedPartialTransactionV2`], and returns the names used for manifest variables in
    /// the root and non-root subintents.
    ///
    /// # Panics
    /// Panics if the built transaction does not pass validation with the latest validator.
    ///
    /// You can use [`build_with_names()`][Self::build_with_names] to skip this validation.
    pub fn build_with_names_and_validate(
        self,
    ) -> (SignedPartialTransactionV2, TransactionObjectNames) {
        let (transaction, names) = self.build_with_names();
        let validator = TransactionValidator::new_with_latest_config_network_agnostic();
        transaction.prepare_and_validate(&validator)
            .expect("Built partial transaction should be valid. Use `build()` to skip validation if needed.");
        (transaction, names)
    }

    /// Builds the [`SignedPartialTransactionV2`].
    ///
    /// You may wish to use [`build_with_names()`][Self::build_with_names] to preserve the names
    /// used for manifest variables in the root and non-root subintents.
    ///
    /// Unlike [`TransactionV2Builder`], `build()` does not validate the partial transaction, to save
    /// lots duplicate work when building a full transaction from layers of partial transaction. If you wish,
    /// you can opt into validation with [`build_and_validate()`][Self::build_and_validate].
    pub fn build(self) -> SignedPartialTransactionV2 {
        self.build_with_names().0
    }

    /// Builds and validates the [`SignedPartialTransactionV2`].
    ///
    /// # Panics
    /// Panics if the built transaction does not pass validation with the latest validator.
    ///
    /// You can use [`build()`][Self::build] to skip this validation.
    pub fn build_and_validate(self) -> SignedPartialTransactionV2 {
        let transaction = self.build();
        let validator = TransactionValidator::new_with_latest_config_network_agnostic();
        transaction.prepare_and_validate(&validator)
            .expect("Built partial transaction should be valid. Use `build()` to skip validation if needed.");
        transaction
    }
}

/// A builder for a [`NotarizedTransactionV2`].
///
/// This should be used in the following order:
///
/// * Configure the root transaction intent:
///   * Set the [`transaction_header`][Self::transaction_header]
///   * Set the [`intent_header`][Self::intent_header]
///   * Optionally, set the [`message`][Self::message]
///   * Optionally, add one or more signed partial transaction children with [`add_signed_child`][Self::add_signed_child].
///     These can be created with the [`PartialTransactionV2Builder`] methods [`build_with_names()`][PartialTransactionV2Builder::build_with_names] or
///     [`build`][PartialTransactionV2Builder::build].
///   * Set the manifest with [`manifest_builder`][Self::manifest_builder].
/// * Sign the root transaction manifest zero or more times:
///   * Use methods [`sign`][Self::sign] or [`multi_sign`][Self::multi_sign] [`add_signature`][Self::add_signature]
/// * Notarize once with the notary key from the intent header:
///   * Use either [`notarize`][Self::notarize] or [`notary_signature`][Self::notary_signature].
/// * Build:
///   * Use method [`build`][Self::build], [`build_no_validate`][Self::build_no_validate],
///     [`build_with_names`][Self::build_with_names] or [`build_with_names_no_validate`][Self::build_with_names_no_validate]
///
/// The error messages aren't great if used out of order.
/// In future, this may become a state-machine style builder, to catch more errors at compile time.
#[derive(Default)]
pub struct TransactionV2Builder {
    // Note - these names are long, but agreed with Yulong that we would clarify
    // non_root_subintents from root_subintent / transaction_intent so this is
    // applying that logic to these field names
    child_partial_transactions: IndexMap<
        String,
        (
            SubintentHash,
            SignedPartialTransactionV2,
            TransactionObjectNames,
        ),
    >,
    transaction_header: Option<TransactionHeaderV2>,
    transaction_intent_header: Option<IntentHeaderV2>,
    transaction_intent_message: Option<MessageV2>,
    transaction_intent_manifest: Option<TransactionManifestV2>,
    // Temporarily cached once created
    transaction_intent_and_non_root_subintent_signatures:
        Option<(TransactionIntentV2, NonRootSubintentSignaturesV2)>,
    transaction_intent_object_names: Option<TransactionObjectNames>,
    prepared_transaction_intent: Option<PreparedTransactionIntentV2>,
    transaction_intent_signatures: Vec<IntentSignatureV1>,
    signed_transaction_intent: Option<SignedTransactionIntentV2>,
    prepared_signed_transaction_intent: Option<PreparedSignedTransactionIntentV2>,
    notary_signature: Option<NotarySignatureV2>,
}

impl TransactionV2Builder {
    pub fn new() -> Self {
        Default::default()
    }

    /// When used with the [`manifest_builder`][Self::manifest_builder] method, the provided name and hash
    /// are provided automatically via [`use_child`][ManifestBuilder::use_child] at the start of manifest creation.
    ///
    /// When used with the [`manifest`][Self::manifest] method, the provided name is simply ignored - names
    /// are returned from the provided manifest.
    pub fn add_signed_child(
        mut self,
        name: impl AsRef<str>,
        signed_partial_transaction: impl Into<(SignedPartialTransactionV2, TransactionObjectNames)>,
    ) -> Self {
        if self.transaction_intent_manifest.is_some() {
            panic!("Call add_signed_child before calling manifest or manifest_builder");
        }

        let (signed_partial_transaction, object_names) = signed_partial_transaction.into();

        let prepared = signed_partial_transaction
            .prepare(PreparationSettings::latest_ref())
            .expect("Child signed partial transation could not be prepared");
        let hash = prepared.subintent_hash();
        let name = name.as_ref();
        let replaced = self.child_partial_transactions.insert(
            name.to_string(),
            (hash, signed_partial_transaction, object_names),
        );
        if replaced.is_some() {
            panic!("Child with name {name} already exists");
        }
        self
    }

    /// If the intent has any children, you should call [`add_signed_child`][Self::add_signed_child] first.
    /// These children will get added to the manifest for you, with the corresponding names.
    pub fn manifest_builder(
        mut self,
        build_manifest: impl FnOnce(TransactionManifestV2Builder) -> TransactionManifestV2Builder,
    ) -> Self {
        let mut manifest_builder = TransactionManifestV2Builder::new_typed();
        for (child_name, (hash, _, _)) in self.child_partial_transactions.iter() {
            manifest_builder = manifest_builder.use_child(child_name, *hash);
        }
        self.transaction_intent_manifest = Some(build_manifest(manifest_builder).build());
        self
    }

    /// ## Panics:
    /// * If called with a manifest which references different children to those provided by [`add_signed_child`][Self::add_signed_child].
    pub fn manifest(mut self, manifest: TransactionManifestV2) -> Self {
        let known_subintent_hashes: IndexSet<_> = self
            .child_partial_transactions
            .values()
            .map(|(hash, _, _)| ChildSubintentSpecifier { hash: *hash })
            .collect();
        if &manifest.children != &known_subintent_hashes {
            panic!(
                "The manifest's children hashes do not match those provided by `add_signed_child`"
            );
        }
        self.transaction_intent_manifest = Some(manifest);
        self
    }

    pub fn message(mut self, message: MessageV2) -> Self {
        self.transaction_intent_message = Some(message);
        self
    }

    pub fn intent_header(mut self, intent_header: IntentHeaderV2) -> Self {
        self.transaction_intent_header = Some(intent_header);
        self
    }

    pub fn transaction_header(mut self, transaction_header: TransactionHeaderV2) -> Self {
        self.transaction_header = Some(transaction_header);
        self
    }

    pub fn create_intent_and_subintent_info(&mut self) -> &TransactionIntentV2 {
        if self
            .transaction_intent_and_non_root_subintent_signatures
            .is_none()
        {
            let manifest = self
                .transaction_intent_manifest
                .take()
                .expect("Manifest must be provided before this action is performed");
            let (instructions, blobs, child_hashes, root_object_names) =
                manifest.for_intent_with_names();
            let subintents = mem::take(&mut self.child_partial_transactions);

            let mut aggregated_subintents = vec![];
            let mut aggregated_subintent_signatures = vec![];
            let mut aggregated_subintent_object_names = vec![];
            for (_name, (_hash, child_partial_transaction, object_names)) in subintents {
                let SignedPartialTransactionV2 {
                    partial_transaction,
                    root_subintent_signatures: root_intent_signatures,
                    non_root_subintent_signatures: subintent_signatures,
                } = child_partial_transaction;
                let TransactionObjectNames {
                    root_intent: root_intent_names,
                    subintents: subintent_names,
                } = object_names;
                aggregated_subintents.push(partial_transaction.root_subintent);
                aggregated_subintents.extend(partial_transaction.non_root_subintents.0);
                aggregated_subintent_signatures.push(root_intent_signatures);
                aggregated_subintent_signatures.extend(subintent_signatures.by_subintent);
                aggregated_subintent_object_names.push(root_intent_names);
                aggregated_subintent_object_names.extend(subintent_names);
            }
            let intent =
                TransactionIntentV2 {
                    transaction_header: self.transaction_header.take().expect(
                        "Transaction Header must be provided before this action is performed",
                    ),
                    root_intent_core: IntentCoreV2 {
                        header: self.transaction_intent_header.take().expect(
                            "Intent Header must be provided before this action is performed",
                        ),
                        blobs,
                        message: self.transaction_intent_message.take().unwrap_or_default(),
                        children: child_hashes,
                        instructions,
                    },
                    non_root_subintents: NonRootSubintentsV2(aggregated_subintents),
                };
            let subintent_signatures = NonRootSubintentSignaturesV2 {
                by_subintent: aggregated_subintent_signatures,
            };
            self.transaction_intent_and_non_root_subintent_signatures =
                Some((intent, subintent_signatures));
            self.transaction_intent_object_names = Some(TransactionObjectNames {
                root_intent: root_object_names,
                subintents: aggregated_subintent_object_names,
            });
        }
        &self
            .transaction_intent_and_non_root_subintent_signatures
            .as_ref()
            .unwrap()
            .0
    }

    pub fn create_prepared_intent(&mut self) -> &PreparedTransactionIntentV2 {
        if self.prepared_transaction_intent.is_none() {
            let prepared = self
                .create_intent_and_subintent_info()
                .prepare(PreparationSettings::latest_ref())
                .expect("Expected that the intent could be prepared");
            self.prepared_transaction_intent = Some(prepared);
        }
        self.prepared_transaction_intent.as_ref().unwrap()
    }

    pub fn intent_hash(&mut self) -> TransactionIntentHash {
        self.create_prepared_intent().transaction_intent_hash()
    }

    pub fn sign<S: Signer>(mut self, signer: &S) -> Self {
        let hash = self.intent_hash();
        self.transaction_intent_signatures
            .push(IntentSignatureV1(signer.sign_with_public_key(&hash)));
        self
    }

    pub fn multi_sign<S: Signer>(mut self, signers: &[&S]) -> Self {
        let hash = self.intent_hash();
        for signer in signers {
            self.transaction_intent_signatures
                .push(IntentSignatureV1(signer.sign_with_public_key(&hash)));
        }
        self
    }

    pub fn add_signature(mut self, signature: SignatureWithPublicKeyV1) -> Self {
        self.transaction_intent_signatures
            .push(IntentSignatureV1(signature));
        self
    }

    pub fn create_signed_transaction_intent(&mut self) -> &SignedTransactionIntentV2 {
        if self.signed_transaction_intent.is_none() {
            self.create_intent_and_subintent_info();
            let (root_intent, subintent_signatures) = self
                .transaction_intent_and_non_root_subintent_signatures
                .take()
                .expect("Intent was created in create_intent_and_subintent_info()");
            let signatures = mem::take(&mut self.transaction_intent_signatures);
            let signed_intent = SignedTransactionIntentV2 {
                transaction_intent: root_intent,
                transaction_intent_signatures: IntentSignaturesV2 { signatures },
                non_root_subintent_signatures: subintent_signatures,
            };
            self.signed_transaction_intent = Some(signed_intent);
        }
        self.signed_transaction_intent.as_ref().unwrap()
    }

    pub fn create_prepared_signed_transaction_intent(
        &mut self,
    ) -> &PreparedSignedTransactionIntentV2 {
        if self.prepared_signed_transaction_intent.is_none() {
            let prepared = self
                .create_signed_transaction_intent()
                .prepare(PreparationSettings::latest_ref())
                .expect("Expected that signed intent could be prepared");
            self.prepared_signed_transaction_intent = Some(prepared);
        }
        self.prepared_signed_transaction_intent.as_ref().unwrap()
    }

    pub fn notarize<S: Signer>(mut self, signer: &S) -> Self {
        let hash = self
            .create_prepared_signed_transaction_intent()
            .signed_transaction_intent_hash();
        self.notary_signature = Some(NotarySignatureV2(
            signer.sign_with_public_key(&hash).signature(),
        ));
        self
    }

    pub fn notary_signature(mut self, signature: SignatureV1) -> Self {
        self.notary_signature = Some(NotarySignatureV2(signature));
        self
    }

    /// Builds the [`NotarizedTransactionV2`], also returning the [`TransactionObjectNames`]
    /// used for manifest variables in the root transaction intent and non-root subintents.
    pub fn build_with_names_no_validate(self) -> (NotarizedTransactionV2, TransactionObjectNames) {
        let transaction = NotarizedTransactionV2 {
            signed_transaction_intent: self
                .signed_transaction_intent
                .expect("Expected signed intent to exist"),
            notary_signature: self
                .notary_signature
                .expect("Expected notary signature to exist"),
        };
        (transaction, self.transaction_intent_object_names.unwrap())
    }

    /// Builds and validates the [`NotarizedTransactionV2`], also returning the [`TransactionObjectNames`]
    /// used for manifest variables in the root transaction intent and non-root subintents.
    ///
    /// # Panics
    /// Panics if the built transaction does not pass validation with the latest validator.
    ///
    /// You can use [`build_with_names_no_validate()`][Self::build_with_names_no_validate] to skip this validation.
    pub fn build_with_names(self) -> (NotarizedTransactionV2, TransactionObjectNames) {
        let (transaction, names) = self.build_with_names_no_validate();
        let validator = TransactionValidator::new_with_latest_config_network_agnostic();
        transaction.prepare_and_validate(&validator)
            .expect("Built transaction should be valid. Use `build_with_names_no_validate()` to skip validation if needed.");
        (transaction, names)
    }

    pub fn build_no_validate(self) -> NotarizedTransactionV2 {
        self.build_with_names_no_validate().0
    }

    /// Builds and validates the [`NotarizedTransactionV2`].
    ///
    /// If you wish to keep a record of the names used in the manifest variables of the transaction intent or any
    /// non-root subintents, use [`build_with_names()`][Self::build_with_names] instead.
    ///
    /// # Panics
    /// Panics if the built transaction does not pass validation with the latest validator.
    ///
    /// You can use [`build_no_validate()`][Self::build_no_validate] to skip this validation.
    pub fn build(self) -> NotarizedTransactionV2 {
        let transaction = self.build_no_validate();
        let validator = TransactionValidator::new_with_latest_config_network_agnostic();
        transaction.prepare_and_validate(&validator)
            .expect("Built transaction should be valid. Use `build_no_validate()` to skip validation if needed.");
        transaction
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
