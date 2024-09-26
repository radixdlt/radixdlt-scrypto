use crate::manifest::*;
use crate::prelude::*;
use radix_common::prelude::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::prelude::*;

pub struct StaticResourceMovementsOutput {
    pub invocation_static_information: IndexMap<usize, InvocationStaticInformation>,
}

impl StaticResourceMovementsOutput {
    pub fn account_withdraws(&self) -> IndexMap<ComponentAddress, Vec<AccountWithdraw>> {
        self.invocation_static_information.values().fold(
            Default::default(),
            |mut acc, invocation| {
                let InvocationStaticInformation {
                    kind:
                        OwnedInvocationKind::Method {
                            address: DynamicGlobalAddress::Static(account_address),
                            module_id: ModuleId::Main,
                            method,
                        },
                    output,
                    ..
                } = invocation
                else {
                    return acc;
                };

                // Convert the global address to a component address
                let Ok(account_address) = ComponentAddress::try_from(account_address.as_bytes())
                else {
                    return acc;
                };

                // Check if this a deposit event.
                if matches!(
                    method.as_str(),
                    ACCOUNT_WITHDRAW_IDENT
                        | ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT
                        | ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT
                        | ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT
                ) {
                    acc.entry(account_address)
                        .or_default()
                        .extend(output.iter().filter_map(|invocation_io| {
                            if matches!(
                                method.as_str(),
                                ACCOUNT_WITHDRAW_IDENT | ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT
                            ) {
                                match invocation_io {
                                    InvocationIo::KnownFungible(
                                        FungibleResourceAddress(resource_address),
                                        FungibleBounds {
                                            lower: LowerFungibleBound::Amount(lower),
                                            upper: UpperFungibleBound::Amount(upper),
                                        },
                                    ) if lower == upper => {
                                        Some(AccountWithdraw::Amount(*resource_address, *lower))
                                    }
                                    InvocationIo::KnownNonFungible(
                                        NonFungibleResourceAddress(resource_address),
                                        NonFungibleBounds {
                                            amount_bounds:
                                                FungibleBounds {
                                                    lower: LowerFungibleBound::Amount(lower),
                                                    upper: UpperFungibleBound::Amount(upper),
                                                },
                                            ..
                                        },
                                    ) if lower == upper => {
                                        Some(AccountWithdraw::Amount(*resource_address, *lower))
                                    }
                                    _ => None,
                                }
                            } else if let InvocationIo::KnownNonFungible(
                                NonFungibleResourceAddress(resource_address),
                                NonFungibleBounds {
                                    id_bounds: NonFungibleIdBounds::FullyKnown(ids),
                                    ..
                                },
                            ) = invocation_io
                            {
                                Some(AccountWithdraw::Ids(*resource_address, ids.clone()))
                            } else {
                                None
                            }
                        }));
                    acc
                } else {
                    acc
                }
            },
        )
    }

    pub fn account_deposits(&self) -> IndexMap<ComponentAddress, Vec<AccountDeposit>> {
        self.invocation_static_information.values().fold(
            Default::default(),
            |mut acc, invocation| {
                let InvocationStaticInformation {
                    kind:
                        OwnedInvocationKind::Method {
                            address: DynamicGlobalAddress::Static(account_address),
                            module_id: ModuleId::Main,
                            method,
                        },
                    input,
                    ..
                } = invocation
                else {
                    return acc;
                };

                // Convert the global address to a component address
                let Ok(account_address) = ComponentAddress::try_from(account_address.as_bytes())
                else {
                    return acc;
                };

                // Check if this a deposit event.
                if matches!(
                    method.as_str(),
                    ACCOUNT_DEPOSIT_IDENT
                        | ACCOUNT_DEPOSIT_BATCH_IDENT
                        | ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT
                        | ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT
                        | ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT
                        | ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT
                ) {
                    acc.entry(account_address)
                        .or_default()
                        .extend(input.iter().cloned().map(AccountDeposit::from));
                    acc
                } else {
                    acc
                }
            },
        )
    }
}

#[derive(Clone, Debug)]
pub struct InvocationStaticInformation {
    pub kind: OwnedInvocationKind,
    pub input: Vec<InvocationIo>,
    pub output: Vec<InvocationIo>,
}

#[derive(Clone, Debug)]
pub enum OwnedInvocationKind {
    Method {
        address: DynamicGlobalAddress,
        module_id: ModuleId,
        method: String,
    },
    Function {
        address: DynamicPackageAddress,
        blueprint: String,
        function: String,
    },
    DirectMethod {
        address: InternalAddress,
        method: String,
    },
    YieldToParent,
    YieldToChild {
        child_index: ManifestNamedIntent,
    },
}

impl<'a> From<InvocationKind<'a>> for OwnedInvocationKind {
    fn from(value: InvocationKind<'a>) -> Self {
        match value {
            InvocationKind::Method {
                address,
                module_id,
                method,
            } => Self::Method {
                address: *address,
                module_id,
                method: method.to_owned(),
            },
            InvocationKind::Function {
                address,
                blueprint,
                function,
            } => Self::Function {
                address: *address,
                blueprint: blueprint.to_owned(),
                function: function.to_owned(),
            },
            InvocationKind::DirectMethod { address, method } => Self::DirectMethod {
                address: *address,
                method: method.to_owned(),
            },
            InvocationKind::YieldToParent => Self::YieldToParent,
            InvocationKind::YieldToChild { child_index } => Self::YieldToChild { child_index },
        }
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
pub(super) enum BucketContent {
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

    pub(super) fn decrease_both_bounds(&mut self, by: Decimal) -> Option<()> {
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
    pub fn new_with_amount(amount: Decimal) -> Self {
        Self {
            amount_bounds: FungibleBounds::new_exact(amount),
            id_bounds: NonFungibleIdBounds::Unknown,
        }
    }

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
                let ids = std::mem::replace(ids1, index_set_new());
                self.id_bounds = NonFungibleIdBounds::PartiallyKnown(ids);
            }
            (NonFungibleIdBounds::FullyKnown(ref mut ids), NonFungibleIdBounds::Unknown) => {
                let ids = std::mem::replace(ids, index_set_new());
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
pub enum InvocationIo {
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
pub struct FungibleResourceAddress(pub(super) ResourceAddress);

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
pub struct NonFungibleResourceAddress(pub(super) ResourceAddress);

impl NonFungibleResourceAddress {
    pub fn new(address: ResourceAddress) -> Option<Self> {
        if !address.is_fungible() {
            Some(Self(address))
        } else {
            None
        }
    }
}
