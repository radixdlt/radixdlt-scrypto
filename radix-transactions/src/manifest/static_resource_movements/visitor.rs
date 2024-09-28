use super::*;
use crate::manifest::*;
use crate::prelude::*;
use core::ops::*;
use radix_common::prelude::*;

/// A [`ManifestInterpretationVisitor`] that statically tracks the resources in the worktop and
/// reports the account withdraws and deposits made.
pub struct StaticResourceMovementsVisitor {
    /// The resource content of the worktop.
    worktop: ResourceBounds,
    /// The buckets tracked by the by the visitor.
    tracked_buckets: IndexMap<ManifestBucket, (ResourceAddress, ResourceBound)>,
    /// The information about the invocations observed in this manifest. This will be surfaced to
    /// the user when they call the output function.
    invocation_static_information: IndexMap<usize, InvocationStaticInformation>,
    /// Details about the currently running transaction
    current_instruction: Option<CurrentInstruction>,
}

pub struct CurrentInstruction {
    index: usize,
    sent_resources: ResourceBounds,
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
        let mut worktop = ResourceBounds::new_empty();
        if initial_worktop_state_is_unknown {
            worktop.mut_add_unspecified_resources([ChangeSource::InitialYieldFromParent])
        }
        Self {
            worktop,
            tracked_buckets: Default::default(),
            invocation_static_information: Default::default(),
            current_instruction: None,
        }
    }

    pub fn output(self) -> StaticResourceMovementsOutput {
        StaticResourceMovementsOutput {
            invocation_static_information: self.invocation_static_information,
        }
    }

    fn handle_invocation_end(
        &mut self,
        kind: InvocationKind<'_>,
        args: &ManifestValue,
        current_instruction: CurrentInstruction,
    ) -> Result<InvocationStaticInformation, StaticResourceMovementsError> {
        // Get the invocation inputs based on the arguments.
        let invocation_input = current_instruction.sent_resources;

        let change_source = ChangeSource::Invocation {
            instruction_index: current_instruction.index,
        };

        // Creating a typed native invocation to use in interpreting the invocation.
        let typed_native_invocation = match kind {
            InvocationKind::Method {
                address: DynamicGlobalAddress::Static(global_address),
                module_id,
                method,
            } => TypedNativeInvocation::from_method_invocation(
                global_address,
                module_id,
                method,
                args,
            )
            .map(|value| (value, Some(*global_address))),
            InvocationKind::Function {
                address: DynamicPackageAddress::Static(package_address),
                blueprint,
                function,
            } => TypedNativeInvocation::from_function_invocation(
                package_address,
                blueprint,
                function,
                args,
            )
            .map(|value| (value, None)),
            // Can't convert into a typed native invocation.
            InvocationKind::DirectMethod { .. }
            | InvocationKind::YieldToParent
            | InvocationKind::YieldToChild { .. }
            | InvocationKind::Method { .. }
            | InvocationKind::Function { .. } => None,
        };

        let invocation_output = match typed_native_invocation {
            Some((matched_invocation, receiver)) => {
                matched_invocation.output(InvocationDetails {
                    receiver,
                    sent_resources: &invocation_input,
                    source: change_source,
                })?
            }
            None => ResourceBounds::new_including_unspecified_resources([change_source]),
        };

        // Add to worktop
        self.worktop.add(invocation_output.clone())?;

        // Return the invocation static information.
        Ok(InvocationStaticInformation {
            kind: kind.into(),
            input: invocation_input,
            output: invocation_output,
        })
    }

    fn current_instruction_index(&mut self) -> usize {
        self.current_instruction
            .as_ref()
            .expect("Should only be called during an instruction")
            .index
    }

    fn current_instruction_sent_resources(&mut self) -> &mut ResourceBounds {
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
            sent_resources: ResourceBounds::new_empty(),
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
                let resource_amount = self.worktop.take_resource(
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
                let resource_amount = self.worktop.take_resource(
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
                let resource_amount = self.worktop.take_resource(
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
                    .add(entire_worktop)?;
            }
            (ManifestExpression::EntireAuthZone, _) => {}
        }

        Ok(())
    }

    fn handle_worktop_assertion(
        &mut self,
        OnWorktopAssertion { assertion }: OnWorktopAssertion,
    ) -> Result<(), StaticResourceMovementsError> {
        let change_source = ChangeSource::assertion_at(self.current_instruction_index());
        self.worktop
            .handle_worktop_assertion(assertion, change_source)
    }

    fn handle_finish(&mut self, OnFinish: OnFinish) -> Result<(), StaticResourceMovementsError> {
        // We should report an error if we know for sure that the worktop is not empty
        for (_resource, bounds) in self.worktop.known_resource_bounds() {
            let (lower_bound, _upper_bound) = bounds.inclusive_bounds();
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

    fn on_new_bucket<'a>(&mut self, event: OnNewBucket<'_, 'a>) -> ControlFlow<Self::Output> {
        match self.handle_new_bucket(event) {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => ControlFlow::Break(err),
        }
    }

    fn on_consume_bucket<'a>(
        &mut self,
        event: OnConsumeBucket<'_, 'a>,
    ) -> ControlFlow<Self::Output> {
        match self.handle_consume_bucket(event) {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => ControlFlow::Break(err),
        }
    }

    fn on_pass_expression<'a>(&mut self, event: OnPassExpression<'a>) -> ControlFlow<Self::Output> {
        match self.handle_pass_expression(event) {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => ControlFlow::Break(err),
        }
    }

    fn on_worktop_assertion<'a>(
        &mut self,
        event: OnWorktopAssertion<'a>,
    ) -> ControlFlow<Self::Output> {
        match self.handle_worktop_assertion(event) {
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
