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

impl TransactionBuilder {
    // In symmetry with the ManifestBuilder, we add in some methods on the V1 builder
    // to create the V2 builders.

    pub fn new_v2() -> TransactionV2Builder {
        TransactionV2Builder::new()
    }

    pub fn new_partial_v2() -> PartialTransactionV2Builder {
        PartialTransactionV2Builder::new()
    }
}

#[derive(Clone)]
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

    pub fn then(self, next: impl FnOnce(Self) -> Self) -> Self {
        next(self)
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

    pub fn sign<S: Signer>(mut self, signer: S) -> Self {
        let intent = self.transaction_intent();
        let prepared = intent
            .prepare(PreparationSettings::latest_ref())
            .expect("Intent could be prepared");
        self.intent_signatures
            .push(signer.sign_with_public_key(&prepared.transaction_intent_hash()));
        self
    }

    pub fn multi_sign<S: Signer>(mut self, signers: impl IntoIterator<Item = S>) -> Self {
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

    pub fn notarize<S: Signer>(mut self, signer: S) -> Self {
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

// We alow either to avoid confusion
pub type SignedPartialTransactionV2Builder = PartialTransactionV2Builder;

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
///   * Use method [`build`][Self::build], [`build_no_validate`][Self::build_no_validate],
///     [`build_minimal`][Self::build_minimal] or [`build_minimal_no_validate`][Self::build_minimal_no_validate]
///
/// The error messages aren't great if used out of order.
/// In future, this may become a state-machine style builder, to catch more errors at compile time.
#[derive(Default, Clone)]
pub struct PartialTransactionV2Builder {
    pub(crate) child_partial_transactions: IndexMap<
        String,
        (
            SubintentHash,
            SignedPartialTransactionV2,
            TransactionObjectNames,
        ),
    >,
    pub(crate) root_subintent_header: Option<IntentHeaderV2>,
    pub(crate) root_subintent_message: Option<MessageV2>,
    pub(crate) root_subintent_manifest: Option<SubintentManifestV2>,
    // Cached once created
    root_subintent: Option<(SubintentV2, ManifestObjectNames)>,
    prepared_root_subintent: Option<PreparedSubintentV2>,
    root_subintent_signatures: Vec<IntentSignatureV1>,
}

impl PartialTransactionV2Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn then(self, next: impl FnOnce(Self) -> Self) -> Self {
        next(self)
    }

    /// When used with the [`manifest_builder`][Self::manifest_builder] method, the provided name and hash
    /// are provided automatically via [`use_child`][ManifestBuilder::use_child] at the start of manifest creation.
    ///
    /// When used with the [`manifest`][Self::manifest] method, the provided name is simply ignored - names
    /// are returned from the provided manifest.
    pub fn add_signed_child(
        mut self,
        name: impl AsRef<str>,
        signed_partial_transaction: impl ResolvableSignedPartialTransaction,
    ) -> Self {
        if self.root_subintent_manifest.is_some() {
            panic!("Call add_signed_child before calling manifest or manifest_builder");
        }
        let (signed_partial_transaction, object_names, root_subintent_hash) =
            signed_partial_transaction.resolve();

        let name = name.as_ref();
        let replaced = self.child_partial_transactions.insert(
            name.to_string(),
            (
                root_subintent_hash,
                signed_partial_transaction,
                object_names,
            ),
        );
        if replaced.is_some() {
            panic!("Child with name {name} already exists");
        }
        self
    }

    /// If the intent has any children, you should call [`add_signed_child`][Self::add_signed_child] first.
    /// These children will get added to the manifest for you, with the corresponding names.
    ///
    /// You may also want to try [`manifest_builder_with_lookup`][Self::manifest_builder_with_lookup].
    pub fn manifest_builder(
        mut self,
        build_manifest: impl FnOnce(SubintentManifestV2Builder) -> SubintentManifestV2Builder,
    ) -> Self {
        let mut manifest_builder = SubintentManifestV2Builder::new_typed();
        for (child_name, (hash, _, _)) in self.child_partial_transactions.iter() {
            manifest_builder = manifest_builder.use_child(child_name, *hash);
        }
        // The manifest will be validated as part of the transaction builder validation.
        self.root_subintent_manifest = Some(build_manifest(manifest_builder).build_no_validate());
        self
    }

    /// If the intent has any children, you should call [`add_signed_child`][Self::add_signed_child] first.
    /// These children will get added to the manifest for you, with the corresponding names.
    pub fn manifest_builder_with_lookup(
        mut self,
        build_manifest: impl FnOnce(
            SubintentManifestV2Builder,
            ManifestNameLookup,
        ) -> SubintentManifestV2Builder,
    ) -> Self {
        let mut manifest_builder = SubintentManifestV2Builder::new_typed();
        let name_lookup = manifest_builder.name_lookup();
        for (child_name, (hash, _, _)) in self.child_partial_transactions.iter() {
            manifest_builder = manifest_builder.use_child(child_name, *hash);
        }
        // The manifest will be validated as part of the transaction builder validation.
        self.root_subintent_manifest =
            Some(build_manifest(manifest_builder, name_lookup).build_no_validate());
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

    pub fn sign<S: Signer>(mut self, signer: S) -> Self {
        let hash = self.subintent_hash();
        self.root_subintent_signatures
            .push(IntentSignatureV1(signer.sign_with_public_key(&hash)));
        self
    }

    pub fn multi_sign<S: Signer>(mut self, signers: impl IntoIterator<Item = S>) -> Self {
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

    fn build_internal(mut self) -> (SignedPartialTransactionV2, TransactionObjectNames) {
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

    /// Builds a [`DetailedSignedPartialTransactionV2`], including the [`SignedPartialTransactionV2`]
    /// and other useful data.
    ///
    /// # Panics
    /// * Panics if any required part of the partial transaction has not been provided.
    /// * Panics if the built transaction cannot be prepared.
    ///
    /// Unlike [`TransactionV2Builder`], `build()` does not validate the partial transaction, to save
    /// lots duplicate work when building a full transaction from layers of partial transaction. If you wish,
    /// you can opt into validation with [`build_and_validate()`][Self::build_and_validate].
    pub fn build(self) -> DetailedSignedPartialTransactionV2 {
        let (partial_transaction, object_names) = self.build_internal();
        let prepared = partial_transaction
            .prepare(PreparationSettings::latest_ref())
            .expect("Transaction must be preparable");
        DetailedSignedPartialTransactionV2 {
            partial_transaction,
            object_names,
            root_subintent_hash: prepared.subintent_hash(),
            non_root_subintent_hashes: prepared.non_root_subintent_hashes().collect(),
        }
    }

    /// Builds a [`DetailedSignedPartialTransactionV2`], including the [`SignedPartialTransactionV2`]
    /// and other useful data.
    ///
    /// # Panics
    /// * Panics if any required part of the partial transaction has not been provided.
    /// * Panics if the built transaction does not pass validation with the latest validator.
    ///
    /// You can use [`build()`][Self::build] to skip this validation.
    pub fn build_and_validate(self) -> DetailedSignedPartialTransactionV2 {
        let (partial_transaction, object_names) = self.build_internal();
        let validator = TransactionValidator::new_with_latest_config_network_agnostic();
        let validated = partial_transaction.prepare_and_validate(&validator)
            .expect("Built partial transaction should be valid. Use `build()` to skip validation if needed.");
        DetailedSignedPartialTransactionV2 {
            partial_transaction,
            object_names,
            root_subintent_hash: validated.prepared.subintent_hash(),
            non_root_subintent_hashes: validated.prepared.non_root_subintent_hashes().collect(),
        }
    }

    /// Builds the [`SignedPartialTransactionV2`].
    ///
    /// You may wish to use [`build_detailed()`][Self::build_detailed] to get the hashes, or to
    /// preserve the names used for manifest variables in the root and non-root subintents.
    ///
    /// Unlike [`TransactionV2Builder`], `build()` does not validate the partial transaction, to save
    /// lots duplicate work when building a full transaction from layers of partial transaction. If you wish,
    /// you can opt into validation with [`build_and_validate()`][Self::build_and_validate].
    pub fn build_minimal(self) -> SignedPartialTransactionV2 {
        self.build_internal().0
    }

    /// Builds and validates the [`SignedPartialTransactionV2`].
    ///
    /// You may wish to use [`build()`][Self::build] to get the hashes, or to
    /// preserve the names used for manifest variables in the root and non-root subintents.
    ///
    /// # Panics
    /// Panics if the built transaction does not pass validation with the latest validator.
    ///
    /// You can use [`build_minimal()`][Self::build_minimal] to skip this validation.
    pub fn build_minimal_and_validate(self) -> SignedPartialTransactionV2 {
        let transaction = self.build_minimal();
        let validator = TransactionValidator::new_with_latest_config_network_agnostic();
        transaction.prepare_and_validate(&validator)
            .expect("Built partial transaction should be valid. Use `build()` to skip validation if needed.");
        transaction
    }
}

/// Includes:
/// * A full [`SignedPartialTransactionV2`], which can be turned into a raw notarized transaction.
/// * The [`TransactionObjectNames`], capturing the manifest names in the
///   root subintent and non-root subintents (assuming `build_detailed` was used at all
///   steps when building the transaction, else this detail can be lost).
/// * The various subintenthashes of the partial transaction.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DetailedSignedPartialTransactionV2 {
    pub partial_transaction: SignedPartialTransactionV2,
    pub object_names: TransactionObjectNames,
    pub root_subintent_hash: SubintentHash,
    pub non_root_subintent_hashes: IndexSet<SubintentHash>,
}

impl DetailedSignedPartialTransactionV2 {
    pub fn to_raw(&self) -> Result<RawSignedPartialTransaction, EncodeError> {
        self.partial_transaction.to_raw()
    }
}

pub trait ResolvableSignedPartialTransaction {
    fn resolve(
        self,
    ) -> (
        SignedPartialTransactionV2,
        TransactionObjectNames,
        SubintentHash,
    );
}

impl ResolvableSignedPartialTransaction for DetailedSignedPartialTransactionV2 {
    fn resolve(
        self,
    ) -> (
        SignedPartialTransactionV2,
        TransactionObjectNames,
        SubintentHash,
    ) {
        (
            self.partial_transaction,
            self.object_names,
            self.root_subintent_hash,
        )
    }
}

impl ResolvableSignedPartialTransaction for SignedPartialTransactionV2 {
    fn resolve(
        self,
    ) -> (
        SignedPartialTransactionV2,
        TransactionObjectNames,
        SubintentHash,
    ) {
        let object_names = TransactionObjectNames::unknown_with_subintent_count(
            self.non_root_subintent_signatures.by_subintent.len(),
        );

        let subintent_hash = self
            .prepare(PreparationSettings::latest_ref())
            .expect("Child signed partial transation could not be prepared")
            .subintent_hash();

        (self, object_names, subintent_hash)
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
///     [`build_minimal`][Self::build_minimal] or [`build_minimal_no_validate`][Self::build_minimal_no_validate]
///
/// The error messages aren't great if used out of order.
/// In future, this may become a state-machine style builder, to catch more errors at compile time.
#[derive(Default, Clone)]
pub struct TransactionV2Builder {
    // Note - these names are long, but agreed with Yulong that we would clarify
    // non_root_subintents from root_subintent / transaction_intent so this is
    // applying that logic to these field names
    pub(crate) child_partial_transactions: IndexMap<
        String,
        (
            SubintentHash,
            SignedPartialTransactionV2,
            TransactionObjectNames,
        ),
    >,
    pub(crate) transaction_header: Option<TransactionHeaderV2>,
    pub(crate) transaction_intent_header: Option<IntentHeaderV2>,
    pub(crate) transaction_intent_message: Option<MessageV2>,
    pub(crate) transaction_intent_manifest: Option<TransactionManifestV2>,
    // Temporarily cached once created
    transaction_intent_and_non_root_subintent_signatures:
        Option<(TransactionIntentV2, NonRootSubintentSignaturesV2)>,
    transaction_object_names: Option<TransactionObjectNames>,
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

    pub fn then(self, next: impl FnOnce(Self) -> Self) -> Self {
        next(self)
    }

    /// When used with the [`manifest_builder`][Self::manifest_builder] method, the provided name and hash
    /// are provided automatically via [`use_child`][ManifestBuilder::use_child] at the start of manifest creation.
    ///
    /// When used with the [`manifest`][Self::manifest] method, the provided name is simply ignored - names
    /// are returned from the provided manifest.
    pub fn add_signed_child(
        mut self,
        name: impl AsRef<str>,
        signed_partial_transaction: impl ResolvableSignedPartialTransaction,
    ) -> Self {
        if self.transaction_intent_manifest.is_some() {
            panic!("Call add_signed_child before calling manifest or manifest_builder");
        }

        let (signed_partial_transaction, object_names, root_subintent_hash) =
            signed_partial_transaction.resolve();

        let name = name.as_ref();
        let replaced = self.child_partial_transactions.insert(
            name.to_string(),
            (
                root_subintent_hash,
                signed_partial_transaction,
                object_names,
            ),
        );
        if replaced.is_some() {
            panic!("Child with name {name} already exists");
        }
        self
    }

    /// If the intent has any children, you should call [`add_signed_child`][Self::add_signed_child] first.
    /// These children will get added to the manifest for you, with the corresponding names.
    ///
    /// You may also want to try [`manifest_builder_with_lookup`][Self::manifest_builder_with_lookup].
    pub fn manifest_builder(
        mut self,
        build_manifest: impl FnOnce(TransactionManifestV2Builder) -> TransactionManifestV2Builder,
    ) -> Self {
        let mut manifest_builder = TransactionManifestV2Builder::new_typed();
        for (child_name, (hash, _, _)) in self.child_partial_transactions.iter() {
            manifest_builder = manifest_builder.use_child(child_name, *hash);
        }
        // The manifest will be validated as part of the transaction builder validation.
        self.transaction_intent_manifest =
            Some(build_manifest(manifest_builder).build_no_validate());
        self
    }

    /// If the intent has any children, you should call [`add_signed_child`][Self::add_signed_child] first.
    /// These children will get added to the manifest for you, with the corresponding names.
    pub fn manifest_builder_with_lookup(
        mut self,
        build_manifest: impl FnOnce(
            TransactionManifestV2Builder,
            ManifestNameLookup,
        ) -> TransactionManifestV2Builder,
    ) -> Self {
        let mut manifest_builder = TransactionManifestV2Builder::new_typed();
        let name_lookup = manifest_builder.name_lookup();
        for (child_name, (hash, _, _)) in self.child_partial_transactions.iter() {
            manifest_builder = manifest_builder.use_child(child_name, *hash);
        }
        // The manifest will be validated as part of the transaction builder validation.
        self.transaction_intent_manifest =
            Some(build_manifest(manifest_builder, name_lookup).build_no_validate());
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
            self.transaction_object_names = Some(TransactionObjectNames {
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

    pub fn sign<S: Signer>(mut self, signer: S) -> Self {
        let hash = self.intent_hash();
        self.transaction_intent_signatures
            .push(IntentSignatureV1(signer.sign_with_public_key(&hash)));
        self
    }

    pub fn multi_sign<S: Signer>(mut self, signers: impl IntoIterator<Item = S>) -> Self {
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

    /// Creates a [`PreviewTransactionV2`]. For all non-root subintents, existing signatures
    /// are converted into the corresponding public key.
    ///
    /// ## Panics
    /// * Panics if any subintent signatures are not recoverable.
    ///   Untrusted partial transactions should be validated before using in the transaction builder.
    /// * If the resulting preview transaction could not be validated.
    pub fn build_preview_transaction(
        &mut self,
        transaction_intent_signer_keys: impl IntoIterator<Item = PublicKey>,
    ) -> PreviewTransactionV2 {
        let preview_transaction =
            self.build_preview_transaction_no_validate(transaction_intent_signer_keys);
        let validator = TransactionValidator::new_with_latest_config_network_agnostic();
        preview_transaction.prepare_and_validate(&validator)
            .expect("Built preview transaction should be valid. Use `build_preview_transaction_no_validate()` to skip validation if needed.");
        preview_transaction
    }

    /// Creates a [`PreviewTransactionV2`]. For all non-root subintents, existing signatures
    /// are converted into the corresponding public key.
    ///
    /// ## Panics
    /// * Panics if any subintent signatures are not recoverable.
    ///   Untrusted partial transactions should be validated before using in the transaction builder.
    pub fn build_preview_transaction_no_validate(
        &mut self,
        transaction_intent_signer_keys: impl IntoIterator<Item = PublicKey>,
    ) -> PreviewTransactionV2 {
        self.create_intent_and_subintent_info();
        let (transaction_intent, subintent_signatures) = self
            .transaction_intent_and_non_root_subintent_signatures
            .clone()
            .take()
            .expect("Intent was created in create_intent_and_subintent_info()");

        // Extract the public keys from the subintent signatures for preview purposes.
        let non_root_subintent_signer_public_keys = subintent_signatures
            .by_subintent
            .into_iter()
            .enumerate()
            .map(|(subintent_index, sigs)| {
                sigs.signatures
                    .into_iter()
                    .map(|signature| match signature.0 {
                        SignatureWithPublicKeyV1::Secp256k1 { .. } => {
                            let subintent = transaction_intent.non_root_subintents.0.get(subintent_index).unwrap();
                            let subintent_hash = subintent.prepare(&PreparationSettings::latest())
                                .expect("Untrusted partial transactions should be validated before using with the builder")
                                .subintent_hash();
                            verify_and_recover(subintent_hash.as_hash(), &signature.0)
                                .expect("Signature was not valid")
                        }
                        SignatureWithPublicKeyV1::Ed25519 { public_key, .. } => public_key.into(),
                    })
                    .collect()
            })
            .collect();

        PreviewTransactionV2 {
            transaction_intent,
            root_signer_public_keys: transaction_intent_signer_keys.into_iter().collect(),
            non_root_subintent_signer_public_keys,
        }
    }

    fn build_internal(self) -> (NotarizedTransactionV2, TransactionObjectNames) {
        let notary_signature = self
            .notary_signature
            .expect("Expected notary signature to exist - ensure you call `notarize` first");
        let transaction = NotarizedTransactionV2 {
            signed_transaction_intent: self
                .signed_transaction_intent
                .expect("If the notary signature exists, the signed intent should already have been populated"),
            notary_signature,
        };
        let object_names = self.transaction_object_names.expect(
            "If the signed intent exists, the object names should have already been populated",
        );
        (transaction, object_names)
    }

    /// Builds a [`DetailedNotarizedTransactionV2`], including the [`NotarizedTransactionV2`]
    /// and other useful data.
    ///
    /// # Panics
    /// * If the builder has not been notarized
    /// * If the transaction is not preparable against latest settings (e.g. it is too big)
    pub fn build_no_validate(self) -> DetailedNotarizedTransactionV2 {
        let (transaction, object_names) = self.build_internal();
        let raw = transaction.to_raw().expect("Transaction must be encodable");
        let prepared = raw
            .prepare(PreparationSettings::latest_ref())
            .expect("Transaction must be preparable");
        DetailedNotarizedTransactionV2 {
            transaction,
            raw,
            object_names,
            transaction_hashes: prepared.hashes(),
        }
    }

    /// Builds and validates a [`DetailedNotarizedTransactionV2`], which includes
    /// a [`NotarizedTransactionV2`] and other useful data.
    ///
    /// # Panics
    /// * Panics if the built transaction does not pass validation with the latest validator.
    ///
    /// You can use [`build_no_validate()`][Self::build_no_validate] to skip this validation.
    pub fn build(self) -> DetailedNotarizedTransactionV2 {
        let (transaction, object_names) = self.build_internal();
        let validator = TransactionValidator::new_with_latest_config_network_agnostic();
        let raw = transaction.to_raw().expect("Transaction must be encodable");
        let validated = raw.validate_as_known_v2(&validator)
            .expect("Built transaction should be valid. Use `build_no_validate()` to skip validation if needed.");
        DetailedNotarizedTransactionV2 {
            transaction,
            raw,
            object_names,
            transaction_hashes: validated.prepared.hashes(),
        }
    }

    /// Builds the [`NotarizedTransactionV2`], with no validation.
    pub fn build_minimal_no_validate(self) -> NotarizedTransactionV2 {
        self.build_internal().0
    }

    /// Builds and validates the [`NotarizedTransactionV2`].
    ///
    /// You may prefer [`build()`][Self::build] instead if you need the transaction hashes,
    /// or wish to keep a record of the names used in the manifest variables.
    ///
    /// # Panics
    /// Panics if the built transaction does not pass validation with the latest validator.
    ///
    /// You can use [`build_minimal_no_validate()`][Self::build_minimal_no_validate] to skip this validation.
    pub fn build_minimal(self) -> NotarizedTransactionV2 {
        let transaction = self.build_minimal_no_validate();
        let validator = TransactionValidator::new_with_latest_config_network_agnostic();
        transaction.prepare_and_validate(&validator)
            .expect("Built transaction should be valid. Use `build_no_validate()` to skip validation if needed.");
        transaction
    }
}

/// Includes:
/// * A full [`NotarizedTransactionV2`], which can be turned into a raw notarized transaction.
/// * The [`TransactionObjectNames`], capturing the manifest names in the
///   root subintent and non-root subintents (assuming `build_detailed` was used at all
///   steps when building the transaction, else this detail can be lost).
/// * The various hashes of the transaction.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DetailedNotarizedTransactionV2 {
    pub transaction: NotarizedTransactionV2,
    pub raw: RawNotarizedTransaction,
    pub object_names: TransactionObjectNames,
    pub transaction_hashes: UserTransactionHashes,
}

impl IntoExecutable for DetailedNotarizedTransactionV2 {
    type Error = TransactionValidationError;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        self.raw.into_executable(validator)
    }
}

impl AsRef<RawNotarizedTransaction> for DetailedNotarizedTransactionV2 {
    fn as_ref(&self) -> &RawNotarizedTransaction {
        &self.raw
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
