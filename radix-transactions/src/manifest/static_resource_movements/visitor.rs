use super::*;
use crate::manifest::*;
use crate::prelude::*;
use radix_common::prelude::*;
use std::ops::*;
use traversal::*;

/// A [`ManifestInterpretationVisitor`] that statically tracks the resources in the worktop and
/// reports the account withdraws and deposits made.
pub struct StaticResourceMovementsVisitor {
    /// The resource content of the worktop.
    worktop_fungible_contents: IndexMap<FungibleResourceAddress, FungibleBounds>,
    /// The resource content of the worktop.
    worktop_non_fungible_contents: IndexMap<NonFungibleResourceAddress, NonFungibleBounds>,
    /// The sources of uncertainty about the worktop.
    worktop_uncertainty_sources: Vec<WorktopUncertaintySource>,
    /// The buckets tracked by the by the visitor.
    tracked_buckets: IndexMap<ManifestBucket, BucketContent>,
    /// The information about the invocations observed in this manifest. This will be surfaced to
    /// the user when they call the output function.
    invocation_static_information: IndexMap<usize, InvocationStaticInformation>,
}

impl StaticResourceMovementsVisitor {
    pub fn new(initial_worktop_state_is_unknown: bool) -> Self {
        let worktop_uncertainty_sources = if initial_worktop_state_is_unknown {
            vec![WorktopUncertaintySource::YieldFromParent]
        } else {
            vec![]
        };
        Self {
            tracked_buckets: Default::default(),
            worktop_fungible_contents: Default::default(),
            worktop_non_fungible_contents: Default::default(),
            worktop_uncertainty_sources,
            invocation_static_information: Default::default(),
        }
    }

    pub fn output(self) -> StaticResourceMovementsOutput {
        StaticResourceMovementsOutput {
            invocation_static_information: self.invocation_static_information,
        }
    }

    fn handle_invocation(
        &mut self,
        kind: InvocationKind<'_>,
        args: &ManifestValue,
        index: usize,
    ) -> ControlFlow<StaticResourceMovementsError<'static>, InvocationStaticInformation> {
        // Creating a new invocation static information which will be returned back to the caller at
        // the end of this handling.
        let mut invocation_information = InvocationStaticInformation {
            kind: kind.into(),
            input: Default::default(),
            output: Default::default(),
        };

        // Get the invocation inputs based on the arguments.
        invocation_information.input = self.resolve_args_into_invocation_io(args)?;

        // Creating a typed native invocation to use in interpreting the invocation.
        let typed_native_invocation = match kind {
            InvocationKind::Method {
                address: DynamicGlobalAddress::Static(global_address),
                module_id,
                method,
            } => TypedNativeInvocation::from_method_invocation(
                global_address.as_node_id(),
                module_id,
                method,
                args,
            )
            .map(|value| (value, global_address.into_node_id())),
            InvocationKind::Function {
                address: DynamicPackageAddress::Static(package_address),
                blueprint,
                function,
            } => TypedNativeInvocation::from_function_invocation(
                package_address.as_node_id(),
                blueprint,
                function,
                args,
            )
            .map(|value| (value, package_address.into_node_id())),
            // Can't convert into a typed native invocation.
            InvocationKind::DirectMethod { .. }
            | InvocationKind::YieldToParent
            | InvocationKind::YieldToChild { .. }
            | InvocationKind::Method { .. }
            | InvocationKind::Function { .. } => None,
        };

        // Getting the invocation outputs based on the typed invocation. If we could not create a
        // typed invocation from the invocation then the invocation output will be an uncertainty
        // source.
        invocation_information.output = typed_native_invocation
            .map(|(invocation, node_id)| {
                invocation.output(&node_id, &invocation_information.input, index)
            })
            .unwrap_or(vec![InvocationIo::Unknown(
                WorktopUncertaintySource::Invocation {
                    instruction_index: index,
                },
            )]);

        // Handle the invocation outputs - add them to the worktop as needed.
        for invocation_io in invocation_information.output.iter() {
            match invocation_io {
                InvocationIo::KnownFungible(fungible_resource_address, fungible_bounds) => {
                    match self
                        .worktop_fungible_contents
                        .get_mut(fungible_resource_address)
                    {
                        Some(worktop_content) => match worktop_content.combine(*fungible_bounds) {
                            Some(value) => ControlFlow::Continue(value),
                            None => {
                                ControlFlow::Break(StaticResourceMovementsError::DecimalOverflow)
                            }
                        }?,
                        None => {
                            self.worktop_fungible_contents
                                .insert(*fungible_resource_address, *fungible_bounds);
                        }
                    }
                }
                InvocationIo::KnownNonFungible(
                    non_fungible_resource_address,
                    non_fungible_bounds,
                ) => {
                    match self
                        .worktop_non_fungible_contents
                        .get_mut(non_fungible_resource_address)
                    {
                        Some(worktop_content) => {
                            match worktop_content.combine(non_fungible_bounds.clone()) {
                                Some(value) => ControlFlow::Continue(value),
                                None => ControlFlow::Break(
                                    StaticResourceMovementsError::DecimalOverflow,
                                ),
                            }?
                        }
                        None => {
                            self.worktop_non_fungible_contents.insert(
                                *non_fungible_resource_address,
                                non_fungible_bounds.clone(),
                            );
                        }
                    }
                }
                InvocationIo::Unknown(worktop_uncertainty_source) => self
                    .worktop_uncertainty_sources
                    .push(*worktop_uncertainty_source),
            }
        }

        // Return the invocation static information.
        ControlFlow::Continue(invocation_information)
    }

    fn resolve_args_into_invocation_io(
        &mut self,
        args: &ManifestValue,
    ) -> ControlFlow<StaticResourceMovementsError<'static>, Vec<InvocationIo>> {
        // Getting all of the buckets and expressions in the arguments.
        let (buckets, expressions) = {
            let encoded = match manifest_encode(args) {
                Ok(encoded) => encoded,
                Err(error) => {
                    return ControlFlow::Break(
                        ManifestValidationError::ArgsEncodeError(error).into(),
                    )
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
            let mut buckets = index_set_new();
            let mut expressions = index_set_new();
            loop {
                let event = traverser.next_event();
                match event.event {
                    TraversalEvent::TerminalValue(value) => match value {
                        TerminalValueRef::Custom(ManifestCustomTerminalValueRef(
                            ManifestCustomValue::Bucket(bucket),
                        )) => {
                            buckets.insert(bucket);
                        }
                        TerminalValueRef::Custom(ManifestCustomTerminalValueRef(
                            ManifestCustomValue::Expression(expression),
                        )) => {
                            expressions.insert(expression);
                        }
                        _ => {}
                    },
                    TraversalEvent::ContainerStart(_)
                    | TraversalEvent::ContainerEnd(_)
                    | TraversalEvent::TerminalValueBatch(_) => {}
                    TraversalEvent::DecodeError(error) => {
                        return ControlFlow::Break(
                            ManifestValidationError::ArgsDecodeError(error).into(),
                        );
                    }
                    TraversalEvent::End => break,
                }
            }
            (buckets, expressions)
        };

        // Resolving the buckets and the expressions into the bucket contents. We do not consume the
        // buckets here, we just get a clone of the contents of the bucket. This is because there is
        // a specific callback for consuming buckets.
        let mut inputs = result_to_control_flow(
            buckets
                .into_iter()
                .map(|bucket| {
                    self.tracked_buckets
                        .get(&bucket)
                        .cloned()
                        .map(InvocationIo::from)
                        .ok_or(StaticResourceMovementsError::BucketDoesntExist(bucket))
                })
                .collect::<Result<Vec<_>, _>>(),
        )?;
        if expressions
            .into_iter()
            .any(|expression| expression == ManifestExpression::EntireWorktop)
        {
            inputs.extend(
                self.worktop_fungible_contents
                    .drain(..)
                    .map(InvocationIo::from)
                    .chain(
                        self.worktop_non_fungible_contents
                            .drain(..)
                            .map(InvocationIo::from),
                    )
                    .chain(
                        self.worktop_uncertainty_sources
                            .drain(..)
                            .map(InvocationIo::Unknown),
                    ),
            );
        };

        ControlFlow::Continue(inputs)
    }
}

impl ManifestInterpretationVisitor for StaticResourceMovementsVisitor {
    type Error<'a> = StaticResourceMovementsError<'a>;

    // region:Invocation
    fn on_start_instruction<'a>(
        &mut self,
        OnStartInstruction { index, effect }: OnStartInstruction<'a>,
    ) -> ControlFlow<Self::Error<'a>> {
        // We only care about invocations. Ignore anything that is not an invocation.
        let ManifestInstructionEffect::Invocation { kind, args } = effect else {
            return ControlFlow::Continue(());
        };

        // Handle the invocation and get its static information back.
        let invocation_static_information = self.handle_invocation(kind, args, index)?;

        // Adding the static information to the state to surface later to the consumer.
        self.invocation_static_information
            .insert(index, invocation_static_information);

        ControlFlow::Continue(())
    }
    // endregion:Invocation

    // region:Bucket Creation
    fn on_new_bucket<'a>(
        &mut self,
        OnNewBucket { bucket, state }: OnNewBucket<'_, 'a>,
    ) -> ControlFlow<Self::Error<'a>> {
        // Converting the resource address into a composite resource address and then acting based
        // on whether the resource is fungible or non-fungible.
        let composite_resource_address =
            CompositeResourceAddress::from(*state.source_amount.resource_address());

        match (composite_resource_address, state.source_amount) {
            // Everything on the worktop is being taken so we remove it from the worktop contents.
            // If the resource was not known to be in the worktop then we create unknown bounds for
            // it.
            (
                CompositeResourceAddress::Fungible(fungible_resource_address),
                BucketSourceAmount::AllOnWorktop { .. },
            ) => {
                self.tracked_buckets.insert(
                    bucket,
                    BucketContent::Fungible(
                        fungible_resource_address,
                        self.worktop_fungible_contents
                            .swap_remove(&fungible_resource_address)
                            .unwrap_or(FungibleBounds {
                                lower: LowerFungibleBound::NonZero,
                                upper: UpperFungibleBound::Unbounded,
                            }),
                    ),
                );
            }
            (
                CompositeResourceAddress::NonFungible(non_fungible_resource_address),
                BucketSourceAmount::AllOnWorktop { .. },
            ) => {
                self.tracked_buckets.insert(
                    bucket,
                    BucketContent::NonFungible(
                        non_fungible_resource_address,
                        self.worktop_non_fungible_contents
                            .swap_remove(&non_fungible_resource_address)
                            .unwrap_or(NonFungibleBounds {
                                amount_bounds: FungibleBounds {
                                    lower: LowerFungibleBound::NonZero,
                                    upper: UpperFungibleBound::Unbounded,
                                },
                                id_bounds: NonFungibleIdBounds::Unknown,
                            }),
                    ),
                );
            }
            // A fungible amount is being taken from the worktop. In the case of fungible resources
            // the fungible amount will just be deducted from the bounds if bounds are defined. If
            // the worktop doesn't have knowledge of this resource being here then a bucket with a
            // guaranteed amount is created,
            (
                CompositeResourceAddress::Fungible(fungible_resource_address),
                BucketSourceAmount::AmountFromWorktop {
                    amount: bucket_amount,
                    ..
                },
            ) => {
                // Check if there's an entry for this resource on the worktop. If there is, then we
                // subtract the amount taken from the bounds when they're defined.
                // Worktop accounting.
                if let Some(worktop_fungible_content) = self
                    .worktop_fungible_contents
                    .get_mut(&fungible_resource_address)
                {
                    match worktop_fungible_content.decrease_both_bounds(bucket_amount) {
                        Some(v) => ControlFlow::Continue(v),
                        None => ControlFlow::Break(StaticResourceMovementsError::DecimalOverflow),
                    }?;
                }
                // Creation of bucket
                self.tracked_buckets.insert(
                    bucket,
                    BucketContent::Fungible(
                        fungible_resource_address,
                        FungibleBounds {
                            lower: LowerFungibleBound::Amount(bucket_amount),
                            upper: UpperFungibleBound::Amount(bucket_amount),
                        },
                    ),
                );
            }
            // Taking an amount from the worktop of a non-fungible. We can't tell which non-fungible
            // ids are being taken so the bucket will just contain unknown ids.
            (
                CompositeResourceAddress::NonFungible(non_fungible_resource_address),
                BucketSourceAmount::AmountFromWorktop {
                    amount: bucket_amount,
                    ..
                },
            ) => {
                // Worktop accounting.
                if let Some(worktop_non_fungible_content) = self
                    .worktop_non_fungible_contents
                    .get_mut(&non_fungible_resource_address)
                {
                    // Reduce the amount bounds.
                    worktop_non_fungible_content
                        .amount_bounds
                        .decrease_both_bounds(bucket_amount);
                    // Switch the id bounds to be unknown.
                    worktop_non_fungible_content.id_bounds = NonFungibleIdBounds::Unknown;
                }
                // Creation of bucket.
                self.tracked_buckets.insert(
                    bucket,
                    BucketContent::NonFungible(
                        non_fungible_resource_address,
                        NonFungibleBounds {
                            amount_bounds: FungibleBounds::new_exact(bucket_amount),
                            id_bounds: NonFungibleIdBounds::Unknown,
                        },
                    ),
                );
            }
            // Taking non-fungibles from the worktop by id.
            (
                CompositeResourceAddress::NonFungible(non_fungible_resource_address),
                BucketSourceAmount::NonFungiblesFromWorktop {
                    ids: bucket_ids, ..
                },
            ) => {
                let bucket_ids = bucket_ids.iter().cloned().collect::<IndexSet<_>>();
                let bucket_ids_amount = Decimal::from(bucket_ids.len());

                // Worktop accounting.
                if let Some(worktop_non_fungible_content) = self
                    .worktop_non_fungible_contents
                    .get_mut(&non_fungible_resource_address)
                {
                    // Reduce the amount bounds.
                    worktop_non_fungible_content
                        .amount_bounds
                        .decrease_both_bounds(bucket_ids_amount);

                    // Remove the ids from the set of ids in the worktop.
                    if let NonFungibleIdBounds::FullyKnown(ref mut id_bounds)
                    | NonFungibleIdBounds::PartiallyKnown(ref mut id_bounds) =
                        worktop_non_fungible_content.id_bounds
                    {
                        bucket_ids.iter().for_each(|id| {
                            id_bounds.swap_remove(id);
                        });
                    }
                }

                // Creation of bucket.
                self.tracked_buckets.insert(
                    bucket,
                    BucketContent::NonFungible(
                        non_fungible_resource_address,
                        NonFungibleBounds {
                            amount_bounds: FungibleBounds::new_exact(bucket_ids_amount),
                            id_bounds: NonFungibleIdBounds::FullyKnown(bucket_ids),
                        },
                    ),
                );
            }
            // Invalid case - taking a fungible by ids from the worktop.
            (
                CompositeResourceAddress::Fungible(_),
                BucketSourceAmount::NonFungiblesFromWorktop { .. },
            ) => {
                return ControlFlow::Break(
                    StaticResourceMovementsError::NonFungibleIdsTakeOnFungibleResource,
                );
            }
        }

        ControlFlow::Continue(())
    }
    // endregion:Bucket Creation

    // region:Bucket Consumption
    fn on_consume_bucket<'a>(
        &mut self,
        OnConsumeBucket {
            bucket,
            destination,
            ..
        }: OnConsumeBucket<'_, 'a>,
    ) -> ControlFlow<Self::Error<'a>> {
        // Try to get the bucket information. If the bucket information doesn't exist then throw an
        // error. There's no way for a bucket to be created without us catching its creation and
        // adding it to the tracked buckets.
        let Some(bucket_bounds) = self.tracked_buckets.swap_remove(&bucket) else {
            return ControlFlow::Break(StaticResourceMovementsError::BucketDoesntExist(bucket));
        };

        // The only bucket destination that matters is the worktop destination. Other than that, the
        // bucket could've been used in an invocation or burned at which case there's not much to do
        // aside from not tracking that bucket anymore, which was done above.
        let BucketDestination::Worktop = destination else {
            return ControlFlow::Continue(());
        };

        match bucket_bounds {
            BucketContent::Fungible(fungible_resource_address, bucket_bounds) => {
                // Get the entry for the fungible resource in the worktop. If one doesn't exist then
                // add it. If it does then we will perform a combination.
                match self
                    .worktop_fungible_contents
                    .get_mut(&fungible_resource_address)
                {
                    Some(fungible_worktop_content) => {
                        match fungible_worktop_content.combine(bucket_bounds) {
                            Some(value) => ControlFlow::Continue(value),
                            None => {
                                ControlFlow::Break(StaticResourceMovementsError::DecimalOverflow)
                            }
                        }?;
                    }
                    None => {
                        self.worktop_fungible_contents
                            .insert(fungible_resource_address, bucket_bounds);
                    }
                }
            }
            BucketContent::NonFungible(non_fungible_resource_address, bucket_bounds) => {
                // Get the entry for the non fungible resource in the worktop. If one doesn't exist
                // then add it. If it does then we will perform a combination.
                match self
                    .worktop_non_fungible_contents
                    .get_mut(&non_fungible_resource_address)
                {
                    Some(non_fungible_worktop_content) => {
                        match non_fungible_worktop_content.combine(bucket_bounds) {
                            Some(value) => ControlFlow::Continue(value),
                            None => {
                                ControlFlow::Break(StaticResourceMovementsError::DecimalOverflow)
                            }
                        }?;
                    }
                    None => {
                        self.worktop_non_fungible_contents
                            .insert(non_fungible_resource_address, bucket_bounds);
                    }
                }
            }
        }

        ControlFlow::Continue(())
    }
    // endregion:Bucket Consumption

    // region:Assertions
    fn on_worktop_assertion<'a>(
        &mut self,
        OnWorktopAssertion { assertion }: OnWorktopAssertion<'a>,
    ) -> ControlFlow<Self::Error<'a>> {
        // Handle the ability to empty the worktop.
        let resource_address = match assertion {
            WorktopAssertion::AnyAmountGreaterThanZero { resource_address }
            | WorktopAssertion::AtLeastAmount {
                resource_address, ..
            }
            | WorktopAssertion::AtLeastNonFungibles {
                resource_address, ..
            } => resource_address,
            WorktopAssertion::IsEmpty => {
                // Empty the worktop completely.
                self.worktop_fungible_contents = Default::default();
                self.worktop_non_fungible_contents = Default::default();
                self.worktop_uncertainty_sources = Default::default();
                return ControlFlow::Continue(());
            }
        };

        // Convert to a composite resource address.
        let composite_resource_address = CompositeResourceAddress::from(*resource_address);

        match (composite_resource_address, assertion) {
            // An assertion of any non-zero amount. This is only useful if the worktop doesn't
            // already know about this resource. If it does, then there's nothing more that this
            // can tell us than what we already know. Handling is the same between fungibles and
            // also non-fungibles.
            (
                CompositeResourceAddress::Fungible(fungible_resource_address),
                WorktopAssertion::AnyAmountGreaterThanZero { .. },
            ) => {
                self.worktop_fungible_contents
                    .entry(fungible_resource_address)
                    .or_insert(FungibleBounds {
                        lower: LowerFungibleBound::NonZero,
                        upper: UpperFungibleBound::Unbounded,
                    });
            }
            (
                CompositeResourceAddress::NonFungible(non_fungible_resource_address),
                WorktopAssertion::AnyAmountGreaterThanZero { .. },
            ) => {
                self.worktop_non_fungible_contents
                    .entry(non_fungible_resource_address)
                    .or_insert(NonFungibleBounds {
                        amount_bounds: FungibleBounds {
                            lower: LowerFungibleBound::NonZero,
                            upper: UpperFungibleBound::Unbounded,
                        },
                        id_bounds: NonFungibleIdBounds::Unknown,
                    });
            }
            // An assertion for an amount of resources. If a worktop entry for the resource doesn't
            // exist then it will be added with the specified amount as the lower bound and no upper
            // bound. Non-fungibles will of course not have any ids be known.
            //
            // If an entry does exist then it updates the amount's lower bound for both fungibles
            // and non-fungibles if the amount specified in the assertion is larger than that of
            // the lower bound.
            (
                CompositeResourceAddress::Fungible(fungible_resource_address),
                WorktopAssertion::AtLeastAmount { amount, .. },
            ) => {
                if let Some(fungible_contents) = self
                    .worktop_fungible_contents
                    .get_mut(&fungible_resource_address)
                {
                    fungible_contents.increase_lower_bound(amount);
                } else {
                    self.worktop_fungible_contents.insert(
                        fungible_resource_address,
                        FungibleBounds {
                            lower: LowerFungibleBound::Amount(amount),
                            upper: UpperFungibleBound::Unbounded,
                        },
                    );
                }
            }
            (
                CompositeResourceAddress::NonFungible(non_fungible_resource_address),
                WorktopAssertion::AtLeastAmount { amount, .. },
            ) => {
                if let Some(non_fungible_contents) = self
                    .worktop_non_fungible_contents
                    .get_mut(&non_fungible_resource_address)
                {
                    non_fungible_contents
                        .amount_bounds
                        .increase_lower_bound(amount);
                } else {
                    self.worktop_non_fungible_contents.insert(
                        non_fungible_resource_address,
                        NonFungibleBounds {
                            amount_bounds: FungibleBounds {
                                lower: LowerFungibleBound::Amount(amount),
                                upper: UpperFungibleBound::Unbounded,
                            },
                            id_bounds: NonFungibleIdBounds::Unknown,
                        },
                    );
                }
            }
            // An assertion that some non-fungibles are on the worktop. If no entry exists in the
            // worktop content then a new one will be added and the ids will be considered to be
            // partially known. Otherwise, if an entry exists in the worktop content then the ids
            // will be extended to it.
            (
                CompositeResourceAddress::NonFungible(non_fungible_resource_address),
                WorktopAssertion::AtLeastNonFungibles { ids, .. },
            ) => {
                let ids = ids.iter().cloned().collect::<IndexSet<_>>();
                let ids_amount = Decimal::from(ids.len());

                if let Some(non_fungible_contents) = self
                    .worktop_non_fungible_contents
                    .get_mut(&non_fungible_resource_address)
                {
                    // Attempt to increase the fungible lower bound to the amount of ids that is
                    // being asserted.
                    non_fungible_contents
                        .amount_bounds
                        .increase_lower_bound(ids_amount);

                    // We have a set of ids that we want to add to the bounds that we have on non
                    // fungible ids. The logic is going to depend on the state of the non-fungible
                    // bounds.
                    match non_fungible_contents.id_bounds {
                        // If they're fully known and an assertion comes with ids outside of the
                        // existing ids then it transitions to be partially known.
                        NonFungibleIdBounds::FullyKnown(ref existing_ids) => {
                            if !ids.iter().all(|item| existing_ids.contains(item)) {
                                let mut existing_ids = existing_ids.clone();
                                existing_ids.extend(ids);
                                non_fungible_contents.id_bounds =
                                    NonFungibleIdBounds::PartiallyKnown(existing_ids)
                            }
                        }
                        // If they were partially known then just extend the set of keys
                        NonFungibleIdBounds::PartiallyKnown(ref mut existing_ids) => {
                            existing_ids.extend(ids)
                        }
                        // If the ids used to be unknown then switch them to be partially known.
                        ref mut non_fungible_id_bounds @ NonFungibleIdBounds::Unknown => {
                            *non_fungible_id_bounds = NonFungibleIdBounds::PartiallyKnown(ids)
                        }
                    }
                } else {
                    self.worktop_non_fungible_contents.insert(
                        non_fungible_resource_address,
                        NonFungibleBounds {
                            amount_bounds: FungibleBounds {
                                lower: LowerFungibleBound::Amount(ids_amount),
                                upper: UpperFungibleBound::Unbounded,
                            },
                            id_bounds: NonFungibleIdBounds::PartiallyKnown(ids),
                        },
                    );
                }
            }
            (_, WorktopAssertion::IsEmpty) => {
                // Empty the worktop completely.
                self.worktop_fungible_contents = Default::default();
                self.worktop_non_fungible_contents = Default::default();
                self.worktop_uncertainty_sources = Default::default();
            }
            // This is invalid. You can't assert by ids on fungibles.
            (
                CompositeResourceAddress::Fungible(..),
                WorktopAssertion::AtLeastNonFungibles { .. },
            ) => {
                ControlFlow::Break(Self::Error::NonFungibleIdsAssertionOnFungibleResource)?;
            }
        }

        ControlFlow::Continue(())
    }
    // endregion:Assertions
}

fn option_to_control_flow<T, E>(option: Option<T>, error: E) -> ControlFlow<E, T> {
    match option {
        Some(value) => ControlFlow::Continue(value),
        None => ControlFlow::Break(error),
    }
}

fn result_to_control_flow<T, E>(result: Result<T, E>) -> ControlFlow<E, T> {
    match result {
        Ok(value) => ControlFlow::Continue(value),
        Err(value) => ControlFlow::Break(value),
    }
}
