use crate::internal_prelude::*;
use core::ops::ControlFlow;

pub trait IntentStructure {
    fn intent_hash(&self) -> IntentHash;
    fn children(&self) -> impl ExactSizeIterator<Item = SubintentHash>;

    /// Should perform all the validation of the intent, except the relationship to other intents.
    fn validate_intent(
        &self,
        validator: &TransactionValidator,
        aggregation: &mut AcrossIntentAggregation,
    ) -> Result<ManifestYieldSummary, IntentValidationError>;
}

pub trait IntentTreeStructure {
    type RootIntentStructure: IntentStructure;
    type SubintentStructure: IntentStructure + HasSubintentHash;
    fn root(&self) -> &Self::RootIntentStructure;
    fn non_root_subintents<'a>(
        &'a self,
    ) -> impl ExactSizeIterator<Item = &'a Self::SubintentStructure>;
}

pub struct ValidatedIntentTreeInformation {
    pub intent_relationships: IntentRelationships,
    pub overall_validity_range: OverallValidityRangeV2,
    pub root_yield_summary: ManifestYieldSummary,
}

impl TransactionValidator {
    pub fn validate_intents_and_structure(
        &self,
        intent_tree: &impl IntentTreeStructure,
    ) -> Result<ValidatedIntentTreeInformation, TransactionValidationError> {
        let intent_relationships = self.validate_intent_relationships(intent_tree)?;

        let non_root_subintent_details = &intent_relationships.non_root_subintents;
        let mut aggregation = AcrossIntentAggregation::start();
        let mut yield_summaries: IndexMap<IntentHash, ManifestYieldSummary> =
            index_map_with_capacity(intent_tree.non_root_subintents().len() + 1);

        let root_yield_summary = {
            let root_intent_hash = intent_tree.root().intent_hash();
            let yield_summary = intent_tree
                .root()
                .validate_intent(self, &mut aggregation)
                .map_err(|err| {
                    TransactionValidationError::IntentValidationError(
                        TransactionValidationErrorLocation::for_root(root_intent_hash),
                        err,
                    )
                })?;
            yield_summaries.insert(root_intent_hash, yield_summary.clone());
            yield_summary
        };

        for (index, subintent) in intent_tree.non_root_subintents().enumerate() {
            let subintent_hash = subintent.subintent_hash();
            let yield_summary =
                subintent
                    .validate_intent(self, &mut aggregation)
                    .map_err(|err| {
                        TransactionValidationError::IntentValidationError(
                            TransactionValidationErrorLocation::NonRootSubintent(
                                SubintentIndex(index),
                                subintent_hash,
                            ),
                            err,
                        )
                    })?;
            yield_summaries.insert(subintent_hash.into(), yield_summary);
        }

        let overall_validity_range = aggregation.finalize(&self.config)?;

        for (child_hash, child_details) in non_root_subintent_details {
            let child_intent_hash = IntentHash::Subintent(*child_hash);
            // This checks that the YIELD_TO_PARENTs in a subintent match the YIELD_TO_CHILDS in the parent.
            // The instruction validation has already checked that the subintents end with a YIELD_TO_PARENT.
            let parent_yield_summary = yield_summaries.get(&child_details.parent).unwrap();
            let parent_yield_child_calls =
                *parent_yield_summary.child_yields.get(child_hash).unwrap();
            let child_yield_summary = yield_summaries.get(&child_intent_hash).unwrap();
            let child_yield_parent_calls = child_yield_summary.parent_yields;
            if parent_yield_child_calls != child_yield_parent_calls {
                return Err(
                    SubintentStructureError::MismatchingYieldChildAndYieldParentCountsForSubintent
                        .for_subintent(child_details.index, *child_hash),
                );
            }
        }

        Ok(ValidatedIntentTreeInformation {
            intent_relationships,
            overall_validity_range,
            root_yield_summary,
        })
    }

    /// The root intent can be either:
    /// * If validating a full transaction: a transaction intent
    /// * If validating a partial transaction: a root subintent
    fn validate_intent_relationships(
        &self,
        intent_tree: &impl IntentTreeStructure,
    ) -> Result<IntentRelationships, TransactionValidationError> {
        let mut root_intent_details = RootIntentRelationshipDetails::default();
        let mut non_root_subintent_details =
            IndexMap::<SubintentHash, SubintentRelationshipDetails>::default();

        // STEP 1
        // ------
        // * We establish that the non-root subintents are unique
        // * We create an index from the SubintentHash to SubintentIndex
        for (index, subintent) in intent_tree.non_root_subintents().enumerate() {
            let subintent_hash = subintent.subintent_hash();
            let index = SubintentIndex(index);
            let details = SubintentRelationshipDetails::default_for(index);
            if let Some(_) = non_root_subintent_details.insert(subintent_hash, details) {
                return Err(SubintentStructureError::DuplicateSubintent
                    .for_subintent(index, subintent_hash));
            }
        }

        // STEP 2
        // ------
        // We establish, for each parent intent, that each of its children:
        // * Exist as subintents in the transaction tree
        // * Has no other parents
        //
        // We also:
        // * Save the unique parent on each subintent which is a child
        // * Save the children of an intent into its intent details
        //
        // After this step, we know that each subintent has at most one parent.
        // We determine that every subintent has exactly one parent in step 4.

        // STEP 2A - Handle children of the root intent
        {
            let parent_hash = intent_tree.root().intent_hash();
            let intent_details = &mut root_intent_details;
            for child_hash in intent_tree.root().children() {
                let child_subintent_details = non_root_subintent_details
                    .get_mut(&child_hash)
                    .ok_or_else(|| {
                        SubintentStructureError::ChildSubintentNotIncludedInTransaction(child_hash)
                            .for_unindexed()
                    })?;
                if child_subintent_details.parent == PLACEHOLDER_PARENT {
                    child_subintent_details.parent = parent_hash;
                } else {
                    return Err(SubintentStructureError::SubintentHasMultipleParents
                        .for_subintent(child_subintent_details.index, child_hash));
                }
                intent_details.children.push(child_subintent_details.index);
            }
        }

        // STEP 2B - Handle the children of each subintent
        for subintent in intent_tree.non_root_subintents() {
            let subintent_hash = subintent.subintent_hash();
            let parent_hash: IntentHash = subintent_hash.into();
            let children = subintent.children();
            let mut children_details = Vec::with_capacity(children.len());
            for child_hash in children {
                let child_subintent_details = non_root_subintent_details
                    .get_mut(&child_hash)
                    .ok_or_else(|| {
                        SubintentStructureError::ChildSubintentNotIncludedInTransaction(child_hash)
                            .for_unindexed()
                    })?;
                if child_subintent_details.parent == PLACEHOLDER_PARENT {
                    child_subintent_details.parent = parent_hash;
                } else {
                    return Err(SubintentStructureError::SubintentHasMultipleParents
                        .for_subintent(child_subintent_details.index, child_hash));
                }
                children_details.push(child_subintent_details.index);
            }
            non_root_subintent_details
                .get_mut(&subintent_hash)
                .unwrap()
                .children = children_details;
        }

        // STEP 3
        // ------
        // We traverse the child relationships from the root, and mark a depth.
        // We error if any exceed the maximum depth.
        //
        // The iteration count is guaranteed to be bounded by the number of subintents because:
        // * Each subintent has at most one parent from step 2.
        // * Each parent -> child relationship is traversed at most once in the iteration.
        //   Quick proof by contradiction:
        //   - Assume not. Then some parent A is visited more than once.
        //   - Take the earliest such A in the iteration.
        //   - On both of its visits, A can only have been visited from its parent B.
        //   - But then B must have been visited more than once, contradicting the minimality of A.
        let mut work_list = vec![];
        for index in root_intent_details.children.iter() {
            work_list.push((*index, 1));
        }

        let max_depth = if intent_tree.root().intent_hash().is_for_subintent() {
            self.config.max_subintent_depth - 1
        } else {
            self.config.max_subintent_depth
        };

        loop {
            let Some((index, depth)) = work_list.pop() else {
                break;
            };
            if depth > max_depth {
                let (hash, _) = non_root_subintent_details.get_index(index.0).unwrap();
                return Err(
                    SubintentStructureError::SubintentExceedsMaxDepth.for_subintent(index, *hash)
                );
            }
            let (_, subintent_details) = non_root_subintent_details.get_index_mut(index.0).unwrap();
            subintent_details.depth = depth;
            for index in subintent_details.children.iter() {
                work_list.push((*index, depth + 1));
            }
        }

        // STEP 4
        // ------
        // We check that every subintent has a marked "depth from root".
        //
        // Combined with step 2 and step 3, we now have that:
        // * Every subintent has a unique parent.
        // * Every subintent is reachable from the root.
        //
        // Therefore there is a unique path from every subintent to the root, which implies
        // the subintents form a tree.
        for (hash, details) in non_root_subintent_details.iter() {
            if details.depth == 0 {
                return Err(
                    SubintentStructureError::SubintentIsNotReachableFromTheTransactionIntent
                        .for_subintent(details.index, *hash),
                );
            }
        }

        Ok(IntentRelationships {
            root_intent: root_intent_details,
            non_root_subintents: non_root_subintent_details,
        })
    }
}

// This type is public so it can be used by the toolkit.
#[must_use]
pub struct AcrossIntentAggregation {
    total_reference_count: usize,
    overall_start_epoch_inclusive: Epoch,
    overall_end_epoch_exclusive: Epoch,
    overall_start_timestamp_inclusive: Option<Instant>,
    overall_end_timestamp_exclusive: Option<Instant>,
}

impl AcrossIntentAggregation {
    pub fn start() -> Self {
        Self {
            total_reference_count: 0,
            overall_start_epoch_inclusive: Epoch::zero(),
            overall_end_epoch_exclusive: Epoch::of(u64::MAX),
            overall_start_timestamp_inclusive: None,
            overall_end_timestamp_exclusive: None,
        }
    }

    pub fn finalize(
        self,
        config: &TransactionValidationConfig,
    ) -> Result<OverallValidityRangeV2, TransactionValidationError> {
        if self.total_reference_count > config.max_total_references {
            return Err(TransactionValidationError::IntentValidationError(
                TransactionValidationErrorLocation::AcrossTransaction,
                IntentValidationError::TooManyReferences {
                    total: self.total_reference_count,
                    limit: config.max_total_references,
                },
            ));
        }
        Ok(OverallValidityRangeV2 {
            epoch_range: EpochRange {
                start_epoch_inclusive: self.overall_start_epoch_inclusive,
                end_epoch_exclusive: self.overall_end_epoch_exclusive,
            },
            proposer_timestamp_range: ProposerTimestampRange {
                start_timestamp_inclusive: self.overall_start_timestamp_inclusive,
                end_timestamp_exclusive: self.overall_end_timestamp_exclusive,
            },
        })
    }

    pub fn record_reference_count(
        &mut self,
        count: usize,
        config: &TransactionValidationConfig,
    ) -> Result<(), IntentValidationError> {
        if count > config.max_references_per_intent {
            return Err(IntentValidationError::TooManyReferences {
                total: count,
                limit: config.max_references_per_intent,
            });
        }
        self.total_reference_count = self.total_reference_count.saturating_add(count);
        Ok(())
    }

    pub fn update_headers(
        &mut self,
        start_epoch_inclusive: Epoch,
        end_epoch_exclusive: Epoch,
        start_timestamp_inclusive: Option<&Instant>,
        end_timestamp_exclusive: Option<&Instant>,
    ) -> Result<(), HeaderValidationError> {
        if start_epoch_inclusive > self.overall_start_epoch_inclusive {
            self.overall_start_epoch_inclusive = start_epoch_inclusive;
        }
        if end_epoch_exclusive < self.overall_end_epoch_exclusive {
            self.overall_end_epoch_exclusive = end_epoch_exclusive;
        }
        if self.overall_start_epoch_inclusive >= self.overall_end_epoch_exclusive {
            return Err(HeaderValidationError::NoValidEpochRangeAcrossAllIntents);
        }
        if let Some(start_timestamp_inclusive) = start_timestamp_inclusive {
            if self.overall_start_timestamp_inclusive.is_none()
                || self
                    .overall_start_timestamp_inclusive
                    .as_ref()
                    .is_some_and(|t| start_timestamp_inclusive > t)
            {
                self.overall_start_timestamp_inclusive = Some(*start_timestamp_inclusive);
            }
        }
        if let Some(end_timestamp_exclusive) = end_timestamp_exclusive {
            if self.overall_end_timestamp_exclusive.is_none()
                || self
                    .overall_end_timestamp_exclusive
                    .as_ref()
                    .is_some_and(|t| end_timestamp_exclusive < t)
            {
                self.overall_end_timestamp_exclusive = Some(*end_timestamp_exclusive);
            }
        }
        match (
            self.overall_start_timestamp_inclusive.as_ref(),
            self.overall_end_timestamp_exclusive.as_ref(),
        ) {
            (Some(start_inclusive), Some(end_exclusive)) => {
                if start_inclusive >= end_exclusive {
                    return Err(HeaderValidationError::NoValidTimestampRangeAcrossAllIntents);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct IntentRelationships {
    pub root_intent: RootIntentRelationshipDetails,
    pub non_root_subintents: IndexMap<SubintentHash, SubintentRelationshipDetails>,
}

#[derive(Default)]
pub struct RootIntentRelationshipDetails {
    pub children: Vec<SubintentIndex>,
}

pub struct SubintentRelationshipDetails {
    pub index: SubintentIndex,
    pub parent: IntentHash,
    pub depth: usize,
    pub children: Vec<SubintentIndex>,
}

impl SubintentRelationshipDetails {
    fn default_for(index: SubintentIndex) -> Self {
        Self {
            index,
            parent: PLACEHOLDER_PARENT,
            depth: Default::default(),
            children: Default::default(),
        }
    }
}

const PLACEHOLDER_PARENT: IntentHash =
    IntentHash::Transaction(TransactionIntentHash(Hash([0u8; Hash::LENGTH])));

// This type is public so it can be used by the toolkit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestYieldSummary {
    pub parent_yields: usize,
    pub child_yields: IndexMap<SubintentHash, usize>,
}

impl ManifestYieldSummary {
    pub fn new_with_children(children: impl Iterator<Item = SubintentHash>) -> Self {
        Self {
            parent_yields: 0,
            child_yields: children.map(|child| (child, 0)).collect(),
        }
    }
}

impl ManifestInterpretationVisitor for ManifestYieldSummary {
    type Output = ManifestValidationError;

    fn on_end_instruction(&mut self, details: OnEndInstruction) -> ControlFlow<Self::Output> {
        // Safe from overflow due to checking max instruction count
        match details.effect {
            ManifestInstructionEffect::Invocation {
                kind: InvocationKind::YieldToParent,
                ..
            } => {
                self.parent_yields += 1;
            }
            ManifestInstructionEffect::Invocation {
                kind:
                    InvocationKind::YieldToChild {
                        child_index: ManifestNamedIntent(index),
                    },
                ..
            } => {
                let index = index as usize;

                // This should exist because we are handling this after the instruction,
                // so the interpreter should have errored with ChildIntentNotRegistered
                // if the child yield was invalid.
                let (_, count) = self.child_yields.get_index_mut(index).unwrap();
                *count += 1;
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}
