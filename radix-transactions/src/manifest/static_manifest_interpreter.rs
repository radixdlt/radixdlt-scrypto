use crate::internal_prelude::*;
use core::ops::ControlFlow;

use traversal::*;
use ManifestInstructionEffect as Effect;

pub struct StaticManifestInterpreter<'a, M: ReadableManifest> {
    validation_ruleset: ValidationRuleset,
    manifest: &'a M,
    location: ManifestLocation,
    bucket_state: Vec<BucketState<'a>>,
    proof_state: Vec<ProofState<'a>>,
    address_reservation_state: Vec<AddressReservationState<'a>>,
    named_address_state: Vec<NamedAddressState<'a>>,
    intent_state: Vec<IntentState<'a>>,
}

impl<'a, M: ReadableManifest> StaticManifestInterpreter<'a, M> {
    pub fn new(validation_ruleset: ValidationRuleset, manifest: &'a M) -> Self {
        Self {
            validation_ruleset,
            manifest,
            location: ManifestLocation::Preamble,
            bucket_state: Default::default(),
            proof_state: Default::default(),
            address_reservation_state: Default::default(),
            named_address_state: Default::default(),
            intent_state: Default::default(),
        }
    }

    pub fn interpret_or_err<V: ManifestInterpretationVisitor>(
        self,
        visitor: &mut V,
    ) -> Result<(), V::Error<'a>> {
        // It's rather weird that ControlFlow doesn't implement Into<Result> :shrug:
        match self.interpret_internal(visitor) {
            ControlFlow::Continue(()) => Ok(()),
            ControlFlow::Break(err) => Err(err),
        }
    }

    fn interpret_internal<V: ManifestInterpretationVisitor>(
        mut self,
        visitor: &mut V,
    ) -> ControlFlow<V::Error<'a>> {
        self.handle_preallocated_addresses(visitor, self.manifest.get_preallocated_addresses())?;
        self.handle_child_subintents(visitor, self.manifest.get_child_subintents())?;
        for (index, instruction) in self.manifest.get_instructions().iter().enumerate() {
            self.handle_instruction(visitor, index, instruction)?;
        }
        self.handle_wrap_up::<V>()
    }

    #[inline]
    fn handle_preallocated_addresses<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        preallocated_addresses: &[PreAllocatedAddress],
    ) -> ControlFlow<V::Error<'a>> {
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

    fn handle_child_subintents<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        child_subintents: &[ChildSubintent],
    ) -> ControlFlow<V::Error<'a>> {
        for child_subintent in child_subintents {
            self.handle_new_intent(
                visitor,
                IntentHash::Sub(child_subintent.hash),
                IntentType::Child,
            )?;
        }
        ControlFlow::Continue(())
    }

    fn handle_instruction<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        index: usize,
        instruction: &'a M::Instruction,
    ) -> ControlFlow<V::Error<'a>> {
        let effect = instruction.effect();
        self.location = ManifestLocation::Instruction { index };
        visitor.on_next_instruction(index, effect)?;

        match effect {
            Effect::CreateBucket {
                resource,
                source_amount,
            } => {
                self.handle_new_bucket(visitor, resource, source_amount)?;
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
                self.handle_cloned_proof(visitor, cloned_proof);
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
                        self.consume_proof(visitor, proof, ProofDestination::Drop);
                    }
                }
                if drop_all_authzone_signature_proofs || drop_all_authzone_non_signature_proofs {
                    visitor.on_drop_authzone_proofs(
                        drop_all_authzone_signature_proofs,
                        drop_all_authzone_non_signature_proofs,
                    )?;
                }
            }
            Effect::Invocation { kind, args } => {
                self.handle_invocation(visitor, kind, args);
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
                self.handle_new_named_address(visitor, Some(reservation));
            }
            Effect::WorktopAssertion { assertion } => {
                visitor.on_worktop_assertion(assertion)?;
            }
        }

        visitor.on_end_instruction(index, effect)
    }

    fn handle_wrap_up<V: ManifestInterpretationVisitor>(&mut self) -> ControlFlow<V::Error<'a>> {
        if self.validation_ruleset.validate_no_dangling_nodes {
            for (index, state) in self.bucket_state.iter().enumerate() {
                if state.consumed_at.is_none() {
                    return ControlFlow::Break(
                        ManifestValidationError::DanglingBucket(
                            ManifestBucket(index as u32),
                            state.clone(),
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
                            state.clone(),
                        )
                        .into(),
                    );
                }
            }
        }

        ControlFlow::Continue(())
    }

    fn handle_invocation<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        invocation_kind: InvocationKind<'a>,
        args: &'a ManifestValue,
    ) -> ControlFlow<V::Error<'a>> {
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
                                        // Check it exists before visiting a reference
                                        self.get_existing_named_address::<V>(named_address)?;
                                        visitor.on_named_address_reference(named_address)?;
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
                                self.consume_proof(
                                    visitor,
                                    proof,
                                    ProofDestination::Invocation(invocation_kind),
                                )?;
                            }
                            ManifestCustomValue::Expression(expression) => {
                                visitor.on_pass_expression(
                                    expression,
                                    ExpressionDestination::Invocation(invocation_kind),
                                )?;
                            }
                            ManifestCustomValue::Blob(blob_ref) => {
                                visitor.on_pass_blob(
                                    blob_ref,
                                    BlobDestination::Invocation(invocation_kind),
                                )?;
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

    fn handle_new_bucket<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        resource_address: &ResourceAddress,
        source_amount: BucketSourceAmount<'a>,
    ) -> ControlFlow<V::Error<'a>> {
        let new_bucket = ManifestBucket(self.bucket_state.len() as u32);
        self.bucket_state.push(BucketState {
            name: self
                .manifest
                .get_known_object_names_ref()
                .known_bucket_name(new_bucket),
            created_at: self.location,
            proof_locks: 0,
            consumed_at: None,
            source_amount,
        });
        visitor.on_new_bucket(new_bucket, resource_address, source_amount)
    }

    fn get_existing_bucket<V: ManifestInterpretationVisitor>(
        &mut self,
        bucket: ManifestBucket,
    ) -> ControlFlow<V::Error<'a>, &mut BucketState<'a>> {
        match self.bucket_state.get_mut(bucket.0 as usize) {
            Some(state) => {
                if state.consumed_at.is_some() {
                    ControlFlow::Break(
                        ManifestValidationError::BucketAlreadyUsed(bucket, state.clone()).into(),
                    )
                } else {
                    ControlFlow::Continue(state)
                }
            }
            None => ControlFlow::Break(ManifestValidationError::BucketNotYetCreated(bucket).into()),
        }
    }

    fn consume_bucket<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        bucket: ManifestBucket,
        destination: BucketDestination,
    ) -> ControlFlow<V::Error<'a>> {
        let check_proof_locks = self.validation_ruleset.validate_bucket_proof_lock;
        let location = self.location;
        let state = self.get_existing_bucket::<V>(bucket)?;
        if check_proof_locks && state.proof_locks > 0 {
            return ControlFlow::Break(
                ManifestValidationError::BucketLockedByProof(bucket, state.clone()).into(),
            );
        }
        state.consumed_at = Some(location);
        visitor.on_consume_bucket(bucket, destination)
    }

    fn handle_new_proof<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        source_amount: ProofSourceAmount<'a>,
    ) -> ControlFlow<V::Error<'a>> {
        match source_amount.proof_kind() {
            ProofKind::BucketProof(bucket) => {
                self.get_existing_bucket::<V>(bucket)?.proof_locks += 1;
                visitor.on_bucket_reference(bucket)?;
            }
            ProofKind::AuthZoneProof => {}
        }
        let new_proof = ManifestProof(self.proof_state.len() as u32);
        self.proof_state.push(ProofState {
            name: self
                .manifest
                .get_known_object_names_ref()
                .known_proof_name(new_proof),
            created_at: self.location,
            consumed_at: None,
            source_amount,
        });
        visitor.on_new_proof(new_proof, source_amount)
    }

    fn get_existing_proof<V: ManifestInterpretationVisitor>(
        &mut self,
        proof: ManifestProof,
    ) -> ControlFlow<V::Error<'a>, &mut ProofState<'a>> {
        match self.proof_state.get_mut(proof.0 as usize) {
            Some(state) => {
                if state.consumed_at.is_some() {
                    ControlFlow::Break(
                        ManifestValidationError::ProofAlreadyUsed(proof, state.clone()).into(),
                    )
                } else {
                    ControlFlow::Continue(state)
                }
            }
            None => ControlFlow::Break(ManifestValidationError::ProofNotYetCreated(proof).into()),
        }
    }

    fn handle_cloned_proof<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        cloned_proof: ManifestProof,
    ) -> ControlFlow<V::Error<'a>> {
        let source_amount = self.get_existing_proof::<V>(cloned_proof)?.source_amount;
        visitor.on_proof_reference(cloned_proof)?;
        self.handle_new_proof(visitor, source_amount)
    }

    fn consume_proof<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        proof: ManifestProof,
        destination: ProofDestination,
    ) -> ControlFlow<V::Error<'a>> {
        let location = self.location;
        let proof_state = self.get_existing_proof::<V>(proof)?;
        proof_state.consumed_at = Some(location);
        let source_amount = proof_state.source_amount;
        match source_amount.proof_kind() {
            ProofKind::BucketProof(bucket) => {
                self.get_existing_bucket::<V>(bucket)?.proof_locks -= 1;
            }
            ProofKind::AuthZoneProof => {}
        }
        visitor.on_consume_proof(proof, destination)
    }

    fn handle_new_address_reservation<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        package_address: &PackageAddress,
        blueprint_name: &str,
        preallocated_address: Option<&GlobalAddress>,
    ) -> ControlFlow<V::Error<'a>, ManifestAddressReservation> {
        let new_address_reservation =
            ManifestAddressReservation(self.address_reservation_state.len() as u32);
        self.address_reservation_state
            .push(AddressReservationState {
                name: self
                    .manifest
                    .get_known_object_names_ref()
                    .known_address_reservation_name(new_address_reservation),
                created_at: self.location,
                consumed_at: None,
            });
        visitor.on_new_address_reservation(
            new_address_reservation,
            package_address,
            blueprint_name,
            preallocated_address,
        )?;
        ControlFlow::Continue(new_address_reservation)
    }

    fn get_existing_address_reservation<V: ManifestInterpretationVisitor>(
        &mut self,
        address_reservation: ManifestAddressReservation,
    ) -> ControlFlow<V::Error<'a>, &mut AddressReservationState<'a>> {
        match self
            .address_reservation_state
            .get_mut(address_reservation.0 as usize)
        {
            Some(state) => {
                if state.consumed_at.is_some() {
                    ControlFlow::Break(
                        ManifestValidationError::AddressReservationAlreadyUsed(
                            address_reservation,
                            state.clone(),
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

    fn consume_address_reservation<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        address_reservation: ManifestAddressReservation,
        destination: AddressReservationDestination,
    ) -> ControlFlow<V::Error<'a>> {
        self.get_existing_address_reservation::<V>(address_reservation)?
            .consumed_at = Some(self.location);
        visitor.on_consume_address_reservation(address_reservation, destination)
    }

    fn handle_new_named_address<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        associated_reservation: Option<ManifestAddressReservation>,
    ) -> ControlFlow<V::Error<'a>> {
        let new_named_address = ManifestNamedAddress(self.named_address_state.len() as u32);
        self.named_address_state.push(NamedAddressState {
            name: self
                .manifest
                .get_known_object_names_ref()
                .known_address_name(new_named_address),
            created_at: self.location,
        });
        visitor.on_new_named_address(new_named_address, associated_reservation)
    }

    fn get_existing_named_address<V: ManifestInterpretationVisitor>(
        &mut self,
        named_address: ManifestNamedAddress,
    ) -> ControlFlow<V::Error<'a>, &mut NamedAddressState<'a>> {
        match self.named_address_state.get_mut(named_address.0 as usize) {
            Some(state) => ControlFlow::Continue(state),
            None => ControlFlow::Break(
                ManifestValidationError::NamedAddressNotYetCreated(named_address).into(),
            ),
        }
    }

    fn handle_new_intent<V: ManifestInterpretationVisitor>(
        &mut self,
        visitor: &mut V,
        intent_hash: IntentHash,
        intent_type: IntentType,
    ) -> ControlFlow<V::Error<'a>> {
        let new_intent = ManifestIntent(self.intent_state.len() as u32);
        self.intent_state.push(IntentState {
            name: self
                .manifest
                .get_known_object_names_ref()
                .known_intent_name(new_intent),
            created_at: self.location,
        });
        visitor.on_new_intent(new_intent, intent_hash, intent_type)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ManifestLocation {
    Preamble,
    Instruction { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BucketState<'a> {
    name: Option<&'a str>,
    source_amount: BucketSourceAmount<'a>,
    created_at: ManifestLocation,
    proof_locks: u32,
    consumed_at: Option<ManifestLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofState<'a> {
    name: Option<&'a str>,
    source_amount: ProofSourceAmount<'a>,
    created_at: ManifestLocation,
    consumed_at: Option<ManifestLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressReservationState<'a> {
    name: Option<&'a str>,
    created_at: ManifestLocation,
    consumed_at: Option<ManifestLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedAddressState<'a> {
    name: Option<&'a str>,
    created_at: ManifestLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentState<'a> {
    name: Option<&'a str>,
    created_at: ManifestLocation,
}

pub enum IntentType {
    Child,
}

// TODO can add:
// * validate_preallocated_address_against_blueprint
// * validate_total_references
// ...
pub struct ValidationRuleset {
    pub validate_bucket_proof_lock: bool,
    pub validate_no_dangling_nodes: bool,
}

impl Default for ValidationRuleset {
    fn default() -> Self {
        Self::all()
    }
}

impl ValidationRuleset {
    pub fn all() -> Self {
        Self {
            validate_bucket_proof_lock: true,
            validate_no_dangling_nodes: true,
        }
    }

    pub fn v1() -> Self {
        Self {
            validate_bucket_proof_lock: true,
            validate_no_dangling_nodes: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestValidationError<'a> {
    BucketNotYetCreated(ManifestBucket),
    BucketAlreadyUsed(ManifestBucket, BucketState<'a>),
    BucketLockedByProof(ManifestBucket, BucketState<'a>),
    ProofNotYetCreated(ManifestProof),
    ProofAlreadyUsed(ManifestProof, ProofState<'a>),
    AddressReservationNotYetCreated(ManifestAddressReservation),
    AddressReservationAlreadyUsed(ManifestAddressReservation, AddressReservationState<'a>),
    NamedAddressNotYetCreated(ManifestNamedAddress),
    DanglingBucket(ManifestBucket, BucketState<'a>),
    DanglingAddressReservation(ManifestAddressReservation, AddressReservationState<'a>),
    ArgsEncodeError(EncodeError),
    ArgsDecodeError(DecodeError),
}

// We allow unused variables so we don't have to prefix them all with `_`
#[allow(unused_variables)]
pub trait ManifestInterpretationVisitor {
    type Error<'a>: From<ManifestValidationError<'a>>;

    fn on_next_instruction<'a>(
        &mut self,
        index: usize,
        effect: ManifestInstructionEffect,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }

    fn on_end_instruction<'a>(
        &mut self,
        index: usize,
        effect: ManifestInstructionEffect,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }

    fn on_new_bucket<'a>(
        &mut self,
        bucket: ManifestBucket,
        resource_address: &ResourceAddress,
        source_amount: BucketSourceAmount,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_bucket_reference<'a>(&mut self, bucket: ManifestBucket) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_consume_bucket<'a>(
        &mut self,
        bucket: ManifestBucket,
        destination: BucketDestination,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_new_proof<'a>(
        &mut self,
        proof: ManifestProof,
        source_amount: ProofSourceAmount,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_proof_reference<'a>(&mut self, bucket: ManifestProof) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_consume_proof<'a>(
        &mut self,
        proof: ManifestProof,
        destination: ProofDestination,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_drop_authzone_proofs<'a>(
        &mut self,
        drop_signature_proofs: bool,
        drop_non_signature_proofs: bool,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_new_address_reservation<'a>(
        &mut self,
        reservation: ManifestAddressReservation,
        package_address: &PackageAddress,
        blueprint_name: &str,
        preallocated_address: Option<&GlobalAddress>,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_consume_address_reservation<'a>(
        &mut self,
        reservation: ManifestAddressReservation,
        destination: AddressReservationDestination,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_new_named_address<'a>(
        &mut self,
        address: ManifestNamedAddress,
        associated_reservation: Option<ManifestAddressReservation>,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_named_address_reference<'a>(
        &mut self,
        address: ManifestNamedAddress,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_pass_expression<'a>(
        &mut self,
        expression: ManifestExpression,
        destination: ExpressionDestination,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_pass_blob<'a>(
        &mut self,
        blob_ref: ManifestBlobRef,
        destination: BlobDestination,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_new_intent<'a>(
        &mut self,
        intent: ManifestIntent,
        intent_hash: IntentHash,
        intent_type: IntentType,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_intent_reference<'a>(&mut self, address: ManifestIntent) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
    fn on_worktop_assertion<'a>(
        &mut self,
        assertion: WorktopAssertion,
    ) -> ControlFlow<Self::Error<'a>> {
        ControlFlow::Continue(())
    }
}
