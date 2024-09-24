use crate::internal_prelude::*;

#[derive(ManifestSbor)]
pub enum TestTransaction {
    V1(TestIntentV1),
    V2 {
        root_intent: TestIntentV2,
        subintents: Vec<TestIntentV2>,
    },
}

pub struct TestTransactionV2Builder {
    nonce: u32,
    subintents: IndexMap<SubintentHash, TestIntentV2>,
}

impl TestTransactionV2Builder {
    pub fn new(nonce: u32) -> Self {
        Self {
            nonce,
            subintents: Default::default(),
        }
    }

    pub fn add_subintent(
        &mut self,
        manifest: SubintentManifestV2,
        proofs: impl IntoIterator<Item = NonFungibleGlobalId>,
    ) -> SubintentHash {
        let (instructions, blobs, child_intents) = manifest.for_intent();
        let intent = self.create_intent(instructions, blobs, child_intents, proofs);
        let hash = intent.hash;
        self.subintents.insert(SubintentHash(hash), intent);
        SubintentHash(hash)
    }

    pub fn finish_with_root_intent(
        self,
        manifest: TransactionManifestV2,
        proofs: impl IntoIterator<Item = NonFungibleGlobalId>,
    ) -> TestTransaction {
        let (instructions, blobs, child_intents) = manifest.for_intent();
        let root_intent = self.create_intent(instructions, blobs, child_intents, proofs);
        TestTransaction::V2 {
            root_intent,
            subintents: self.subintents.into_values().collect(),
        }
    }

    fn create_intent(
        &self,
        instructions: InstructionsV2,
        blobs: BlobsV1,
        child_intents: ChildIntentsV2,
        proofs: impl IntoIterator<Item = NonFungibleGlobalId>,
    ) -> TestIntentV2 {
        let children_intent_indices = child_intents
            .children
            .into_iter()
            .map(|child| {
                let subintent_index = self
                    .subintents
                    .get_index_of(&child.hash)
                    .expect("Child subintents should exist already in the Test Transaction");
                let thread_index = subintent_index + 1;
                thread_index
            })
            .collect();
        let nonce = self.nonce;
        let subintent_count = self.subintents.len();
        let hash = hash(format!(
            "Test transaction intent: {nonce} - {subintent_count}"
        ));
        TestIntentV2 {
            instructions,
            blobs,
            hash,
            initial_proofs: proofs.into_iter().collect(),
            children_intent_indices,
        }
    }
}

#[derive(ManifestSbor)]
pub struct TestIntentV1 {
    pub instructions: InstructionsV1,
    pub blobs: BlobsV1,
    pub hash: Hash,
    pub initial_proofs: BTreeSet<NonFungibleGlobalId>,
}

#[derive(ManifestSbor)]
pub struct TestIntentV2 {
    pub instructions: InstructionsV2,
    pub blobs: BlobsV1,
    pub hash: Hash,
    pub initial_proofs: BTreeSet<NonFungibleGlobalId>,
    pub children_intent_indices: Vec<usize>,
}

#[derive(ManifestSbor)]
pub enum PreparedTestTransaction {
    V1(PreparedTestIntent),
    V2(Vec<PreparedTestIntent>),
}

#[derive(ManifestSbor)]
pub struct PreparedTestIntent {
    pub encoded_instructions: Rc<Vec<u8>>,
    pub references: IndexSet<Reference>,
    pub blobs: Rc<IndexMap<Hash, Vec<u8>>>,
    pub hash: Hash,
    pub children_intent_indices: Vec<usize>,
    pub initial_proofs: BTreeSet<NonFungibleGlobalId>,
}

impl PreparedTestIntent {
    #[allow(deprecated)]
    pub fn from_v1(
        intent: TestIntentV1,
        settings: &PreparationSettings,
    ) -> Result<Self, PrepareError> {
        let prepared_instructions = intent.instructions.prepare_partial(settings)?;
        Ok(PreparedTestIntent {
            encoded_instructions: Rc::new(manifest_encode(&prepared_instructions.inner.0)?),
            references: prepared_instructions.references,
            blobs: intent.blobs.prepare_partial(settings)?.blobs_by_hash,
            hash: intent.hash,
            children_intent_indices: vec![],
            initial_proofs: intent.initial_proofs,
        })
    }

    pub fn from_v2(
        intent: TestIntentV2,
        settings: &PreparationSettings,
    ) -> Result<Self, PrepareError> {
        let prepared_instructions = intent.instructions.prepare_partial(settings)?;
        Ok(PreparedTestIntent {
            encoded_instructions: Rc::new(manifest_encode(&prepared_instructions.inner.0)?),
            references: prepared_instructions.references.as_ref().clone(),
            blobs: intent.blobs.prepare_partial(settings)?.blobs_by_hash,
            hash: intent.hash,
            children_intent_indices: intent.children_intent_indices,
            initial_proofs: intent.initial_proofs,
        })
    }
}

impl TestTransaction {
    /// The nonce needs to be globally unique amongst test transactions on your ledger
    pub fn new_v1_from_nonce(
        manifest: TransactionManifestV1,
        nonce: u32,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> Self {
        Self::new_v1(
            manifest,
            hash(format!("Test transaction: {}", nonce)),
            initial_proofs,
        )
    }

    pub fn new_v1(
        manifest: TransactionManifestV1,
        hash: Hash,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> Self {
        let (instructions, blobs) = manifest.for_intent();
        Self::V1(TestIntentV1 {
            instructions,
            blobs,
            hash,
            initial_proofs,
        })
    }

    /// ## Example usage
    /// ```ignore
    /// # // Ignored as it depends on scrypto_test which isn't a dev dependency
    /// let mut ledger = LedgerSimulatorBuilder::new().build();
    /// let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());
    ///
    /// let child = builder.add_subintent(
    ///     ManifestBuilder::new_subintent_v2()
    ///         .yield_to_parent(())
    ///         .build(),
    ///     [child_public_key.signature_proof()],
    /// );
    ///
    /// let transaction = builder.finish_with_root_intent(
    ///     ManifestBuilder::new_v2()
    ///         .use_child("child", child)
    ///         .lock_standard_test_fee(account)
    ///         .yield_to_child("child", ())
    ///         .build(),
    ///     [public_key.signature_proof()],
    /// );
    ///
    /// let receipt = ledger.execute_test_transaction(transaction);
    /// ```
    pub fn new_v2_builder(nonce: u32) -> TestTransactionV2Builder {
        TestTransactionV2Builder::new(nonce)
    }

    #[allow(deprecated)]
    pub fn prepare(
        self,
        settings: &PreparationSettings,
    ) -> Result<PreparedTestTransaction, PrepareError> {
        match self {
            Self::V1(intent) => Ok(PreparedTestTransaction::V1(PreparedTestIntent::from_v1(
                intent, settings,
            )?)),
            Self::V2 {
                root_intent,
                subintents,
            } => {
                let mut prepared = vec![PreparedTestIntent::from_v2(root_intent, settings)?];
                for intent in subintents {
                    prepared.push(PreparedTestIntent::from_v2(intent, settings)?);
                }

                Ok(PreparedTestTransaction::V2(prepared))
            }
        }
    }
}

impl IntoExecutable for TestTransaction {
    type Error = PrepareError;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        Ok(self
            .prepare(validator.preparation_settings())?
            .get_executable())
    }
}

impl PreparedTestTransaction {
    pub fn get_executable(&self) -> ExecutableTransaction {
        match self {
            PreparedTestTransaction::V1(intent) => {
                let num_of_signature_validations = intent.initial_proofs.len() + 1;
                ExecutableTransaction::new_v1(
                    intent.encoded_instructions.clone(),
                    AuthZoneInit::proofs(intent.initial_proofs.clone()),
                    intent.references.clone(),
                    intent.blobs.clone(),
                    ExecutionContext {
                        unique_hash: intent.hash,
                        intent_hash_nullifications: vec![],
                        epoch_range: None,
                        payload_size: intent.encoded_instructions.len()
                            + intent.blobs.values().map(|x| x.len()).sum::<usize>(),
                        // For testing purpose, assume `num_of_signature_validations = num_of_initial_proofs + 1`
                        num_of_signature_validations,
                        costing_parameters: TransactionCostingParameters {
                            tip: TipSpecifier::None,
                            free_credit_in_xrd: Decimal::ZERO,
                            abort_when_loan_repaid: false,
                        },
                        pre_allocated_addresses: vec![],
                        disable_limits_and_costing_modules: false,
                        start_timestamp_inclusive: None,
                        end_timestamp_exclusive: None,
                    },
                )
            }
            PreparedTestTransaction::V2(intents) => {
                let payload_size = intents
                    .iter()
                    .map(|intent| {
                        intent.encoded_instructions.len()
                            + intent.blobs.values().map(|x| x.len()).sum::<usize>()
                    })
                    .sum();
                let num_of_signature_validations = intents
                    .iter()
                    .map(|intent| intent.initial_proofs.len())
                    .sum();

                let context = ExecutionContext {
                    unique_hash: intents.get(0).unwrap().hash,
                    intent_hash_nullifications: vec![],
                    epoch_range: None,
                    payload_size,
                    // For testing purpose, assume `num_of_signature_validations = num_of_initial_proofs + 1`
                    num_of_signature_validations,
                    costing_parameters: TransactionCostingParameters {
                        tip: TipSpecifier::None,
                        free_credit_in_xrd: Decimal::ZERO,
                        abort_when_loan_repaid: false,
                    },
                    pre_allocated_addresses: vec![],
                    disable_limits_and_costing_modules: false,
                    start_timestamp_inclusive: None,
                    end_timestamp_exclusive: None,
                };
                let intents = intents
                    .iter()
                    .map(|intent| {
                        let auth_zone_init = AuthZoneInit::proofs(intent.initial_proofs.clone());

                        ExecutableIntent {
                            encoded_instructions: intent.encoded_instructions.clone(),
                            auth_zone_init,
                            references: intent.references.clone(),
                            blobs: intent.blobs.clone(),
                            children_intent_indices: intent.children_intent_indices.clone(),
                        }
                    })
                    .collect();

                ExecutableTransaction::new_v2(intents, context)
            }
        }
    }
}

impl IntoExecutable for PreparedTestTransaction {
    type Error = core::convert::Infallible;

    fn into_executable(
        self,
        _validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        Ok(self.get_executable())
    }
}
