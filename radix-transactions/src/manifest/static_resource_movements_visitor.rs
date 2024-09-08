use super::*;
use crate::model::*;
use radix_common::prelude::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::locker::*;
use std::ops::*;
use traversal::*;
use typed_invocations::*;

/// A [`ManifestInterpretationVisitor`] that statically tracks the resources in the worktop and
/// reports the account withdraws and deposits made.
pub struct StaticResourceMovementsVisitor {
    // TODO: How can we do this better? How can we abstract this so that it just does the static
    // asset movement and make the account stuff added on top?
    account_deposits: IndexMap<ComponentAddress, Vec<AccountDeposit>>,
    account_withdraws: IndexMap<ComponentAddress, Vec<AccountWithdraw>>,
    /// The resource content of the worktop.
    worktop_fungible_contents: IndexMap<FungibleResourceAddress, FungibleBounds>,
    /// The resource content of the worktop.
    worktop_non_fungible_contents: IndexMap<NonFungibleResourceAddress, NonFungibleBounds>,
    /// The sources of uncertainty about the worktop.
    worktop_uncertainty_sources: Vec<WorktopUncertaintySource>,
    /// The buckets tracked by the by the visitor.
    tracked_buckets: IndexMap<ManifestBucket, BucketContent>,
}

impl StaticResourceMovementsVisitor {
    pub fn new(initial_worktop_state_is_unknown: bool) -> Self {
        let worktop_uncertainty_sources = if initial_worktop_state_is_unknown {
            vec![WorktopUncertaintySource::YieldFromParent]
        } else {
            vec![]
        };
        Self {
            account_deposits: Default::default(),
            account_withdraws: Default::default(),
            tracked_buckets: Default::default(),
            worktop_fungible_contents: Default::default(),
            worktop_non_fungible_contents: Default::default(),
            worktop_uncertainty_sources,
        }
    }

    pub fn output(
        self,
    ) -> (
        IndexMap<ComponentAddress, Vec<AccountDeposit>>,
        IndexMap<ComponentAddress, Vec<AccountWithdraw>>,
    ) {
        (self.account_deposits, self.account_withdraws)
    }
}

impl ManifestInterpretationVisitor for StaticResourceMovementsVisitor {
    type Error<'a> = StaticResourceMovementsError<'a>;

    // region:Invocation
    fn on_next_instruction<'a>(
        &mut self,
        index: usize,
        effect: ManifestInstructionEffect,
    ) -> ControlFlow<Self::Error<'a>> {
        // We only care about invocations. Ignore anything that is not an invocation.
        let ManifestInstructionEffect::Invocation { kind, args } = effect else {
            return ControlFlow::Continue(());
        };

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
            let mut buckets = IndexSet::new();
            let mut expressions = IndexSet::new();
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
        let mut invocation_inputs = result_to_control_flow(
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
            invocation_inputs.extend(
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
        }

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
            ),
            InvocationKind::Function {
                address: DynamicPackageAddress::Static(package_address),
                blueprint,
                function,
            } => TypedNativeInvocation::from_function_invocation(
                package_address.as_node_id(),
                blueprint,
                function,
                args,
            ),
            // Can't convert into a typed native invocation.
            InvocationKind::DirectMethod { .. }
            | InvocationKind::YieldToParent
            | InvocationKind::YieldToChild { .. }
            | InvocationKind::VerifyParent
            | InvocationKind::Method { .. }
            | InvocationKind::Function { .. } => None,
        };

        // Handle the account deposits and withdraws.
        match &typed_native_invocation {
            // Withdraws
            Some(TypedNativeInvocation::AccountPackage(AccountInvocations::AccountBlueprint(
                AccountBlueprintInvocations::Method(
                    account,
                    AccountMethod::Withdraw(AccountWithdrawInput {
                        resource_address,
                        amount,
                    })
                    | AccountMethod::LockFeeAndWithdraw(AccountLockFeeAndWithdrawInput {
                        resource_address,
                        amount,
                        ..
                    }),
                ),
            ))) => {
                self.account_withdraws
                    .entry(*account)
                    .or_default()
                    .push(AccountWithdraw::Amount(*resource_address, *amount));
            }
            Some(TypedNativeInvocation::AccountPackage(AccountInvocations::AccountBlueprint(
                AccountBlueprintInvocations::Method(
                    account,
                    AccountMethod::WithdrawNonFungibles(AccountWithdrawNonFungiblesInput {
                        resource_address,
                        ids,
                    })
                    | AccountMethod::LockFeeAndWithdrawNonFungibles(
                        AccountLockFeeAndWithdrawNonFungiblesInput {
                            resource_address,
                            ids,
                            ..
                        },
                    ),
                ),
            ))) => {
                self.account_withdraws
                    .entry(*account)
                    .or_default()
                    .push(AccountWithdraw::Ids(*resource_address, ids.clone()));
            }
            // Deposits
            Some(TypedNativeInvocation::AccountPackage(AccountInvocations::AccountBlueprint(
                AccountBlueprintInvocations::Method(
                    account,
                    AccountMethod::Deposit(..)
                    | AccountMethod::DepositBatch(..)
                    | AccountMethod::TryDepositOrAbort(..)
                    | AccountMethod::TryDepositBatchOrAbort(..),
                ),
            ))) => {
                self.account_deposits
                    .entry(*account)
                    .or_default()
                    .extend(invocation_inputs.iter().cloned().map(AccountDeposit::from));
            }
            _ => {}
        }

        // Handle the worktop puts due to the invocation. Takes are handled by the bucket creation.
        match typed_native_invocation {
            Some(TypedNativeInvocation::AccessControllerPackage(
                AccessControllerInvocations::AccessControllerBlueprint(
                    access_controller_invocations,
                ),
            )) => {
                match access_controller_invocations {
                    AccessControllerBlueprintInvocations::Function(function_invocation) => {
                        match function_invocation {
                            AccessControllerFunction::Create(_) => {}
                        }
                    }
                    AccessControllerBlueprintInvocations::Method(_, method_invocations) => {
                        match method_invocations {
                            // Known effect
                            AccessControllerMethod::WithdrawRecoveryFee(withdraw_recovery_fee) => {
                                let fungible_resource_address = FungibleResourceAddress(XRD);
                                let fungible_bounds = FungibleBounds::new_exact(withdraw_recovery_fee.amount);
                                match self.worktop_fungible_contents.get_mut(&fungible_resource_address) {
                                    Some(worktop_content) => {
                                        match worktop_content.combine(fungible_bounds) {
                                            Some(v) => ControlFlow::Continue(v),
                                            None => ControlFlow::Break(Self::Error::DecimalOverflow),
                                        }?;
                                    }
                                    None => {
                                        self.worktop_fungible_contents
                                            .insert(fungible_resource_address, fungible_bounds);
                                    }
                                }
                            }
                            // No effect on worktop.
                            AccessControllerMethod::CreateProof(..)
                            | AccessControllerMethod::InitiateRecoveryAsPrimary(..)
                            | AccessControllerMethod::InitiateRecoveryAsRecovery(..)
                            | AccessControllerMethod::QuickConfirmPrimaryRoleRecoveryProposal(..)
                            | AccessControllerMethod::QuickConfirmRecoveryRoleRecoveryProposal(..)
                            | AccessControllerMethod::TimedConfirmRecovery(..)
                            | AccessControllerMethod::StopTimedRecovery(..)
                            | AccessControllerMethod::MintRecoveryBadges(..)
                            | AccessControllerMethod::LockRecoveryFee(..)
                            | AccessControllerMethod::ContributeRecoveryFee(..)
                            | AccessControllerMethod::InitiateBadgeWithdrawAttemptAsPrimary(..)
                            | AccessControllerMethod::InitiateBadgeWithdrawAttemptAsRecovery(..)
                            | AccessControllerMethod::CancelPrimaryRoleRecoveryProposal(..)
                            | AccessControllerMethod::CancelRecoveryRoleRecoveryProposal(..)
                            | AccessControllerMethod::CancelPrimaryRoleBadgeWithdrawAttempt(..)
                            | AccessControllerMethod::CancelRecoveryRoleBadgeWithdrawAttempt(..)
                            | AccessControllerMethod::LockPrimaryRole(..)
                            | AccessControllerMethod::UnlockPrimaryRole(..) => {}
                            // Puts worktop in unknown state.
                            AccessControllerMethod::QuickConfirmPrimaryRoleBadgeWithdrawAttempt(..)
                            | AccessControllerMethod::QuickConfirmRecoveryRoleBadgeWithdrawAttempt(..) => {
                                self.worktop_uncertainty_sources
                                    .push(WorktopUncertaintySource::Invocation {
                                        instruction_index: index,
                                    });
                            }
                        }
                    }
                }
            }
            Some(TypedNativeInvocation::AccountPackage(AccountInvocations::AccountBlueprint(
                account_invocations,
            ))) => {
                match account_invocations {
                    AccountBlueprintInvocations::Function(function_invocation) => {
                        match function_invocation {
                            // No effect on worktop.
                            AccountFunction::CreateAdvanced(_) => {}
                            // Puts worktop in unknown state.
                            AccountFunction::Create(_) => {
                                self.worktop_uncertainty_sources.push(
                                    WorktopUncertaintySource::Invocation {
                                        instruction_index: index,
                                    },
                                );
                            }
                        }
                    }
                    AccountBlueprintInvocations::Method(_, method_invocation) => {
                        match method_invocation {
                            // Known effect
                            AccountMethod::WithdrawNonFungibles(
                                AccountWithdrawNonFungiblesInput {
                                    resource_address,
                                    ids,
                                },
                            )
                            | AccountMethod::LockFeeAndWithdrawNonFungibles(
                                AccountLockFeeAndWithdrawNonFungiblesInput {
                                    resource_address,
                                    ids,
                                    ..
                                },
                            ) => {
                                let CompositeResourceAddress::NonFungible(
                                    non_fungible_resource_address,
                                ) = CompositeResourceAddress::from(resource_address)
                                else {
                                    return ControlFlow::Break(
                                        StaticResourceMovementsError::AccountWithdrawNonFungiblesOnAFungibleResource,
                                    );
                                };
                                let bound = NonFungibleBounds::new_exact(ids);

                                match self
                                    .worktop_non_fungible_contents
                                    .get_mut(&non_fungible_resource_address)
                                {
                                    Some(worktop_non_fungible_bounds) => {
                                        worktop_non_fungible_bounds.combine(bound);
                                    }
                                    None => {
                                        self.worktop_non_fungible_contents
                                            .insert(non_fungible_resource_address, bound);
                                    }
                                }
                            }
                            AccountMethod::Withdraw(AccountWithdrawInput {
                                resource_address,
                                amount,
                            })
                            | AccountMethod::LockFeeAndWithdraw(AccountLockFeeAndWithdrawInput {
                                resource_address,
                                amount,
                                ..
                            }) => {
                                let composite_resource_address =
                                    CompositeResourceAddress::from(resource_address);
                                match composite_resource_address {
                                    CompositeResourceAddress::Fungible(
                                        fungible_resource_address,
                                    ) => {
                                        let bound = FungibleBounds::new_exact(amount);
                                        match self
                                            .worktop_fungible_contents
                                            .get_mut(&fungible_resource_address)
                                        {
                                            Some(worktop_fungible_bounds) => {
                                                worktop_fungible_bounds.combine(bound);
                                            }
                                            None => {
                                                self.worktop_fungible_contents
                                                    .insert(fungible_resource_address, bound);
                                            }
                                        }
                                    }
                                    CompositeResourceAddress::NonFungible(
                                        non_fungible_resource_address,
                                    ) => {
                                        let bound = NonFungibleBounds {
                                            amount_bounds: FungibleBounds::new_exact(amount),
                                            id_bounds: NonFungibleIdBounds::Unknown,
                                        };
                                        match self
                                            .worktop_non_fungible_contents
                                            .get_mut(&non_fungible_resource_address)
                                        {
                                            Some(worktop_non_fungible_bounds) => {
                                                worktop_non_fungible_bounds.combine(bound);
                                            }
                                            None => {
                                                self.worktop_non_fungible_contents
                                                    .insert(non_fungible_resource_address, bound);
                                            }
                                        }
                                    }
                                }
                            }
                            AccountMethod::TryDepositOrRefund(_)
                            | AccountMethod::TryDepositBatchOrRefund(_) => {
                                // The case of the InvocationIos all being consumed by the call is
                                // easy to do. They're just not used and the worktop becomes empty
                                // of them. Otherwise, we assume that they were all returned back to
                                // the worktop and we process them all.
                                for invocation_input in invocation_inputs {
                                    match invocation_input {
                                        InvocationIo::KnownFungible(
                                            fungible_resource_address,
                                            fungible_bounds,
                                        ) => {
                                            // If no entry exists in the worktop content then add
                                            // one.
                                            match self
                                                .worktop_fungible_contents
                                                .get_mut(&fungible_resource_address)
                                            {
                                                Some(fungible_worktop_content) => {
                                                    match (
                                                        fungible_worktop_content.upper,
                                                        fungible_bounds.upper,
                                                    ) {
                                                        (
                                                            UpperFungibleBound::Amount(
                                                                ref mut amount1,
                                                            ),
                                                            UpperFungibleBound::Amount(amount2),
                                                        ) => {
                                                            *amount1 = option_to_control_flow(
                                                                amount1.checked_add(amount2),
                                                                StaticResourceMovementsError::DecimalOverflow,
                                                            )?;
                                                        }
                                                        (_, UpperFungibleBound::Unbounded)
                                                        | (UpperFungibleBound::Unbounded, _) => {
                                                            fungible_worktop_content.upper =
                                                                UpperFungibleBound::Unbounded
                                                        }
                                                    }
                                                }
                                                None => {
                                                    self.worktop_fungible_contents.insert(
                                                        fungible_resource_address,
                                                        fungible_bounds,
                                                    );
                                                }
                                            }
                                        }
                                        InvocationIo::KnownNonFungible(
                                            non_fungible_resource_address,
                                            non_fungible_bounds,
                                        ) => {
                                            // If no entry exists in the worktop content then add
                                            // one.
                                            match self
                                                .worktop_non_fungible_contents
                                                .get_mut(&non_fungible_resource_address)
                                            {
                                                Some(non_fungible_worktop_content) => {
                                                    // Update the amounts
                                                    match (
                                                        non_fungible_worktop_content
                                                            .amount_bounds
                                                            .upper,
                                                        non_fungible_bounds.amount_bounds.upper,
                                                    ) {
                                                        (
                                                            UpperFungibleBound::Amount(
                                                                ref mut amount1,
                                                            ),
                                                            UpperFungibleBound::Amount(amount2),
                                                        ) => {
                                                            *amount1 = option_to_control_flow(
                                                                amount1.checked_add(amount2),
                                                                StaticResourceMovementsError::DecimalOverflow,
                                                            )?;
                                                        }
                                                        (_, UpperFungibleBound::Unbounded)
                                                        | (UpperFungibleBound::Unbounded, _) => {
                                                            non_fungible_worktop_content
                                                                .amount_bounds
                                                                .upper =
                                                                UpperFungibleBound::Unbounded
                                                        }
                                                    }

                                                    // Update the id bounds.
                                                    non_fungible_worktop_content
                                                        .combine(non_fungible_bounds);
                                                }
                                                None => {
                                                    self.worktop_non_fungible_contents.insert(
                                                        non_fungible_resource_address,
                                                        non_fungible_bounds,
                                                    );
                                                }
                                            }
                                        }
                                        // For the worktop uncertainty sources we just add them back
                                        // to the set of uncertainty sources.
                                        InvocationIo::Unknown(worktop_uncertainty_source) => self
                                            .worktop_uncertainty_sources
                                            .push(worktop_uncertainty_source),
                                    }
                                }
                            }
                            // No effect on worktop.
                            AccountMethod::LockFee(_)
                            | AccountMethod::LockContingentFee(_)
                            | AccountMethod::Deposit(_)
                            | AccountMethod::DepositBatch(_)
                            | AccountMethod::Burn(_)
                            | AccountMethod::BurnNonFungibles(_)
                            | AccountMethod::AddAuthorizedDepositor(_)
                            | AccountMethod::RemoveAuthorizedDepositor(_)
                            | AccountMethod::CreateProofOfAmount(_)
                            | AccountMethod::CreateProofOfNonFungibles(_)
                            | AccountMethod::SetDefaultDepositRule(_)
                            | AccountMethod::SetResourcePreference(_)
                            | AccountMethod::RemoveResourcePreference(_)
                            | AccountMethod::TryDepositOrAbort(_)
                            | AccountMethod::TryDepositBatchOrAbort(_) => {}
                            // Puts worktop in unknown state.
                            AccountMethod::Securify(_) => {
                                self.worktop_uncertainty_sources.push(
                                    WorktopUncertaintySource::Invocation {
                                        instruction_index: index,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            Some(TypedNativeInvocation::ConsensusManagerPackage(
                ConsensusManagerInvocations::ConsensusManagerBlueprint(
                    consensus_manager_invocations,
                ),
            )) => {
                match consensus_manager_invocations {
                    ConsensusManagerBlueprintInvocations::Function(function_invocation) => {
                        match function_invocation {
                            ConsensusManagerFunction::Create(_) => {}
                        }
                    }
                    ConsensusManagerBlueprintInvocations::Method(_, method_invocation) => {
                        match method_invocation {
                            // No effect.
                            ConsensusManagerMethod::GetCurrentEpoch(_)
                            | ConsensusManagerMethod::Start(_)
                            | ConsensusManagerMethod::GetCurrentTime(_)
                            | ConsensusManagerMethod::NextRound(_) => {}
                            // Puts the worktop in unknown state.
                            ConsensusManagerMethod::CreateValidator(_) => {
                                self.worktop_uncertainty_sources.push(
                                    WorktopUncertaintySource::Invocation {
                                        instruction_index: index,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            Some(TypedNativeInvocation::ConsensusManagerPackage(
                ConsensusManagerInvocations::ValidatorBlueprint(consensus_manager_invocations),
            )) => {
                match consensus_manager_invocations {
                    ValidatorBlueprintInvocations::Function(function_invocation) => {
                        match function_invocation {}
                    }
                    ValidatorBlueprintInvocations::Method(_, method_invocation) => {
                        match method_invocation {
                            // Unknown effect
                            ValidatorMethod::StakeAsOwner(_)
                            | ValidatorMethod::Stake(_)
                            | ValidatorMethod::Unstake(_)
                            | ValidatorMethod::ClaimXrd(_)
                            | ValidatorMethod::FinishUnlockOwnerStakeUnits(_) => {
                                self.worktop_uncertainty_sources.push(
                                    WorktopUncertaintySource::Invocation {
                                        instruction_index: index,
                                    },
                                );
                            }
                            // No effect on worktop
                            ValidatorMethod::Register(_)
                            | ValidatorMethod::Unregister(_)
                            | ValidatorMethod::UpdateKey(_)
                            | ValidatorMethod::UpdateFee(_)
                            | ValidatorMethod::UpdateAcceptDelegatedStake(_)
                            | ValidatorMethod::AcceptsDelegatedStake(_)
                            | ValidatorMethod::TotalStakeXrdAmount(_)
                            | ValidatorMethod::TotalStakeUnitSupply(_)
                            | ValidatorMethod::GetRedemptionValue(_)
                            | ValidatorMethod::SignalProtocolUpdateReadiness(_)
                            | ValidatorMethod::GetProtocolUpdateReadiness(_)
                            | ValidatorMethod::LockOwnerStakeUnits(_)
                            | ValidatorMethod::StartUnlockOwnerStakeUnits(_) => {}
                        }
                    }
                }
            }
            Some(TypedNativeInvocation::IdentityPackage(
                IdentityInvocations::IdentityBlueprint(identity_invocations),
            )) => match identity_invocations {
                IdentityBlueprintInvocations::Function(function_invocation) => {
                    match function_invocation {
                        IdentityFunction::Create(_) => {
                            self.worktop_uncertainty_sources.push(
                                WorktopUncertaintySource::Invocation {
                                    instruction_index: index,
                                },
                            );
                        }
                        IdentityFunction::CreateAdvanced(_) => {}
                    }
                }
                IdentityBlueprintInvocations::Method(_, method_invocation) => {
                    match method_invocation {
                        IdentityMethod::Securify(_) => {
                            self.worktop_uncertainty_sources.push(
                                WorktopUncertaintySource::Invocation {
                                    instruction_index: index,
                                },
                            );
                        }
                    }
                }
            },
            Some(TypedNativeInvocation::LockerPackage(
                LockerInvocations::AccountLockerBlueprint(account_locker_invocations),
            )) => {
                match account_locker_invocations {
                    AccountLockerBlueprintInvocations::Function(function_invocation) => {
                        match function_invocation {
                            AccountLockerFunction::Instantiate(_) => {
                                self.worktop_uncertainty_sources.push(
                                    WorktopUncertaintySource::Invocation {
                                        instruction_index: index,
                                    },
                                );
                            }
                            AccountLockerFunction::InstantiateSimple(_) => {}
                        }
                    }
                    AccountLockerBlueprintInvocations::Method(_, method_invocation) => {
                        match method_invocation {
                            // No effect
                            AccountLockerMethod::Store(_)
                            | AccountLockerMethod::Airdrop(_)
                            | AccountLockerMethod::GetAmount(_)
                            | AccountLockerMethod::GetNonFungibleLocalIds(_) => {}
                            // Known effect
                            AccountLockerMethod::Recover(AccountLockerRecoverManifestInput {
                                resource_address,
                                amount,
                                ..
                            })
                            | AccountLockerMethod::Claim(AccountLockerClaimManifestInput {
                                resource_address,
                                amount,
                                ..
                            }) => {
                                let composite_resource_address =
                                    CompositeResourceAddress::from(resource_address);
                                match composite_resource_address {
                                    CompositeResourceAddress::Fungible(
                                        fungible_resource_address,
                                    ) => {
                                        let bound = FungibleBounds::new_exact(amount);
                                        match self
                                            .worktop_fungible_contents
                                            .get_mut(&fungible_resource_address)
                                        {
                                            Some(worktop_fungible_bounds) => {
                                                worktop_fungible_bounds.combine(bound);
                                            }
                                            None => {
                                                self.worktop_fungible_contents
                                                    .insert(fungible_resource_address, bound);
                                            }
                                        }
                                    }
                                    CompositeResourceAddress::NonFungible(
                                        non_fungible_resource_address,
                                    ) => {
                                        let bound = NonFungibleBounds {
                                            amount_bounds: FungibleBounds::new_exact(amount),
                                            id_bounds: NonFungibleIdBounds::Unknown,
                                        };
                                        match self
                                            .worktop_non_fungible_contents
                                            .get_mut(&non_fungible_resource_address)
                                        {
                                            Some(worktop_non_fungible_bounds) => {
                                                worktop_non_fungible_bounds.combine(bound);
                                            }
                                            None => {
                                                self.worktop_non_fungible_contents
                                                    .insert(non_fungible_resource_address, bound);
                                            }
                                        }
                                    }
                                }
                            }
                            AccountLockerMethod::RecoverNonFungibles(
                                AccountLockerRecoverNonFungiblesManifestInput {
                                    resource_address,
                                    ids,
                                    ..
                                },
                            )
                            | AccountLockerMethod::ClaimNonFungibles(
                                AccountLockerClaimNonFungiblesManifestInput {
                                    resource_address,
                                    ids,
                                    ..
                                },
                            ) => {
                                let CompositeResourceAddress::NonFungible(
                                    non_fungible_resource_address,
                                ) = CompositeResourceAddress::from(resource_address)
                                else {
                                    return ControlFlow::Break(
                                        StaticResourceMovementsError::AccountLockerWithdrawNonFungiblesOnAFungibleResource,
                                    );
                                };
                                let bound = NonFungibleBounds::new_exact(ids);

                                match self
                                    .worktop_non_fungible_contents
                                    .get_mut(&non_fungible_resource_address)
                                {
                                    Some(worktop_non_fungible_bounds) => {
                                        worktop_non_fungible_bounds.combine(bound);
                                    }
                                    None => {
                                        self.worktop_non_fungible_contents
                                            .insert(non_fungible_resource_address, bound);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Not a native invocation. The worktop will contain unknown resources.
            None => {
                self.worktop_uncertainty_sources
                    .push(WorktopUncertaintySource::Invocation {
                        instruction_index: index,
                    });
            }
        }

        ControlFlow::Continue(())
    }
    // endregion:Invocation

    // region:Bucket Creation
    fn on_new_bucket<'a>(
        &mut self,
        bucket: ManifestBucket,
        resource_address: &ResourceAddress,
        source_amount: BucketSourceAmount,
    ) -> ControlFlow<Self::Error<'a>> {
        // Converting the resource address into a composite resource address and then acting based
        // on whether the resource is fungible or non-fungible.
        let composite_resource_address = CompositeResourceAddress::from(*resource_address);

        match (composite_resource_address, source_amount) {
            // Everything on the worktop is being taken so we remove it from the worktop contents.
            // If the resource was not known to be in the worktop then we create unknown bounds for
            // it.
            (
                CompositeResourceAddress::Fungible(fungible_resource_address),
                BucketSourceAmount::AllOnWorktop,
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
                BucketSourceAmount::AllOnWorktop,
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
                BucketSourceAmount::AmountFromWorktop(bucket_amount),
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
                BucketSourceAmount::AmountFromWorktop(bucket_amount),
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
                BucketSourceAmount::NonFungiblesFromWorktop(bucket_ids),
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
                BucketSourceAmount::NonFungiblesFromWorktop(_),
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
        bucket: ManifestBucket,
        destination: BucketDestination,
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
        assertion: WorktopAssertion,
    ) -> ControlFlow<Self::Error<'a>> {
        // Convert to a composite resource address.
        let resource_address = assertion.resource_address();
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

#[derive(Clone, Debug)]
pub enum StaticResourceMovementsError<'a> {
    DecimalOverflow,
    NonFungibleIdsTakeOnFungibleResource,
    NonFungibleIdsAssertionOnFungibleResource,
    AccountWithdrawNonFungiblesOnAFungibleResource,
    AccountLockerWithdrawNonFungiblesOnAFungibleResource,
    BucketDoesntExist(ManifestBucket),
    ManifestValidationError(ManifestValidationError<'a>),
}

impl<'a> From<ManifestValidationError<'a>> for StaticResourceMovementsError<'a> {
    fn from(value: ManifestValidationError<'a>) -> Self {
        Self::ManifestValidationError(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountWithdraw {
    Amount(ResourceAddress, Decimal),
    Ids(ResourceAddress, IndexSet<NonFungibleLocalId>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountDeposit {
    KnownFungible(ResourceAddress, FungibleBounds),
    KnownNonFungible(ResourceAddress, NonFungibleBounds),
    Unknown(WorktopUncertaintySource),
}

impl From<InvocationIo> for AccountDeposit {
    fn from(value: InvocationIo) -> Self {
        match value {
            InvocationIo::KnownFungible(address, bound) => Self::KnownFungible(address.0, bound),
            InvocationIo::KnownNonFungible(address, bound) => {
                Self::KnownNonFungible(address.0, bound)
            }
            InvocationIo::Unknown(uncertainty) => Self::Unknown(uncertainty),
        }
    }
}

#[derive(Clone, Debug)]
enum BucketContent {
    Fungible(FungibleResourceAddress, FungibleBounds),
    NonFungible(NonFungibleResourceAddress, NonFungibleBounds),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FungibleBounds {
    pub lower: LowerFungibleBound,
    pub upper: UpperFungibleBound,
}

impl FungibleBounds {
    pub fn new_exact(amount: Decimal) -> Self {
        Self {
            lower: LowerFungibleBound::Amount(amount),
            upper: UpperFungibleBound::Amount(amount),
        }
    }

    pub fn combine(&mut self, other: Self) -> Option<()> {
        // Handling the lower bound.
        match (self.lower, other.lower) {
            // Two non-zero's produce a non-zero - no change needed.
            (LowerFungibleBound::NonZero, LowerFungibleBound::NonZero) => {}
            // Non-zero and an amount produces an amount.
            (LowerFungibleBound::NonZero, LowerFungibleBound::Amount(amount))
            | (LowerFungibleBound::Amount(amount), LowerFungibleBound::NonZero) => {
                self.lower = LowerFungibleBound::Amount(amount)
            }
            // Two amounts get added together
            (
                LowerFungibleBound::Amount(ref mut self_lower_bound),
                LowerFungibleBound::Amount(other_lower_bound),
            ) => {
                *self_lower_bound = self_lower_bound.checked_add(other_lower_bound)?;
            }
        };

        // Handling the upper bound.
        match (self.upper, other.upper) {
            // If both upper bounds are known then the new upper bound will also be
            // known.
            (
                UpperFungibleBound::Amount(ref mut self_upper_bound),
                UpperFungibleBound::Amount(other_upper_bound),
            ) => {
                *self_upper_bound = self_upper_bound.checked_add(other_upper_bound)?;
            }
            // If either of the upper bound is unbounded then the new upper bound
            // is also unbounded.
            (_, UpperFungibleBound::Unbounded) | (UpperFungibleBound::Unbounded, _) => {
                self.upper = UpperFungibleBound::Unbounded;
            }
        };

        Some(())
    }

    pub fn increase_lower_bound(&mut self, new_lower_bound: Decimal) {
        match (self.lower, self.upper) {
            // If the lower bound is non-zero and the upper bound is unbounded then we can just
            // update the lower bound without needing to worry about moving the upper bound.
            (ref mut lower_bound @ LowerFungibleBound::NonZero, UpperFungibleBound::Unbounded) => {
                *lower_bound = LowerFungibleBound::Amount(new_lower_bound);
            }
            // If the lower bound is non-zero and the upper bound is defined then we might need to
            // update both the upper and lower bound.
            (
                ref mut lower_bound @ LowerFungibleBound::NonZero,
                UpperFungibleBound::Amount(ref mut existing_upper_bound),
            ) => {
                *lower_bound = LowerFungibleBound::Amount(new_lower_bound);
                *existing_upper_bound = new_lower_bound.max(*existing_upper_bound);
            }
            // If a lower bound is defined and the upper bound is not defined then the new lower
            // bound would be the maximum of the existing lower bound and the new lower bound.
            (
                ref mut lower_bound @ LowerFungibleBound::Amount(existing_lower_bound),
                UpperFungibleBound::Unbounded,
            ) => {
                *lower_bound = LowerFungibleBound::Amount(existing_lower_bound.max(new_lower_bound))
            }
            // If both a lower and upper bound are numerically defined then we do the following:
            // - Set the lower bound to the max of the existing lower bound and the new lower bound.
            // - Set the upper bound to tbe the maximum of the new lower bound and the existing
            //   upper bound.
            // This is done to move both the lower and upper bounds in cases where they need to be
            // moved.
            (
                LowerFungibleBound::Amount(ref mut existing_lower_bound),
                UpperFungibleBound::Amount(ref mut existing_upper_bound),
            ) => {
                *existing_lower_bound = (*existing_lower_bound).max(new_lower_bound);
                *existing_upper_bound = (*existing_upper_bound).max(*existing_lower_bound);
            }
        }
    }

    fn decrease_both_bounds(&mut self, by: Decimal) -> Option<()> {
        match (self.lower, self.upper) {
            // The upper bound is being reduced by some amount. We first start by reducing the upper
            // bound and then check to ensure that it is not zero. If it is equal to zero then the
            // bounds will be changed to an exact of zero.
            (LowerFungibleBound::NonZero, UpperFungibleBound::Amount(ref mut upper_bound)) => {
                // Reduce the upper bound.
                *upper_bound = upper_bound.checked_sub(by)?.max(Decimal::ZERO);

                // Check if the upper bound is now zero. If it is, then we switch to being an exact
                // zero.
                if *upper_bound <= Decimal::ZERO {
                    *self = Self::new_exact(Decimal::ZERO);
                }
            }
            // The lower bound is not zero and the upper bound is unbounded. Can't do anything at
            // all here.
            (LowerFungibleBound::NonZero, UpperFungibleBound::Unbounded) => {}
            // Both an upper and a lower bound are defined. Reduce both and limit both to a min of
            // zero.
            (
                LowerFungibleBound::Amount(ref mut lower_bound),
                UpperFungibleBound::Amount(ref mut upper_bound),
            ) => {
                *lower_bound = lower_bound.checked_sub(by)?.max(Decimal::ZERO);
                *upper_bound = upper_bound.checked_sub(by)?.max(Decimal::ZERO);
            }
            // Only a lower bound is defined while the upper bound is not defined. So, we just
            // reduce the lower bound.
            (LowerFungibleBound::Amount(ref mut lower_bound), UpperFungibleBound::Unbounded) => {
                *lower_bound = lower_bound.checked_sub(by)?.max(Decimal::ZERO);
            }
        };
        Some(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LowerFungibleBound {
    NonZero,
    Amount(Decimal),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UpperFungibleBound {
    Amount(Decimal),
    Unbounded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NonFungibleBounds {
    pub amount_bounds: FungibleBounds,
    pub id_bounds: NonFungibleIdBounds,
}

impl NonFungibleBounds {
    pub fn new_exact(ids: IndexSet<NonFungibleLocalId>) -> Self {
        Self {
            amount_bounds: FungibleBounds::new_exact(ids.len().into()),
            id_bounds: NonFungibleIdBounds::FullyKnown(ids),
        }
    }

    pub fn combine(&mut self, other: Self) -> Option<()> {
        // Combine the fungible amounts according to the fungible rules.
        self.amount_bounds.combine(other.amount_bounds);

        // Combine the id bounds.
        match (&mut self.id_bounds, other.id_bounds) {
            // Add both sets together
            (
                NonFungibleIdBounds::FullyKnown(ref mut ids1),
                NonFungibleIdBounds::FullyKnown(ids2),
            ) => {
                ids1.extend(ids2);
            }
            // Convert to partially known.
            (
                NonFungibleIdBounds::PartiallyKnown(ref mut ids1),
                NonFungibleIdBounds::FullyKnown(ids2),
            )
            | (
                NonFungibleIdBounds::FullyKnown(ref mut ids1),
                NonFungibleIdBounds::PartiallyKnown(ids2),
            )
            | (
                NonFungibleIdBounds::PartiallyKnown(ref mut ids1),
                NonFungibleIdBounds::PartiallyKnown(ids2),
            ) => {
                ids1.extend(ids2);
                let ids = std::mem::replace(ids1, IndexSet::new());
                self.id_bounds = NonFungibleIdBounds::PartiallyKnown(ids);
            }
            (NonFungibleIdBounds::FullyKnown(ref mut ids), NonFungibleIdBounds::Unknown) => {
                let ids = std::mem::replace(ids, IndexSet::new());
                self.id_bounds = NonFungibleIdBounds::PartiallyKnown(ids);
            }
            (NonFungibleIdBounds::Unknown, NonFungibleIdBounds::FullyKnown(ids))
            | (NonFungibleIdBounds::Unknown, NonFungibleIdBounds::PartiallyKnown(ids)) => {
                self.id_bounds = NonFungibleIdBounds::PartiallyKnown(ids)
            }
            // No changes
            (NonFungibleIdBounds::Unknown, NonFungibleIdBounds::Unknown)
            | (NonFungibleIdBounds::PartiallyKnown(_), NonFungibleIdBounds::Unknown) => {}
        };

        Some(())
    }
}

#[derive(Clone, Debug)]
enum InvocationIo {
    KnownFungible(FungibleResourceAddress, FungibleBounds),
    KnownNonFungible(NonFungibleResourceAddress, NonFungibleBounds),
    Unknown(WorktopUncertaintySource),
}

impl From<BucketContent> for InvocationIo {
    fn from(value: BucketContent) -> Self {
        match value {
            BucketContent::Fungible(address, bounds) => Self::KnownFungible(address, bounds),
            BucketContent::NonFungible(address, bounds) => Self::KnownNonFungible(address, bounds),
        }
    }
}

impl From<(FungibleResourceAddress, FungibleBounds)> for InvocationIo {
    fn from((address, bounds): (FungibleResourceAddress, FungibleBounds)) -> Self {
        Self::KnownFungible(address, bounds)
    }
}

impl From<(NonFungibleResourceAddress, NonFungibleBounds)> for InvocationIo {
    fn from((address, bounds): (NonFungibleResourceAddress, NonFungibleBounds)) -> Self {
        Self::KnownNonFungible(address, bounds)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NonFungibleIdBounds {
    FullyKnown(IndexSet<NonFungibleLocalId>),
    PartiallyKnown(IndexSet<NonFungibleLocalId>),
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorktopUncertaintySource {
    YieldFromParent,
    Invocation { instruction_index: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompositeResourceAddress {
    Fungible(FungibleResourceAddress),
    NonFungible(NonFungibleResourceAddress),
}

impl CompositeResourceAddress {
    pub fn resource_address(&self) -> &ResourceAddress {
        match self {
            Self::Fungible(FungibleResourceAddress(address))
            | Self::NonFungible(NonFungibleResourceAddress(address)) => address,
        }
    }
}

impl From<ResourceAddress> for CompositeResourceAddress {
    fn from(value: ResourceAddress) -> Self {
        match value.is_fungible() {
            true => Self::Fungible(FungibleResourceAddress(value)),
            false => Self::NonFungible(NonFungibleResourceAddress(value)),
        }
    }
}

impl From<FungibleResourceAddress> for CompositeResourceAddress {
    fn from(value: FungibleResourceAddress) -> Self {
        Self::Fungible(value)
    }
}

impl From<NonFungibleResourceAddress> for CompositeResourceAddress {
    fn from(value: NonFungibleResourceAddress) -> Self {
        Self::NonFungible(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FungibleResourceAddress(ResourceAddress);

impl FungibleResourceAddress {
    pub fn new(address: ResourceAddress) -> Option<Self> {
        if address.is_fungible() {
            Some(Self(address))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonFungibleResourceAddress(ResourceAddress);

impl NonFungibleResourceAddress {
    pub fn new(address: ResourceAddress) -> Option<Self> {
        if !address.is_fungible() {
            Some(Self(address))
        } else {
            None
        }
    }
}

/// A module that contains a typed invocation types.
#[allow(clippy::enum_variant_names, dead_code, unused_variables)]
mod typed_invocations {
    use radix_engine_interface::blueprints::access_controller;
    use radix_engine_interface::blueprints::account;
    use radix_engine_interface::blueprints::consensus_manager;
    use radix_engine_interface::blueprints::identity;
    use radix_engine_interface::blueprints::locker;

    use radix_common::prelude::*;
    use radix_engine_interface::prelude::*;

    macro_rules! define_typed_invocations {
    (
        $(
            $package_name: ident => {
                $(
                    $blueprint_name: ident => {
                        type: $type: ty,
                        entity_type_pat: $entity_type_pat: pat,
                        module_id: $module_id: expr,
                        functions: {
                            $(
                                $func_ident: ident => ($func_input: ty, $func_name: expr $(,)?)
                            ),* $(,)?
                        },
                        methods: {
                            $(
                                $method_ident: ident => ($method_input: ty, $method_name: expr $(,)?)
                            ),* $(,)?
                        } $(,)?
                    }
                ),* $(,)?
            }
        ),* $(,)?
    ) => {
        paste::paste! {
            // There's a single typed invocation type that captures all of
            // the packages that we support.
            pub enum TypedNativeInvocation {
                $(
                    [< $package_name Package >]([< $package_name Invocations >])
                ),*
            }

            impl TypedNativeInvocation {
                pub fn from_method_invocation(
                    address: &NodeId,
                    module_id: ::radix_engine_interface::prelude::ModuleId,
                    method_name: &str,
                    args: &::radix_common::prelude::ManifestValue
                ) -> Option<Self> {
                    match (address.entity_type(), module_id) {
                        $(
                            $(
                                (Some($entity_type_pat), $module_id) => {
                                    [< $blueprint_name Method >]
                                        ::from_invocation(
                                            method_name,
                                            args
                                        )
                                        .and_then(|invocation| $type::try_from(address.as_bytes()).ok().map(|address| (address, invocation)))
                                        .map(|(address, invocation)|
                                            [< $blueprint_name BlueprintInvocations >]::Method(address, invocation)
                                        )
                                        .map([< $package_name Invocations >]::[< $blueprint_name Blueprint >])
                                        .map(Self::[< $package_name Package >])

                                }
                            )*
                        )*
                        _ => None
                    }
                }

                pub fn from_function_invocation(
                    address: &NodeId,
                    blueprint_name: &str,
                    function_name: &str,
                    args: &::radix_common::prelude::ManifestValue
                ) -> Option<Self> {
                    match (address.entity_type(), blueprint_name) {
                        $(
                            $(
                                (Some($entity_type_pat), stringify!($blueprint_name)) => {
                                    [< $blueprint_name Function >]
                                        ::from_invocation(
                                            function_name,
                                            args
                                        )
                                        .map(
                                            [< $blueprint_name BlueprintInvocations >]::Function
                                        )
                                        .map([< $package_name Invocations >]::[< $blueprint_name Blueprint >])
                                        .map(Self::[< $package_name Package >])

                                }
                            )*
                        )*
                        _ => None
                    }
                }
            }

            $(
                // For each package we define an invocation type that has all of the blueprints that
                // this package has.
                pub enum [< $package_name Invocations >] {
                    $(
                        [< $blueprint_name Blueprint >]([< $blueprint_name BlueprintInvocations >])
                    ),*
                }

                $(
                    // For each blueprint we define a type that's made up of the method and function
                    pub enum [< $blueprint_name BlueprintInvocations >] {
                        Function([< $blueprint_name Function >]),
                        Method($type, [< $blueprint_name Method >]),
                    }

                    pub enum [< $blueprint_name Method >] {
                        $(
                            $method_ident($method_input)
                        ),*
                    }

                    impl [< $blueprint_name Method >] {
                        #[allow(unreachable_patterns)]
                        pub fn method_name(&self) -> &str {
                            match self {
                                $(
                                    Self::$method_ident(..) => $method_name,
                                )*
                                _ => unreachable!()
                            }
                        }

                        pub fn from_invocation(
                            method_name: &str,
                            args: &::radix_common::prelude::ManifestValue
                        ) -> Option<Self> {
                            match method_name {
                                $(
                                    $method_name => ::radix_common::prelude::manifest_encode(args)
                                        .ok()
                                        .and_then(|value| ::radix_common::prelude::manifest_decode(&value).ok())
                                        .map(Self::$method_ident),
                                )*
                                _ => None
                            }
                        }
                    }

                    pub enum [< $blueprint_name Function >] {
                        $(
                            $func_ident($func_input)
                        ),*
                    }

                    impl [< $blueprint_name Function >] {
                        #[allow(unreachable_patterns)]
                        pub fn function_name(&self) -> &str {
                            match self {
                                $(
                                    Self::$func_ident(..) => $func_name,
                                )*
                                _ => unreachable!()
                            }
                        }

                        pub fn from_invocation(
                            function_name: &str,
                            args: &::radix_common::prelude::ManifestValue
                        ) -> Option<Self> {
                            match function_name {
                                $(
                                    $func_name => ::radix_common::prelude::manifest_encode(args)
                                        .ok()
                                        .and_then(|value| ::radix_common::prelude::manifest_decode(&value).ok())
                                        .map(Self::$func_ident),
                                )*
                                _ => None
                            }
                        }
                    }
                )*
            )*
        }
    };
}

    define_typed_invocations! {
        AccessController => {
            AccessController => {
                type: ComponentAddress,
                entity_type_pat: EntityType::GlobalAccessController,
                module_id: ModuleId::Main,
                functions: {
                    Create => (
                        access_controller::AccessControllerCreateManifestInput,
                        access_controller::ACCESS_CONTROLLER_CREATE_IDENT
                    )
                },
                methods: {
                    CreateProof => (
                        access_controller::AccessControllerCreateProofInput,
                        access_controller::ACCESS_CONTROLLER_CREATE_PROOF_IDENT
                    ),
                    InitiateRecoveryAsPrimary => (
                        access_controller::AccessControllerInitiateRecoveryAsPrimaryInput,
                        access_controller::ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT
                    ),
                    InitiateRecoveryAsRecovery => (
                        access_controller::AccessControllerInitiateRecoveryAsRecoveryInput,
                        access_controller::ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT
                    ),
                    QuickConfirmPrimaryRoleRecoveryProposal => (
                        access_controller::AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput,
                        access_controller::ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT
                    ),
                    QuickConfirmRecoveryRoleRecoveryProposal => (
                        access_controller::AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput,
                        access_controller::ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT
                    ),
                    TimedConfirmRecovery => (
                        access_controller::AccessControllerTimedConfirmRecoveryInput,
                        access_controller::ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT
                    ),
                    StopTimedRecovery => (
                        access_controller::AccessControllerStopTimedRecoveryInput,
                        access_controller::ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT
                    ),
                    MintRecoveryBadges => (
                        access_controller::AccessControllerMintRecoveryBadgesInput,
                        access_controller::ACCESS_CONTROLLER_MINT_RECOVERY_BADGES_IDENT
                    ),
                    LockRecoveryFee => (
                        access_controller::AccessControllerLockRecoveryFeeInput,
                        access_controller::ACCESS_CONTROLLER_LOCK_RECOVERY_FEE_IDENT
                    ),
                    WithdrawRecoveryFee => (
                        access_controller::AccessControllerWithdrawRecoveryFeeInput,
                        access_controller::ACCESS_CONTROLLER_WITHDRAW_RECOVERY_FEE_IDENT
                    ),
                    ContributeRecoveryFee => (
                        access_controller::AccessControllerContributeRecoveryFeeManifestInput,
                        access_controller::ACCESS_CONTROLLER_CONTRIBUTE_RECOVERY_FEE_IDENT
                    ),
                    InitiateBadgeWithdrawAttemptAsPrimary => (
                        access_controller::AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput,
                        access_controller::ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_PRIMARY_IDENT
                    ),
                    InitiateBadgeWithdrawAttemptAsRecovery => (
                        access_controller::AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput,
                        access_controller::ACCESS_CONTROLLER_INITIATE_BADGE_WITHDRAW_ATTEMPT_AS_RECOVERY_IDENT
                    ),
                    QuickConfirmPrimaryRoleBadgeWithdrawAttempt => (
                        access_controller::AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput,
                        access_controller::ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
                    ),
                    QuickConfirmRecoveryRoleBadgeWithdrawAttempt => (
                        access_controller::AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput,
                        access_controller::ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
                    ),
                    CancelPrimaryRoleRecoveryProposal => (
                        access_controller::AccessControllerCancelPrimaryRoleRecoveryProposalInput,
                        access_controller::ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT
                    ),
                    CancelRecoveryRoleRecoveryProposal => (
                        access_controller::AccessControllerCancelRecoveryRoleRecoveryProposalInput,
                        access_controller::ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT
                    ),
                    CancelPrimaryRoleBadgeWithdrawAttempt => (
                        access_controller::AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput,
                        access_controller::ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
                    ),
                    CancelRecoveryRoleBadgeWithdrawAttempt => (
                        access_controller::AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput,
                        access_controller::ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_BADGE_WITHDRAW_ATTEMPT_IDENT
                    ),
                    LockPrimaryRole => (
                        access_controller::AccessControllerLockPrimaryRoleInput,
                        access_controller::ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT
                    ),
                    UnlockPrimaryRole => (
                        access_controller::AccessControllerUnlockPrimaryRoleInput,
                        access_controller::ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT
                    ),
                }
            }
        },
        Account => {
            Account => {
                type: ComponentAddress,
                entity_type_pat:
                    EntityType::GlobalAccount
                    | EntityType::GlobalPreallocatedEd25519Account
                    | EntityType::GlobalPreallocatedSecp256k1Account,
                module_id: ModuleId::Main,
                functions: {
                    Create => (
                        account::AccountCreateInput,
                        account::ACCOUNT_CREATE_IDENT
                    ),
                    CreateAdvanced => (
                        account::AccountCreateAdvancedManifestInput,
                        account::ACCOUNT_CREATE_ADVANCED_IDENT
                    ),
                },
                methods: {
                    Securify => (
                        account::AccountSecurifyInput,
                        account::ACCOUNT_SECURIFY_IDENT
                    ),
                    LockFee => (
                        account::AccountLockFeeInput,
                        account::ACCOUNT_LOCK_FEE_IDENT
                    ),
                    LockContingentFee => (
                        account::AccountLockContingentFeeInput,
                        account::ACCOUNT_LOCK_CONTINGENT_FEE_IDENT
                    ),
                    Deposit => (
                        account::AccountDepositManifestInput,
                        account::ACCOUNT_DEPOSIT_IDENT
                    ),
                    DepositBatch => (
                        ManifestValue,
                        account::ACCOUNT_DEPOSIT_BATCH_IDENT
                    ),
                    Withdraw => (
                        account::AccountWithdrawInput,
                        account::ACCOUNT_WITHDRAW_IDENT
                    ),
                    WithdrawNonFungibles => (
                        account::AccountWithdrawNonFungiblesInput,
                        account::ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT
                    ),
                    LockFeeAndWithdraw => (
                        account::AccountLockFeeAndWithdrawInput,
                        account::ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT
                    ),
                    LockFeeAndWithdrawNonFungibles => (
                        account::AccountLockFeeAndWithdrawNonFungiblesInput,
                        account::ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT
                    ),
                    CreateProofOfAmount => (
                        account::AccountCreateProofOfAmountInput,
                        account::ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT
                    ),
                    CreateProofOfNonFungibles => (
                        account::AccountCreateProofOfNonFungiblesInput,
                        account::ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT
                    ),
                    SetDefaultDepositRule => (
                        account::AccountSetDefaultDepositRuleInput,
                        account::ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT
                    ),
                    SetResourcePreference => (
                        account::AccountSetResourcePreferenceInput,
                        account::ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT
                    ),
                    RemoveResourcePreference => (
                        account::AccountRemoveResourcePreferenceInput,
                        account::ACCOUNT_REMOVE_RESOURCE_PREFERENCE_IDENT
                    ),
                    TryDepositOrRefund => (
                        account::AccountTryDepositOrRefundManifestInput,
                        account::ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT
                    ),
                    TryDepositBatchOrRefund => (
                        ManifestValue,
                        account::ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT
                    ),
                    TryDepositOrAbort => (
                        account::AccountTryDepositOrAbortManifestInput,
                        account::ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT
                    ),
                    TryDepositBatchOrAbort => (
                        ManifestValue,
                        account::ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT
                    ),
                    Burn => (
                        account::AccountBurnInput,
                        account::ACCOUNT_BURN_IDENT
                    ),
                    BurnNonFungibles => (
                        account::AccountBurnNonFungiblesInput,
                        account::ACCOUNT_BURN_NON_FUNGIBLES_IDENT
                    ),
                    AddAuthorizedDepositor => (
                        account::AccountAddAuthorizedDepositorInput,
                        account::ACCOUNT_ADD_AUTHORIZED_DEPOSITOR
                    ),
                    RemoveAuthorizedDepositor => (
                        account::AccountRemoveAuthorizedDepositorInput,
                        account::ACCOUNT_REMOVE_AUTHORIZED_DEPOSITOR
                    )
                },
            },
        },
        ConsensusManager => {
            Validator => {
                type: ComponentAddress,
                entity_type_pat: EntityType::GlobalValidator,
                module_id: ModuleId::Main,
                functions: {},
                methods: {
                    Register => (
                        consensus_manager::ValidatorRegisterInput,
                        consensus_manager::VALIDATOR_REGISTER_IDENT,
                    ),
                    Unregister => (
                        consensus_manager::ValidatorUnregisterInput,
                        consensus_manager::VALIDATOR_UNREGISTER_IDENT,
                    ),
                    StakeAsOwner => (
                        consensus_manager::ValidatorStakeAsOwnerManifestInput,
                        consensus_manager::VALIDATOR_STAKE_AS_OWNER_IDENT,
                    ),
                    Stake => (
                        consensus_manager::ValidatorStakeManifestInput,
                        consensus_manager::VALIDATOR_STAKE_IDENT,
                    ),
                    Unstake => (
                        consensus_manager::ValidatorUnstakeManifestInput,
                        consensus_manager::VALIDATOR_UNSTAKE_IDENT,
                    ),
                    ClaimXrd => (
                        consensus_manager::ValidatorClaimXrdManifestInput,
                        consensus_manager::VALIDATOR_CLAIM_XRD_IDENT,
                    ),
                    UpdateKey => (
                        consensus_manager::ValidatorUpdateKeyInput,
                        consensus_manager::VALIDATOR_UPDATE_KEY_IDENT,
                    ),
                    UpdateFee => (
                        consensus_manager::ValidatorUpdateFeeInput,
                        consensus_manager::VALIDATOR_UPDATE_FEE_IDENT,
                    ),
                    UpdateAcceptDelegatedStake => (
                        consensus_manager::ValidatorUpdateAcceptDelegatedStakeInput,
                        consensus_manager::VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT,
                    ),
                    AcceptsDelegatedStake => (
                        consensus_manager::ValidatorAcceptsDelegatedStakeInput,
                        consensus_manager::VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT,
                    ),
                    TotalStakeXrdAmount => (
                        consensus_manager::ValidatorTotalStakeXrdAmountInput,
                        consensus_manager::VALIDATOR_TOTAL_STAKE_XRD_AMOUNT_IDENT,
                    ),
                    TotalStakeUnitSupply => (
                        consensus_manager::ValidatorTotalStakeUnitSupplyInput,
                        consensus_manager::VALIDATOR_TOTAL_STAKE_UNIT_SUPPLY_IDENT,
                    ),
                    GetRedemptionValue => (
                        consensus_manager::ValidatorGetRedemptionValueInput,
                        consensus_manager::VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
                    ),
                    SignalProtocolUpdateReadiness => (
                        consensus_manager::ValidatorSignalProtocolUpdateReadinessInput,
                        consensus_manager::VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS,
                    ),
                    GetProtocolUpdateReadiness => (
                        consensus_manager::ValidatorGetProtocolUpdateReadinessInput,
                        consensus_manager::VALIDATOR_GET_PROTOCOL_UPDATE_READINESS_IDENT,
                    ),
                    LockOwnerStakeUnits => (
                        consensus_manager::ValidatorLockOwnerStakeUnitsManifestInput,
                        consensus_manager::VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT,
                    ),
                    StartUnlockOwnerStakeUnits => (
                        consensus_manager::ValidatorStartUnlockOwnerStakeUnitsInput,
                        consensus_manager::VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT,
                    ),
                    FinishUnlockOwnerStakeUnits => (
                        consensus_manager::ValidatorFinishUnlockOwnerStakeUnitsInput,
                        consensus_manager::VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT,
                    ),
                }
            },
            ConsensusManager => {
                type: ComponentAddress,
                entity_type_pat: EntityType::GlobalConsensusManager,
                module_id: ModuleId::Main,
                functions: {
                    Create => (
                        consensus_manager::ConsensusManagerCreateManifestInput,
                        consensus_manager::CONSENSUS_MANAGER_CREATE_IDENT,
                    ),
                },
                methods: {
                    GetCurrentEpoch => (
                        consensus_manager::ConsensusManagerGetCurrentEpochInput,
                        consensus_manager::CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
                    ),
                    Start => (
                        consensus_manager::ConsensusManagerStartInput,
                        consensus_manager::CONSENSUS_MANAGER_START_IDENT,
                    ),
                    GetCurrentTime => (
                        consensus_manager::ConsensusManagerGetCurrentTimeInputV2,
                        consensus_manager::CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT,
                    ),
                    NextRound => (
                        consensus_manager::ConsensusManagerNextRoundInput,
                        consensus_manager::CONSENSUS_MANAGER_NEXT_ROUND_IDENT,
                    ),
                    CreateValidator => (
                        consensus_manager::ConsensusManagerCreateValidatorManifestInput,
                        consensus_manager::CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT,
                    ),
                }
            }
        },
        Identity => {
            Identity => {
                type: ComponentAddress,
                entity_type_pat:
                    EntityType::GlobalIdentity
                    | EntityType::GlobalPreallocatedEd25519Identity
                    | EntityType::GlobalPreallocatedSecp256k1Identity,
                module_id: ModuleId::Main,
                functions: {
                    Create => (
                        identity::IdentityCreateInput,
                        identity::IDENTITY_CREATE_IDENT
                    ),
                    CreateAdvanced => (
                        identity::IdentityCreateAdvancedInput,
                        identity::IDENTITY_CREATE_ADVANCED_IDENT
                    ),
                },
                methods: {
                    Securify => (
                        identity::IdentitySecurifyToSingleBadgeInput,
                        identity::IDENTITY_SECURIFY_IDENT
                    )
                },
            },
        },
        Locker => {
            AccountLocker => {
                type: ComponentAddress,
                entity_type_pat: EntityType::GlobalAccountLocker,
                module_id: ModuleId::Main,
                functions: {
                    Instantiate => (
                        locker::AccountLockerInstantiateManifestInput,
                        locker::ACCOUNT_LOCKER_INSTANTIATE_IDENT,
                    ),
                    InstantiateSimple => (
                        locker::AccountLockerInstantiateSimpleManifestInput,
                        locker::ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                    ),
                },
                methods: {
                    Store => (
                        locker::AccountLockerStoreManifestInput,
                        locker::ACCOUNT_LOCKER_STORE_IDENT,
                    ),
                    Airdrop => (
                        locker::AccountLockerAirdropManifestInput,
                        locker::ACCOUNT_LOCKER_AIRDROP_IDENT,
                    ),
                    Recover => (
                        locker::AccountLockerRecoverManifestInput,
                        locker::ACCOUNT_LOCKER_RECOVER_IDENT,
                    ),
                    RecoverNonFungibles => (
                        locker::AccountLockerRecoverNonFungiblesManifestInput,
                        locker::ACCOUNT_LOCKER_RECOVER_NON_FUNGIBLES_IDENT,
                    ),
                    Claim => (
                        locker::AccountLockerClaimManifestInput,
                        locker::ACCOUNT_LOCKER_CLAIM_IDENT,
                    ),
                    ClaimNonFungibles => (
                        locker::AccountLockerClaimNonFungiblesManifestInput,
                        locker::ACCOUNT_LOCKER_CLAIM_NON_FUNGIBLES_IDENT,
                    ),
                    GetAmount => (
                        locker::AccountLockerGetAmountManifestInput,
                        locker::ACCOUNT_LOCKER_GET_AMOUNT_IDENT,
                    ),
                    GetNonFungibleLocalIds => (
                        locker::AccountLockerGetNonFungibleLocalIdsManifestInput,
                        locker::ACCOUNT_LOCKER_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                    ),
                },
            }
        }
    }
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
