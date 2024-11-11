use super::*;
use crate::internal_prelude::*;
use crate::manifest::*;
use core::ops::*;
use radix_engine_interface::prelude::*;

/// A [`ManifestInterpretationVisitor`] that statically tracks the resources in the worktop and
/// reports the account withdraws and deposits made.
pub struct StaticResourceMovementsVisitor {
    /// The resource content of the worktop.
    worktop: TrackedResources,
    /// Bounds against all existing buckets tracked by the visitor.
    tracked_buckets: IndexMap<ManifestBucket, (ResourceAddress, TrackedResource)>,
    /// The blueprint of all running named addresses
    tracked_named_addresses: IndexMap<ManifestNamedAddress, BlueprintId>,
    /// The information about the invocations observed in this manifest. This will be surfaced to
    /// the user when they call the output function.
    invocation_static_information: IndexMap<usize, InvocationStaticInformation>,
    /// Details about the currently running instruction. Has a value between OnStartInstruction and OnEndInstruction.
    current_instruction: Option<CurrentInstruction>,
    /// Details the assertion to apply to the contents returned from the next invocation.
    next_invocation_assertion: Option<(OwnedNextCallAssertion, ChangeSource)>,
}

pub struct CurrentInstruction {
    index: usize,
    sent_resources: TrackedResources,
}

/// Created by the visitor, generally references a particular instruction, or maybe an initial YIELD_TO_PARENT.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChangeSource {
    InitialYieldFromParent,
    Invocation { instruction_index: usize },
    NewBucket { instruction_index: usize },
    Assertion { instruction_index: usize },
}

impl ChangeSource {
    pub fn invocation_at(instruction_index: usize) -> Self {
        Self::Invocation { instruction_index }
    }

    pub fn bucket_at(instruction_index: usize) -> Self {
        Self::NewBucket { instruction_index }
    }

    pub fn assertion_at(instruction_index: usize) -> Self {
        Self::Assertion { instruction_index }
    }
}

impl StaticResourceMovementsVisitor {
    pub fn new(initial_worktop_state_is_unknown: bool) -> Self {
        let mut worktop = TrackedResources::new_empty();

        if initial_worktop_state_is_unknown {
            worktop.mut_add_unspecified_resources([ChangeSource::InitialYieldFromParent])
        }

        Self {
            worktop,
            tracked_buckets: Default::default(),
            tracked_named_addresses: Default::default(),
            invocation_static_information: Default::default(),
            current_instruction: None,
            next_invocation_assertion: None,
        }
    }

    pub fn output(self) -> StaticResourceMovementsOutput {
        StaticResourceMovementsOutput {
            invocation_static_information: self.invocation_static_information,
        }
    }

    fn handle_invocation_end(
        &mut self,
        invocation_kind: InvocationKind<'_>,
        args: &ManifestValue,
        current_instruction: CurrentInstruction,
    ) -> Result<InvocationStaticInformation, StaticResourceMovementsError> {
        // Get the invocation inputs from the aggregated resources sent during the current instruction.
        let invocation_input = current_instruction.sent_resources;

        let change_source = ChangeSource::Invocation {
            instruction_index: current_instruction.index,
        };

        // TODO: In the future we should propagate errors from the native instruction conversion.
        // We do not do it at the moment as we have found issues when decoding valid invocations as
        // their manifest SBOR types.
        let mut invocation_output = match self.resolve_native_invocation(invocation_kind, args) {
            Ok(Some((matched_invocation, receiver))) => {
                matched_invocation.output(InvocationDetails {
                    receiver,
                    sent_resources: &invocation_input,
                    source: change_source,
                })?
            }
            Err(..) | Ok(None) => {
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    change_source,
                ])
            }
        };

        if let Some((assertion, change_source)) = self.next_invocation_assertion.take() {
            // FUTURE TWEAK: Could output an inequality constraints when handling the assertions,
            // for use in any analyzers.
            match assertion.as_ref() {
                NextCallAssertion::ReturnsOnly { constraints } => {
                    invocation_output
                        .handle_resources_only_assertion(constraints, change_source)?;
                }
                NextCallAssertion::ReturnsInclude { constraints } => {
                    invocation_output
                        .handle_resources_include_assertion(constraints, change_source)?;
                }
            }
        }

        // Add the returned resources to the worktop
        self.worktop.mut_add(invocation_output.clone())?;

        Ok(InvocationStaticInformation {
            kind: invocation_kind.into(),
            input: invocation_input,
            output: invocation_output,
        })
    }

    fn resolve_native_invocation(
        &self,
        invocation_kind: InvocationKind,
        args: &ManifestValue,
    ) -> Result<
        Option<(TypedManifestNativeInvocation, InvocationReceiver)>,
        StaticResourceMovementsError,
    > {
        // Creating a typed native invocation to use in interpreting the invocation.
        match invocation_kind {
            InvocationKind::DirectMethod { address, method } => {
                let resolved_dynamic_address = ResolvedDynamicAddress::StaticAddress(*address);
                let Some(typed_invocation) =
                    TypedManifestNativeInvocation::from_direct_method_invocation(
                        &resolved_dynamic_address,
                        method,
                        args,
                    )?
                else {
                    return Ok(None);
                };
                let invocation_receiver = InvocationReceiver::DirectAccess(*address);
                Ok(Some((typed_invocation, invocation_receiver)))
            }
            InvocationKind::Method {
                address,
                module_id,
                method,
            } => {
                let resolved_dynamic_address = match address {
                    ManifestGlobalAddress::Static(global_address) => {
                        ResolvedDynamicAddress::StaticAddress(*global_address)
                    }
                    ManifestGlobalAddress::Named(named_address) => {
                        let blueprint_id = self.tracked_named_addresses.get(named_address)
                            .expect("Interpreter should have validated the address exists, because we're handling this on instruction end");
                        ResolvedDynamicAddress::BlueprintResolvedFromNamedAddress(
                            blueprint_id.clone(),
                        )
                    }
                };
                let Some(typed_invocation) = TypedManifestNativeInvocation::from_method_invocation(
                    &resolved_dynamic_address,
                    module_id,
                    method,
                    args,
                )?
                else {
                    return Ok(None);
                };
                let invocation_receiver =
                    InvocationReceiver::GlobalMethod(resolved_dynamic_address);
                Ok(Some((typed_invocation, invocation_receiver)))
            }
            InvocationKind::Function {
                address: ManifestPackageAddress::Static(package_address),
                blueprint,
                function,
            } => {
                let blueprint_id = BlueprintId::new(package_address, blueprint);
                let Some(typed_invocation) =
                    TypedManifestNativeInvocation::from_function_invocation(
                        &blueprint_id,
                        function,
                        args,
                    )?
                else {
                    return Ok(None);
                };
                let invocation_receiver = InvocationReceiver::BlueprintFunction(blueprint_id);
                Ok(Some((typed_invocation, invocation_receiver)))
            }
            InvocationKind::YieldToParent
            | InvocationKind::YieldToChild { .. }
            | InvocationKind::Function {
                address: ManifestPackageAddress::Named(_),
                ..
            } => Ok(None),
        }
    }

    fn current_instruction_index(&mut self) -> usize {
        self.current_instruction
            .as_ref()
            .expect("Should only be called during an instruction")
            .index
    }

    fn current_instruction_sent_resources(&mut self) -> &mut TrackedResources {
        &mut self
            .current_instruction
            .as_mut()
            .expect("Should only be called during an instruction")
            .sent_resources
    }

    fn handle_start_instruction(
        &mut self,
        OnStartInstruction { index, .. }: OnStartInstruction,
    ) -> Result<(), StaticResourceMovementsError> {
        self.current_instruction = Some(CurrentInstruction {
            index,
            sent_resources: TrackedResources::new_empty(),
        });
        Ok(())
    }

    fn handle_end_instruction(
        &mut self,
        OnEndInstruction { effect, index }: OnEndInstruction,
    ) -> Result<(), StaticResourceMovementsError> {
        let instruction_data = self.current_instruction.take().unwrap();

        // We only care about invocations. Ignore anything that is not an invocation.
        let ManifestInstructionEffect::Invocation { kind, args, .. } = effect else {
            return Ok(());
        };

        // Handle the invocation and get its static information back.
        let invocation_static_information =
            self.handle_invocation_end(kind, args, instruction_data)?;

        // Adding the static information to the state to surface later to the consumer.
        self.invocation_static_information
            .insert(index, invocation_static_information);

        Ok(())
    }

    fn handle_new_bucket(
        &mut self,
        OnNewBucket { bucket, state }: OnNewBucket,
    ) -> Result<(), StaticResourceMovementsError> {
        let source = ChangeSource::NewBucket {
            instruction_index: self.current_instruction_index(),
        };
        let (resource_address, resource_amount) = match state.source_amount {
            BucketSourceAmount::AllOnWorktop { resource_address } => {
                let resource_amount = self.worktop.mut_take_resource(
                    *resource_address,
                    ResourceTakeAmount::All,
                    source,
                )?;
                (*resource_address, resource_amount)
            }
            BucketSourceAmount::AmountFromWorktop {
                resource_address,
                amount,
            } => {
                let resource_amount = self.worktop.mut_take_resource(
                    *resource_address,
                    ResourceTakeAmount::exact_amount(amount)?,
                    source,
                )?;
                (*resource_address, resource_amount)
            }
            BucketSourceAmount::NonFungiblesFromWorktop {
                resource_address,
                ids,
            } => {
                let resource_amount = self.worktop.mut_take_resource(
                    *resource_address,
                    ResourceTakeAmount::exact_non_fungibles(ids.iter().cloned()),
                    source,
                )?;
                (*resource_address, resource_amount)
            }
        };

        self.tracked_buckets
            .insert(bucket, (resource_address, resource_amount));

        Ok(())
    }

    fn handle_consume_bucket(
        &mut self,
        OnConsumeBucket {
            bucket,
            destination,
            ..
        }: OnConsumeBucket,
    ) -> Result<(), StaticResourceMovementsError> {
        let (resource_address, amount) = self
            .tracked_buckets
            .swap_remove(&bucket)
            .expect("Interpreter should ensure the bucket lifetimes are validated");

        match destination {
            BucketDestination::Worktop => {
                self.worktop.mut_add_resource(resource_address, amount)?;
            }
            BucketDestination::Burned => {}
            BucketDestination::Invocation(_) => self
                .current_instruction_sent_resources()
                .mut_add_resource(resource_address, amount)?,
        }

        Ok(())
    }

    fn handle_pass_expression(
        &mut self,
        OnPassExpression {
            expression,
            destination,
            ..
        }: OnPassExpression,
    ) -> Result<(), StaticResourceMovementsError> {
        match (expression, destination) {
            (ManifestExpression::EntireWorktop, ExpressionDestination::Invocation(_)) => {
                let entire_worktop = self.worktop.take_all();
                self.current_instruction_sent_resources()
                    .mut_add(entire_worktop)?;
            }
            (ManifestExpression::EntireAuthZone, _) => {}
        }

        Ok(())
    }

    fn handle_resource_assertion(
        &mut self,
        OnResourceAssertion { assertion }: OnResourceAssertion,
    ) -> Result<(), StaticResourceMovementsError> {
        // FUTURE TWEAK: Could add inequality constraints when handling assertions,
        // for use in any analyzers.
        let change_source = ChangeSource::assertion_at(self.current_instruction_index());
        match assertion {
            ResourceAssertion::Worktop(WorktopAssertion::ResourceNonZeroAmount {
                resource_address,
            }) => self.worktop.handle_resource_assertion(
                *resource_address,
                ResourceBounds::non_zero(),
                change_source,
            ),
            ResourceAssertion::Worktop(WorktopAssertion::ResourceAtLeastAmount {
                resource_address,
                amount,
            }) => self.worktop.handle_resource_assertion(
                *resource_address,
                ResourceBounds::at_least_amount(amount)?,
                change_source,
            ),
            ResourceAssertion::Worktop(WorktopAssertion::ResourceAtLeastNonFungibles {
                resource_address,
                ids,
            }) => self.worktop.handle_resource_assertion(
                *resource_address,
                ResourceBounds::at_least_non_fungibles(ids.iter().cloned()),
                change_source,
            ),
            ResourceAssertion::Worktop(WorktopAssertion::ResourcesOnly { constraints }) => self
                .worktop
                .handle_resources_only_assertion(constraints, change_source),
            ResourceAssertion::Worktop(WorktopAssertion::ResourcesInclude { constraints }) => self
                .worktop
                .handle_resources_include_assertion(constraints, change_source),
            ResourceAssertion::NextCall(next_call_assertion) => {
                if self.next_invocation_assertion.is_some() {
                    panic!("Interpreter should have verified that a next call assertion must be used before another is created");
                }
                self.next_invocation_assertion = Some((next_call_assertion.into(), change_source));
                Ok(())
            }
            ResourceAssertion::Bucket(BucketAssertion::Contents { bucket, constraint }) => {
                let (_, tracked_resource) = self
                    .tracked_buckets
                    .get_mut(&bucket)
                    .expect("Interpreter should have already validated that the bucket exists");
                tracked_resource.handle_assertion(
                    ResourceBounds::new_for_manifest_constraint(constraint)?,
                    change_source,
                )
            }
        }
    }

    fn handle_new_named_address(
        &mut self,
        OnNewNamedAddress {
            named_address,
            package_address,
            blueprint_name,
            ..
        }: OnNewNamedAddress,
    ) -> Result<(), StaticResourceMovementsError> {
        self.tracked_named_addresses.insert(
            named_address,
            BlueprintId::new(package_address, blueprint_name),
        );
        Ok(())
    }

    fn handle_finish(&mut self, OnFinish: OnFinish) -> Result<(), StaticResourceMovementsError> {
        // We should report an error if we know for sure that the worktop is not empty
        for (_resource, resource_bound) in self.worktop.specified_resources() {
            let (lower_bound, _upper_bound) = resource_bound.bounds().numeric_bounds();
            if lower_bound.is_positive() {
                return Err(StaticResourceMovementsError::WorktopEndsWithKnownResourcesPresent);
            }
        }
        Ok(())
    }
}

impl ManifestInterpretationVisitor for StaticResourceMovementsVisitor {
    type Output = StaticResourceMovementsError;

    fn on_start_instruction(&mut self, event: OnStartInstruction) -> ControlFlow<Self::Output> {
        match self.handle_start_instruction(event) {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => ControlFlow::Break(err),
        }
    }

    fn on_end_instruction(&mut self, event: OnEndInstruction) -> ControlFlow<Self::Output> {
        match self.handle_end_instruction(event) {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => ControlFlow::Break(err),
        }
    }

    fn on_new_bucket(&mut self, event: OnNewBucket) -> ControlFlow<Self::Output> {
        match self.handle_new_bucket(event) {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => ControlFlow::Break(err),
        }
    }

    fn on_consume_bucket(&mut self, event: OnConsumeBucket) -> ControlFlow<Self::Output> {
        match self.handle_consume_bucket(event) {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => ControlFlow::Break(err),
        }
    }

    fn on_pass_expression(&mut self, event: OnPassExpression) -> ControlFlow<Self::Output> {
        match self.handle_pass_expression(event) {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => ControlFlow::Break(err),
        }
    }

    fn on_resource_assertion(&mut self, event: OnResourceAssertion) -> ControlFlow<Self::Output> {
        match self.handle_resource_assertion(event) {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => ControlFlow::Break(err),
        }
    }

    fn on_new_named_address(&mut self, event: OnNewNamedAddress) -> ControlFlow<Self::Output> {
        match self.handle_new_named_address(event) {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => ControlFlow::Break(err),
        }
    }

    fn on_finish(&mut self, event: OnFinish) -> ControlFlow<Self::Output> {
        match self.handle_finish(event) {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => ControlFlow::Break(err),
        }
    }
}
