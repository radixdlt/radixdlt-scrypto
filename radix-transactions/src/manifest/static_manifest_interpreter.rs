use crate::internal_prelude::*;
use core::ops::ControlFlow;

use traversal::*;
use ManifestInstructionEffect as Effect;

/// This is a manifest interpreter which can be used to easily perform
/// more complex validations on a manifest, and supports an optional
/// visitor plugin mechanism.
///
/// This forms a next-generation validation ahead of the [`BasicManifestValidator`].
pub struct StaticManifestInterpreter<'a, M: ReadableManifest + ?Sized> {
    validation_ruleset: ValidationRuleset,
    manifest: &'a M,
    location: ManifestLocation,
    registered_blobs: IndexSet<ManifestBlobRef>,
    bucket_state: Vec<BucketState<'a>>,
    proof_state: Vec<ProofState<'a>>,
    address_reservation_state: Vec<AddressReservationState<'a>>,
    named_address_state: Vec<NamedAddressState<'a>>,
    intent_state: Vec<IntentState<'a>>,
    next_instruction_requirement: NextInstructionRequirement,
}

// --------------------------------------------
// IMPLEMENTATION NOTES - Regarding ControlFlow
// --------------------------------------------
// This manifest interpreter uses an optional visitor pattern, with the
// ControlFlow element from the Rust core library.
//
// ControlFlow is designed for a visitor use case, but as per my comment here
// (https://github.com/rust-lang/rust/issues/75744#issuecomment-2358375882)
// there are a couple of key missing functions:
// * It is missing #[must_use] - which means it's very easy to miss a ? in
//   an intermediate layer. As a workaround, we should stick #[must_use] on
//   all methods returning it.
//   (... yes, there is a war story here where I wasted more time than I care
//   to admit debugging a test :facepalm:)
// * It is missing a built-in conversion to Result
// * It is missing an automatic from conversion on Break when using the ?
//   operator. Apparently this is desired.
//
// Perhaps we should consider using Result here as it'd be easier to work with,
// even if semantically less accurate.
// --------------------------------------------

impl<'a, M: ReadableManifest + ?Sized> StaticManifestInterpreter<'a, M> {
    pub fn new(validation_ruleset: ValidationRuleset, manifest: &'a M) -> Self {
        Self {
            validation_ruleset,
            manifest,
            location: ManifestLocation::Preamble,
            registered_blobs: Default::default(),
            bucket_state: Default::default(),
            proof_state: Default::default(),
            address_reservation_state: Default::default(),
            named_address_state: Default::default(),
            intent_state: Default::default(),
            next_instruction_requirement: NextInstructionRequirement::None,
        }
    }

    pub fn validate(self) -> Result<(), ManifestValidationError> {
        self.validate_and_apply_visitor(&mut ())
    }

    pub fn validate_and_apply_visitor<V: ManifestInterpretationVisitor>(
        self,
        visitor: &mut V,
    ) -> Result<(), V::Output> {
        // For some reason ControlFlow doesn't implement Into<Result>
        match self.interpret_internal(visitor) {
            ControlFlow::Continue(()) => Ok(()),
            ControlFlow::Break(err) => Err(err),
        }
    }

    #[must_use]
    fn interpret_internal<V: ManifestInterpretationVisitor>(
        mut self,
        visitor: &mut V,
    ) -> ControlFlow<V::Output> {
        self.handle_preallocated_addresses(visitor, self.manifest.get_preallocated_addresses())?;
        self.handle_child_subintents(visitor, self.manifest.get_child_subintent_hashes())?;
        self.handle_blobs(visitor, self.manifest.get_blobs())?;
        for (index, instruction_effect) in self.manifest.iter_instruction_effects().enumerate() {
            self.handle_instruction(visitor, index, instruction_effect)?;
        }
        self.verify_final_instruction::<V>()?;
        self.handle_wrap_up(visitor)?;
        ControlFlow::Continue(())
    }

    #[must_use]
    fn handle_preallocated_addresses<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        preallocated_addresses: &'a [PreAllocatedAddress],
    ) -> ControlFlow<V::Output> {
        for preallocated_address in preallocated_addresses.iter() {
            let _ = self.handle_new_address_reservation(
                visitor,
                &preallocated_address.blueprint_id.package_address,
                preallocated_address.blueprint_id.blueprint_name.as_str(),
                Some(&preallocated_address.address),
            )?;
        }
        ControlFlow::Continue(())
    }

    #[must_use]
    fn handle_child_subintents<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        child_subintents: impl Iterator<Item = &'a ChildSubintentSpecifier>,
    ) -> ControlFlow<V::Output> {
        for child_subintent in child_subintents {
            self.handle_new_intent(
                visitor,
                IntentHash::Subintent(child_subintent.hash),
                IntentType::Child,
            )?;
        }
        ControlFlow::Continue(())
    }

    #[must_use]
    fn handle_blobs<'b, V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        blobs: impl Iterator<Item = (&'b Hash, &'b Vec<u8>)>,
    ) -> ControlFlow<V::Output> {
        for (hash, content) in blobs {
            if !self.registered_blobs.insert(ManifestBlobRef(hash.0)) {
                if self.validation_ruleset.validate_no_duplicate_blobs {
                    return ControlFlow::Break(
                        ManifestValidationError::DuplicateBlob(ManifestBlobRef(hash.0)).into(),
                    );
                }
            }
            visitor.on_register_blob(OnRegisterBlob {
                blob_ref: ManifestBlobRef(hash.0),
                content: content.as_ref(),
            })?;
        }
        ControlFlow::Continue(())
    }

    #[must_use]
    fn handle_instruction<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        index: usize,
        effect: ManifestInstructionEffect<'a>,
    ) -> ControlFlow<V::Output> {
        self.location = ManifestLocation::Instruction { index };

        match self
            .next_instruction_requirement
            .handle_next_instruction(effect)
        {
            Ok(()) => {}
            Err(error) => return ControlFlow::Break(error.into()),
        }

        visitor.on_start_instruction(OnStartInstruction { index })?;

        match effect {
            Effect::CreateBucket { source_amount } => {
                self.handle_new_bucket(visitor, source_amount)?;
            }
            Effect::CreateProof { source_amount } => {
                self.handle_new_proof(visitor, source_amount)?;
            }
            Effect::ConsumeBucket {
                consumed_bucket,
                destination,
            } => {
                self.consume_bucket(visitor, consumed_bucket, destination)?;
            }
            Effect::ConsumeProof {
                consumed_proof,
                destination,
            } => {
                self.consume_proof(visitor, consumed_proof, destination)?;
            }
            Effect::CloneProof { cloned_proof } => {
                self.handle_cloned_proof(visitor, cloned_proof)?;
            }
            Effect::DropManyProofs {
                drop_all_named_proofs,
                drop_all_authzone_signature_proofs,
                drop_all_authzone_non_signature_proofs,
            } => {
                if drop_all_named_proofs {
                    let proofs_to_drop: Vec<_> = self
                        .proof_state
                        .iter()
                        .enumerate()
                        .filter_map(|(index, p)| match p.consumed_at {
                            Some(_) => None,
                            None => Some(ManifestProof(index as u32)),
                        })
                        .collect();
                    for proof in proofs_to_drop {
                        self.consume_proof(visitor, proof, ProofDestination::Drop)?;
                    }
                }
                if drop_all_authzone_signature_proofs || drop_all_authzone_non_signature_proofs {
                    visitor.on_drop_authzone_proofs(OnDropAuthZoneProofs {
                        drop_all_signature_proofs: drop_all_authzone_signature_proofs,
                        drop_all_non_signature_proofs: drop_all_authzone_non_signature_proofs,
                    })?;
                }
            }
            Effect::Invocation { kind, args } => {
                self.handle_invocation(visitor, kind, args)?;
            }
            Effect::CreateAddressAndReservation {
                package_address,
                blueprint_name,
            } => {
                let reservation = self.handle_new_address_reservation(
                    visitor,
                    package_address,
                    blueprint_name,
                    None,
                )?;
                self.handle_new_named_address(
                    visitor,
                    Some(reservation),
                    package_address,
                    blueprint_name,
                )?;
            }
            Effect::ResourceAssertion { assertion } => {
                self.handle_resource_assertion(visitor, assertion)?;
            }
            Effect::Verification {
                verification,
                access_rule,
            } => {
                self.handle_verification(visitor, verification, access_rule)?;
            }
        }

        visitor.on_end_instruction(OnEndInstruction { index, effect })
    }

    #[must_use]
    fn verify_final_instruction<V: ManifestInterpretationVisitor>(
        &mut self,
    ) -> ControlFlow<V::Output> {
        if !self.manifest.is_subintent() {
            return ControlFlow::Continue(());
        }
        let instruction_count = self.manifest.instruction_count();
        let last_instruction_index = if instruction_count > 0 {
            instruction_count - 1
        } else {
            return ControlFlow::Break(
                ManifestValidationError::SubintentDoesNotEndWithYieldToParent.into(),
            );
        };
        match self.manifest.instruction_effect(last_instruction_index) {
            ManifestInstructionEffect::Invocation {
                kind: InvocationKind::YieldToParent,
                ..
            } => ControlFlow::Continue(()),
            _ => ControlFlow::Break(
                ManifestValidationError::SubintentDoesNotEndWithYieldToParent.into(),
            ),
        }
    }

    #[must_use]
    fn handle_wrap_up<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
    ) -> ControlFlow<V::Output> {
        match self.next_instruction_requirement.validate_at_end() {
            Ok(()) => {}
            Err(error) => return ControlFlow::Break(error.into()),
        }
        if self.validation_ruleset.validate_no_dangling_nodes {
            for (index, state) in self.bucket_state.iter().enumerate() {
                if state.consumed_at.is_none() {
                    return ControlFlow::Break(
                        ManifestValidationError::DanglingBucket(
                            ManifestBucket(index as u32),
                            format!("{state:?}"),
                        )
                        .into(),
                    );
                }
            }
            for (index, state) in self.address_reservation_state.iter().enumerate() {
                if state.consumed_at.is_none() {
                    return ControlFlow::Break(
                        ManifestValidationError::DanglingAddressReservation(
                            ManifestAddressReservation(index as u32),
                            format!("{state:?}"),
                        )
                        .into(),
                    );
                }
            }
        }
        visitor.on_finish(OnFinish)?;

        ControlFlow::Continue(())
    }

    #[must_use]
    fn handle_invocation<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        invocation_kind: InvocationKind<'a>,
        args: &'a ManifestValue,
    ) -> ControlFlow<V::Output> {
        let yields_across_intent = match invocation_kind {
            InvocationKind::Method { address, .. } => {
                if self
                    .validation_ruleset
                    .validate_dynamic_address_in_command_part
                {
                    match address {
                        ManifestGlobalAddress::Static(_) => {}
                        ManifestGlobalAddress::Named(named_address) => {
                            // Check it exists
                            self.get_existing_named_address::<V>(*named_address)?;
                        }
                    }
                }
                false
            }
            InvocationKind::Function { address, .. } => {
                if self
                    .validation_ruleset
                    .validate_dynamic_address_in_command_part
                {
                    match address {
                        ManifestPackageAddress::Static(_) => {}
                        ManifestPackageAddress::Named(named_address) => {
                            // Check it exists
                            self.get_existing_named_address::<V>(*named_address)?;
                        }
                    }
                }
                false
            }
            InvocationKind::DirectMethod { .. } => false,
            InvocationKind::YieldToParent => {
                if !self.manifest.is_subintent() {
                    return ControlFlow::Break(
                        ManifestValidationError::InstructionNotSupportedInTransactionIntent.into(),
                    );
                }
                true
            }
            InvocationKind::YieldToChild { child_index } => {
                let index = child_index.0 as usize;
                if index >= self.manifest.get_child_subintent_hashes().len() {
                    return ControlFlow::Break(
                        ManifestValidationError::ChildIntentNotRegistered(child_index).into(),
                    );
                }
                true
            }
        };
        let encoded = match manifest_encode(args) {
            Ok(encoded) => encoded,
            Err(error) => {
                return ControlFlow::Break(ManifestValidationError::ArgsEncodeError(error).into())
            }
        };
        let mut traverser = ManifestTraverser::new(
            &encoded,
            ExpectedStart::PayloadPrefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX),
            VecTraverserConfig {
                max_depth: MANIFEST_SBOR_V1_MAX_DEPTH,
                check_exact_end: true,
            },
        );
        loop {
            let event = traverser.next_event();
            match event.event {
                TraversalEvent::ContainerStart(_) => {}
                TraversalEvent::ContainerEnd(_) => {}
                TraversalEvent::TerminalValue(r) => {
                    if let traversal::TerminalValueRef::Custom(c) = r {
                        match c.0 {
                            ManifestCustomValue::Address(address) => {
                                match address {
                                    ManifestAddress::Static(_) => {}
                                    ManifestAddress::Named(named_address) => {
                                        // Check it exists
                                        self.get_existing_named_address::<V>(named_address)?;
                                    }
                                }
                            }
                            ManifestCustomValue::Bucket(bucket) => {
                                self.consume_bucket(
                                    visitor,
                                    bucket,
                                    BucketDestination::Invocation(invocation_kind),
                                )?;
                            }
                            ManifestCustomValue::Proof(proof) => {
                                if yields_across_intent {
                                    return ControlFlow::Break(
                                        ManifestValidationError::ProofCannotBePassedToAnotherIntent
                                            .into(),
                                    );
                                }
                                self.consume_proof(
                                    visitor,
                                    proof,
                                    ProofDestination::Invocation(invocation_kind),
                                )?;
                            }
                            ManifestCustomValue::Expression(expression) => {
                                visitor.on_pass_expression(OnPassExpression {
                                    expression,
                                    destination: ExpressionDestination::Invocation(invocation_kind),
                                })?;
                            }
                            ManifestCustomValue::Blob(blob_ref) => {
                                if self.validation_ruleset.validate_blob_refs {
                                    if !self.registered_blobs.contains(&blob_ref) {
                                        return ControlFlow::Break(
                                            ManifestValidationError::BlobNotRegistered(blob_ref)
                                                .into(),
                                        );
                                    }
                                }
                                visitor.on_pass_blob(OnPassBlob {
                                    blob_ref,
                                    destination: BlobDestination::Invocation(invocation_kind),
                                })?;
                            }
                            ManifestCustomValue::AddressReservation(reservation) => {
                                self.consume_address_reservation(
                                    visitor,
                                    reservation,
                                    AddressReservationDestination::Invocation(invocation_kind),
                                )?;
                            }
                            ManifestCustomValue::Decimal(_)
                            | ManifestCustomValue::NonFungibleLocalId(_)
                            | ManifestCustomValue::PreciseDecimal(_) => {}
                        }
                    }
                }
                TraversalEvent::TerminalValueBatch(_) => {}
                TraversalEvent::End => {
                    break;
                }
                TraversalEvent::DecodeError(error) => {
                    return ControlFlow::Break(
                        ManifestValidationError::ArgsDecodeError(error).into(),
                    );
                }
            }
        }
        ControlFlow::Continue(())
    }

    #[must_use]
    fn handle_resource_assertion<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        assertion: ResourceAssertion<'a>,
    ) -> ControlFlow<V::Output> {
        if self.validation_ruleset.validate_resource_assertions {
            match assertion {
                ResourceAssertion::Worktop(WorktopAssertion::ResourceNonZeroAmount { .. }) => {
                    // Nothing to validate
                }
                ResourceAssertion::Worktop(WorktopAssertion::ResourceAtLeastAmount {
                    amount,
                    ..
                }) => {
                    if amount.is_negative() {
                        return ControlFlow::Break(
                            ManifestValidationError::InvalidResourceConstraint.into(),
                        );
                    }
                }
                ResourceAssertion::Worktop(WorktopAssertion::ResourceAtLeastNonFungibles {
                    resource_address,
                    ..
                }) => {
                    if resource_address.is_fungible() {
                        return ControlFlow::Break(
                            ManifestValidationError::InvalidResourceConstraint.into(),
                        );
                    }
                }
                ResourceAssertion::Worktop(WorktopAssertion::ResourcesOnly { constraints })
                | ResourceAssertion::Worktop(WorktopAssertion::ResourcesInclude { constraints }) => {
                    if !constraints.is_valid() {
                        return ControlFlow::Break(
                            ManifestValidationError::InvalidResourceConstraint.into(),
                        );
                    }
                }
                ResourceAssertion::NextCall(NextCallAssertion::ReturnsOnly { constraints })
                | ResourceAssertion::NextCall(NextCallAssertion::ReturnsInclude { constraints }) => {
                    if !constraints.is_valid() {
                        return ControlFlow::Break(
                            ManifestValidationError::InvalidResourceConstraint.into(),
                        );
                    }
                    self.next_instruction_requirement =
                        NextInstructionRequirement::RequiredInvocationDueToNextCallAssertion;
                }
                ResourceAssertion::Bucket(BucketAssertion::Contents { bucket, constraint }) => {
                    // Check the bucket currently exists
                    let state = self.get_existing_bucket::<V>(bucket)?;
                    let resource_address = state.source_amount.resource_address();
                    if !constraint.is_valid_for(resource_address) {
                        return ControlFlow::Break(
                            ManifestValidationError::InvalidResourceConstraint.into(),
                        );
                    }
                }
            }
        }
        visitor.on_resource_assertion(OnResourceAssertion { assertion })
    }

    #[must_use]
    fn handle_verification<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        verification_kind: VerificationKind,
        access_rule: &AccessRule,
    ) -> ControlFlow<V::Output> {
        match verification_kind {
            VerificationKind::Parent => {
                if !self.manifest.is_subintent() {
                    return ControlFlow::Break(
                        ManifestValidationError::InstructionNotSupportedInTransactionIntent.into(),
                    );
                }
            }
        }
        visitor.on_verification(OnVerification {
            kind: verification_kind,
            access_rule,
        })?;
        ControlFlow::Continue(())
    }

    #[must_use]
    fn handle_new_bucket<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        source_amount: BucketSourceAmount<'a>,
    ) -> ControlFlow<V::Output> {
        let new_bucket = ManifestBucket(self.bucket_state.len() as u32);
        let state = BucketState {
            name: self
                .manifest
                .get_known_object_names_ref()
                .known_bucket_name(new_bucket),
            created_at: self.location,
            proof_locks: 0,
            consumed_at: None,
            source_amount,
        };
        visitor.on_new_bucket(OnNewBucket {
            bucket: new_bucket,
            state: &state,
        })?;
        self.bucket_state.push(state);
        ControlFlow::Continue(())
    }

    #[must_use]
    fn get_existing_bucket<V: ManifestInterpretationVisitor>(
        &mut self,
        bucket: ManifestBucket,
    ) -> ControlFlow<V::Output, &mut BucketState<'a>> {
        match self.bucket_state.get_mut(bucket.0 as usize) {
            Some(state) => {
                if state.consumed_at.is_some() {
                    ControlFlow::Break(
                        ManifestValidationError::BucketAlreadyUsed(bucket, format!("{state:?}"))
                            .into(),
                    )
                } else {
                    ControlFlow::Continue(state)
                }
            }
            None => ControlFlow::Break(ManifestValidationError::BucketNotYetCreated(bucket).into()),
        }
    }

    #[must_use]
    fn consume_bucket<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        bucket: ManifestBucket,
        destination: BucketDestination<'a>,
    ) -> ControlFlow<V::Output> {
        let check_proof_locks = self.validation_ruleset.validate_bucket_proof_lock;
        let location = self.location;
        let state = self.get_existing_bucket::<V>(bucket)?;
        if check_proof_locks && state.proof_locks > 0 {
            return ControlFlow::Break(
                ManifestValidationError::BucketConsumedWhilstLockedByProof(
                    bucket,
                    format!("{state:?}"),
                )
                .into(),
            );
        }
        state.consumed_at = Some(location);
        visitor.on_consume_bucket(OnConsumeBucket {
            bucket,
            state: &state,
            destination,
        })
    }

    #[must_use]
    fn handle_new_proof<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        source_amount: ProofSourceAmount<'a>,
    ) -> ControlFlow<V::Output> {
        match source_amount.proof_kind() {
            ProofKind::BucketProof(bucket) => {
                self.get_existing_bucket::<V>(bucket)?.proof_locks += 1;
            }
            ProofKind::AuthZoneProof => {}
        }
        let new_proof = ManifestProof(self.proof_state.len() as u32);
        let state = ProofState {
            name: self
                .manifest
                .get_known_object_names_ref()
                .known_proof_name(new_proof),
            created_at: self.location,
            consumed_at: None,
            source_amount,
        };
        visitor.on_new_proof(OnNewProof {
            proof: new_proof,
            state: &state,
        })?;
        self.proof_state.push(state);
        ControlFlow::Continue(())
    }

    #[must_use]
    fn get_existing_proof<V: ManifestInterpretationVisitor>(
        &mut self,
        proof: ManifestProof,
    ) -> ControlFlow<V::Output, &mut ProofState<'a>> {
        match self.proof_state.get_mut(proof.0 as usize) {
            Some(state) => {
                if state.consumed_at.is_some() {
                    ControlFlow::Break(
                        ManifestValidationError::ProofAlreadyUsed(proof, format!("{state:?}"))
                            .into(),
                    )
                } else {
                    ControlFlow::Continue(state)
                }
            }
            None => ControlFlow::Break(ManifestValidationError::ProofNotYetCreated(proof).into()),
        }
    }

    #[must_use]
    fn handle_cloned_proof<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        cloned_proof: ManifestProof,
    ) -> ControlFlow<V::Output> {
        let source_amount = self.get_existing_proof::<V>(cloned_proof)?.source_amount;
        self.handle_new_proof(visitor, source_amount)
    }

    #[must_use]
    fn consume_proof<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        proof: ManifestProof,
        destination: ProofDestination<'a>,
    ) -> ControlFlow<V::Output> {
        let location = self.location;
        let state = self.get_existing_proof::<V>(proof)?;
        state.consumed_at = Some(location);
        visitor.on_consume_proof(OnConsumeProof {
            proof,
            state: &state,
            destination,
        })?;
        let source_amount = state.source_amount;
        match source_amount.proof_kind() {
            ProofKind::BucketProof(bucket) => {
                self.get_existing_bucket::<V>(bucket)?.proof_locks -= 1;
            }
            ProofKind::AuthZoneProof => {}
        }
        ControlFlow::Continue(())
    }

    #[must_use]
    fn handle_new_address_reservation<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        package_address: &'a PackageAddress,
        blueprint_name: &'a str,
        preallocated_address: Option<&'a GlobalAddress>,
    ) -> ControlFlow<V::Output, ManifestAddressReservation> {
        let new_address_reservation =
            ManifestAddressReservation(self.address_reservation_state.len() as u32);
        let state = AddressReservationState {
            name: self
                .manifest
                .get_known_object_names_ref()
                .known_address_reservation_name(new_address_reservation),
            package_address,
            blueprint_name,
            preallocated_address,
            created_at: self.location,
            consumed_at: None,
        };
        visitor.on_new_address_reservation(OnNewAddressReservation {
            address_reservation: new_address_reservation,
            state: &state,
        })?;
        self.address_reservation_state.push(state);
        ControlFlow::Continue(new_address_reservation)
    }

    #[must_use]
    fn get_existing_address_reservation<V: ManifestInterpretationVisitor>(
        &mut self,
        address_reservation: ManifestAddressReservation,
    ) -> ControlFlow<V::Output, &mut AddressReservationState<'a>> {
        match self
            .address_reservation_state
            .get_mut(address_reservation.0 as usize)
        {
            Some(state) => {
                if state.consumed_at.is_some() {
                    ControlFlow::Break(
                        ManifestValidationError::AddressReservationAlreadyUsed(
                            address_reservation,
                            format!("{state:?}"),
                        )
                        .into(),
                    )
                } else {
                    ControlFlow::Continue(state)
                }
            }
            None => ControlFlow::Break(
                ManifestValidationError::AddressReservationNotYetCreated(address_reservation)
                    .into(),
            ),
        }
    }

    #[must_use]
    fn consume_address_reservation<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        address_reservation: ManifestAddressReservation,
        destination: AddressReservationDestination<'a>,
    ) -> ControlFlow<V::Output> {
        let location = self.location;
        let state = self.get_existing_address_reservation::<V>(address_reservation)?;
        state.consumed_at = Some(location);
        visitor.on_consume_address_reservation(OnConsumeAddressReservation {
            address_reservation,
            state: &state,
            destination,
        })
    }

    #[must_use]
    fn handle_new_named_address<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        associated_reservation: Option<ManifestAddressReservation>,
        package_address: &PackageAddress,
        blueprint_name: &str,
    ) -> ControlFlow<V::Output> {
        let new_named_address = ManifestNamedAddress(self.named_address_state.len() as u32);
        let state = NamedAddressState {
            name: self
                .manifest
                .get_known_object_names_ref()
                .known_address_name(new_named_address),
            associated_reservation,
            created_at: self.location,
        };
        visitor.on_new_named_address(OnNewNamedAddress {
            named_address: new_named_address,
            state: &state,
            package_address,
            blueprint_name,
        })?;
        self.named_address_state.push(state);
        ControlFlow::Continue(())
    }

    #[must_use]
    fn get_existing_named_address<V: ManifestInterpretationVisitor>(
        &mut self,
        named_address: ManifestNamedAddress,
    ) -> ControlFlow<V::Output, &mut NamedAddressState<'a>> {
        match self.named_address_state.get_mut(named_address.0 as usize) {
            Some(state) => ControlFlow::Continue(state),
            None => ControlFlow::Break(
                ManifestValidationError::NamedAddressNotYetCreated(named_address).into(),
            ),
        }
    }

    #[must_use]
    fn handle_new_intent<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        intent_hash: IntentHash,
        intent_type: IntentType,
    ) -> ControlFlow<V::Output> {
        let new_intent = ManifestNamedIntent(self.intent_state.len() as u32);
        let state = IntentState {
            name: self
                .manifest
                .get_known_object_names_ref()
                .known_intent_name(new_intent),
            intent_hash,
            intent_type,
            created_at: self.location,
        };
        visitor.on_new_intent(OnNewIntent {
            intent: new_intent,
            state: &state,
        })?;
        self.intent_state.push(state);
        ControlFlow::Continue(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ManifestLocation {
    Preamble,
    Instruction { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BucketState<'a> {
    pub name: Option<&'a str>,
    pub source_amount: BucketSourceAmount<'a>,
    pub created_at: ManifestLocation,
    pub proof_locks: u32,
    pub consumed_at: Option<ManifestLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofState<'a> {
    pub name: Option<&'a str>,
    pub source_amount: ProofSourceAmount<'a>,
    pub created_at: ManifestLocation,
    pub consumed_at: Option<ManifestLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressReservationState<'a> {
    pub name: Option<&'a str>,
    pub package_address: &'a PackageAddress,
    pub blueprint_name: &'a str,
    pub preallocated_address: Option<&'a GlobalAddress>,
    pub created_at: ManifestLocation,
    pub consumed_at: Option<ManifestLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedAddressState<'a> {
    pub name: Option<&'a str>,
    pub associated_reservation: Option<ManifestAddressReservation>,
    pub created_at: ManifestLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentState<'a> {
    pub name: Option<&'a str>,
    pub intent_hash: IntentHash,
    pub intent_type: IntentType,
    pub created_at: ManifestLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentType {
    Child,
}

enum NextInstructionRequirement {
    None,
    RequiredInvocationDueToNextCallAssertion,
}

impl NextInstructionRequirement {
    fn handle_next_instruction(
        &mut self,
        effect: ManifestInstructionEffect,
    ) -> Result<(), ManifestValidationError> {
        match self {
            NextInstructionRequirement::None => Ok(()),
            NextInstructionRequirement::RequiredInvocationDueToNextCallAssertion => {
                if matches!(effect, ManifestInstructionEffect::Invocation { .. }) {
                    *self = NextInstructionRequirement::None;
                    Ok(())
                } else {
                    Err(ManifestValidationError::InstructionFollowingNextCallAssertionWasNotInvocation)
                }
            }
        }
    }

    fn validate_at_end(&self) -> Result<(), ManifestValidationError> {
        match self {
            NextInstructionRequirement::None => Ok(()),
            NextInstructionRequirement::RequiredInvocationDueToNextCallAssertion => {
                Err(ManifestValidationError::ManifestEndedWhilstExpectingNextCallAssertion)
            }
        }
    }
}

// TODO can add:
// * validate_preallocated_address_against_blueprint
// ...
// Possibly we should consider making this a generic to make it more performant.
pub struct ValidationRuleset {
    pub validate_no_duplicate_blobs: bool,
    pub validate_blob_refs: bool,
    pub validate_bucket_proof_lock: bool,
    pub validate_no_dangling_nodes: bool,
    pub validate_dynamic_address_in_command_part: bool,
    pub validate_resource_assertions: bool,
}

impl Default for ValidationRuleset {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub enum InterpreterValidationRulesetSpecifier {
    AllValidations,
    Cuttlefish,
}

impl ValidationRuleset {
    pub fn for_specifier(specifier: InterpreterValidationRulesetSpecifier) -> Self {
        match specifier {
            InterpreterValidationRulesetSpecifier::AllValidations => Self::all(),
            InterpreterValidationRulesetSpecifier::Cuttlefish => Self::cuttlefish(),
        }
    }

    pub fn all() -> Self {
        Self {
            validate_no_duplicate_blobs: true,
            validate_blob_refs: true,
            validate_bucket_proof_lock: true,
            validate_no_dangling_nodes: true,
            validate_dynamic_address_in_command_part: true,
            validate_resource_assertions: true,
        }
    }

    pub fn babylon_equivalent() -> Self {
        Self {
            validate_no_duplicate_blobs: false,
            validate_blob_refs: false,
            validate_bucket_proof_lock: true,
            validate_no_dangling_nodes: false,
            validate_dynamic_address_in_command_part: false,
            validate_resource_assertions: false,
        }
    }

    pub fn cuttlefish() -> Self {
        Self {
            validate_no_duplicate_blobs: true,
            validate_blob_refs: true,
            validate_bucket_proof_lock: true,
            validate_no_dangling_nodes: true,
            validate_dynamic_address_in_command_part: true,
            validate_resource_assertions: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestValidationError {
    DuplicateBlob(ManifestBlobRef),
    BlobNotRegistered(ManifestBlobRef),
    BucketNotYetCreated(ManifestBucket),
    BucketAlreadyUsed(ManifestBucket, String),
    BucketConsumedWhilstLockedByProof(ManifestBucket, String),
    ProofNotYetCreated(ManifestProof),
    ProofAlreadyUsed(ManifestProof, String),
    AddressReservationNotYetCreated(ManifestAddressReservation),
    AddressReservationAlreadyUsed(ManifestAddressReservation, String),
    NamedAddressNotYetCreated(ManifestNamedAddress),
    ChildIntentNotRegistered(ManifestNamedIntent),
    DanglingBucket(ManifestBucket, String),
    DanglingAddressReservation(ManifestAddressReservation, String),
    ArgsEncodeError(EncodeError),
    ArgsDecodeError(DecodeError),
    InstructionNotSupportedInTransactionIntent,
    SubintentDoesNotEndWithYieldToParent,
    ProofCannotBePassedToAnotherIntent,
    TooManyInstructions,
    InvalidResourceConstraint,
    InstructionFollowingNextCallAssertionWasNotInvocation,
    ManifestEndedWhilstExpectingNextCallAssertion,
}

// We allow unused variables so we don't have to prefix them all with `_`
#[allow(unused_variables)]
pub trait ManifestInterpretationVisitor {
    type Output: From<ManifestValidationError>;

    #[must_use]
    fn on_start_instruction(&mut self, details: OnStartInstruction) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_end_instruction(&mut self, details: OnEndInstruction) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_new_bucket(&mut self, details: OnNewBucket) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_consume_bucket(&mut self, details: OnConsumeBucket) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_new_proof(&mut self, details: OnNewProof) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_consume_proof(&mut self, details: OnConsumeProof) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_new_address_reservation(
        &mut self,
        details: OnNewAddressReservation,
    ) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_consume_address_reservation(
        &mut self,
        details: OnConsumeAddressReservation,
    ) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_new_named_address(&mut self, details: OnNewNamedAddress) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_new_intent(&mut self, details: OnNewIntent) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_drop_authzone_proofs(
        &mut self,
        details: OnDropAuthZoneProofs,
    ) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_pass_expression(&mut self, details: OnPassExpression) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_register_blob(&mut self, details: OnRegisterBlob) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_pass_blob(&mut self, details: OnPassBlob) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_resource_assertion(&mut self, details: OnResourceAssertion) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_verification(&mut self, details: OnVerification) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn on_finish(&mut self, details: OnFinish) -> ControlFlow<Self::Output> {
        ControlFlow::Continue(())
    }
}

impl ManifestInterpretationVisitor for () {
    type Output = ManifestValidationError;
}

/// The instruction has not yet passed validity checks, therefore is not included in the event.
pub struct OnStartInstruction {
    pub index: usize,
}

/// The instruction and its effects have passed validity checks.
pub struct OnEndInstruction<'a> {
    pub index: usize,
    pub effect: ManifestInstructionEffect<'a>,
}

/// A valid bucket has been created.
pub struct OnNewBucket<'s, 'a> {
    pub bucket: ManifestBucket,
    pub state: &'s BucketState<'a>,
}

/// A bucket has been consumed in a valid manner.
pub struct OnConsumeBucket<'s, 'a> {
    pub bucket: ManifestBucket,
    pub state: &'s BucketState<'a>,
    pub destination: BucketDestination<'a>,
}

/// A valid proof has been created.
pub struct OnNewProof<'s, 'a> {
    pub proof: ManifestProof,
    pub state: &'s ProofState<'a>,
}

/// A proof has been consumed in a valid manner.
pub struct OnConsumeProof<'s, 'a> {
    pub proof: ManifestProof,
    pub state: &'s ProofState<'a>,
    pub destination: ProofDestination<'a>,
}

/// A valid address reservation has been created.
pub struct OnNewAddressReservation<'s, 'a> {
    pub address_reservation: ManifestAddressReservation,
    pub state: &'s AddressReservationState<'a>,
}

/// An address reservation has been consumed in a valid manner.
pub struct OnConsumeAddressReservation<'s, 'a> {
    pub address_reservation: ManifestAddressReservation,
    pub state: &'s AddressReservationState<'a>,
    pub destination: AddressReservationDestination<'a>,
}

/// A valid named address has been created.
pub struct OnNewNamedAddress<'s, 'a> {
    pub named_address: ManifestNamedAddress,
    pub state: &'s NamedAddressState<'a>,
    pub package_address: &'a PackageAddress,
    pub blueprint_name: &'a str,
}

/// A valid named intent has been created.
pub struct OnNewIntent<'s, 'a> {
    pub intent: ManifestNamedIntent,
    pub state: &'s IntentState<'a>,
}

/// A valid auth zone bulk-drop instruction has been processed.
pub struct OnDropAuthZoneProofs {
    pub drop_all_signature_proofs: bool,
    pub drop_all_non_signature_proofs: bool,
}

/// An expression has been passed into an invocation.
pub struct OnPassExpression<'a> {
    pub expression: ManifestExpression,
    pub destination: ExpressionDestination<'a>,
}

/// A blob has been registered
pub struct OnRegisterBlob<'a> {
    pub blob_ref: ManifestBlobRef,
    pub content: &'a [u8],
}

/// A blob has been passed into an invocation.
pub struct OnPassBlob<'a> {
    pub blob_ref: ManifestBlobRef,
    pub destination: BlobDestination<'a>,
}

/// A valid assertion has been processed.
pub struct OnResourceAssertion<'a> {
    pub assertion: ResourceAssertion<'a>,
}

/// A valid verification has been processed.
pub struct OnVerification<'a> {
    pub kind: VerificationKind,
    pub access_rule: &'a AccessRule,
}

/// The manifest has finished processing, and has passed validity checks.
pub struct OnFinish;
