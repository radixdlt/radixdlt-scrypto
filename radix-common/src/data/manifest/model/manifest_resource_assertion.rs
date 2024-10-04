use crate::internal_prelude::*;

// This file isn't part of the Manifest SBOR value model, but is included here
// for consolidation of the manifest types

#[derive(Debug, Clone, PartialEq, Eq, Default, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct ManifestResourceConstraints {
    specified_resources: IndexMap<ResourceAddress, ManifestResourceConstraint>,
}

impl ManifestResourceConstraints {
    pub fn specified_resources(&self) -> &IndexMap<ResourceAddress, ManifestResourceConstraint> {
        &self.specified_resources
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ResourceAddress, &ManifestResourceConstraint)> {
        self.specified_resources.iter()
    }

    pub fn is_valid(&self) -> bool {
        for (resource_address, constraint) in self.iter() {
            if !constraint.is_valid_for(resource_address) {
                return false;
            }
        }
        true
    }

    pub fn contains_specified_resource(&self, resource_address: &ResourceAddress) -> bool {
        self.specified_resources.contains_key(resource_address)
    }
}

impl IntoIterator for ManifestResourceConstraints {
    type Item = (ResourceAddress, ManifestResourceConstraint);
    type IntoIter =
        <IndexMap<ResourceAddress, ManifestResourceConstraint> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.specified_resources.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub enum ManifestResourceConstraint {
    NonZeroAmount,
    ExactAmount(Decimal),
    AtLeastAmount(Decimal),
    ExactNonFungibles(IndexSet<NonFungibleLocalId>),
    AtLeastNonFungibles(IndexSet<NonFungibleLocalId>),
    General(GeneralResourceConstraint),
}

impl ManifestResourceConstraint {
    pub fn is_valid_for(&self, resource_address: &ResourceAddress) -> bool {
        if resource_address.is_fungible() {
            self.is_valid_for_fungible_use()
        } else {
            self.is_valid_for_non_fungible_use()
        }
    }

    pub fn is_valid_for_fungible_use(&self) -> bool {
        match self {
            ManifestResourceConstraint::NonZeroAmount => true,
            ManifestResourceConstraint::ExactAmount(amount) => !amount.is_negative(),
            ManifestResourceConstraint::AtLeastAmount(amount) => !amount.is_negative(),
            ManifestResourceConstraint::ExactNonFungibles(_) => false,
            ManifestResourceConstraint::AtLeastNonFungibles(_) => false,
            ManifestResourceConstraint::General(general) => general.is_valid_for_fungible_use(),
        }
    }

    pub fn is_valid_for_non_fungible_use(&self) -> bool {
        match self {
            ManifestResourceConstraint::NonZeroAmount => true,
            ManifestResourceConstraint::ExactAmount(amount) => !amount.is_negative(),
            ManifestResourceConstraint::AtLeastAmount(amount) => !amount.is_negative(),
            ManifestResourceConstraint::ExactNonFungibles(_) => true,
            ManifestResourceConstraint::AtLeastNonFungibles(_) => true,
            ManifestResourceConstraint::General(general) => general.is_valid_for_non_fungible_use(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct GeneralResourceConstraint {
    pub required_ids: IndexSet<NonFungibleLocalId>,
    pub lower_bound: LowerBound,
    pub upper_bound: UpperBound,
    pub allowed_ids: AllowedIds,
}

impl GeneralResourceConstraint {
    pub fn is_valid_for_fungible_use(&self) -> bool {
        return self.required_ids.is_empty()
            && self.allowed_ids.is_valid_for_fungible_use()
            && self.are_bounds_valid();
    }

    pub fn is_valid_for_non_fungible_use(&self) -> bool {
        return self.are_bounds_valid();
    }

    fn are_bounds_valid(&self) -> bool {
        let required_ids_amount = Decimal::from(self.required_ids.len());
        // These inequalities also validate that the lower and upper bounds are non-negative.
        if required_ids_amount > self.lower_bound.equivalent_decimal() {
            return false;
        }
        if self.lower_bound.equivalent_decimal() > self.upper_bound.equivalent_decimal() {
            return false;
        }
        match &self.allowed_ids {
            AllowedIds::Allowlist(allowlist) => {
                let allowlist_ids_amount = Decimal::from(allowlist.len());
                if self.upper_bound.equivalent_decimal() > allowlist_ids_amount {
                    return false;
                }
                if !self.required_ids.is_subset(allowlist) {
                    return false;
                }
            }
            AllowedIds::Any => {}
        }
        true
    }
}

/// Represents a lower bound on a non-negative decimal.
///
/// ## Invariants
/// * The `amount` in `LowerBound::Inclusive(amount)` is required to be non-negative before using
///   this model. This can be validated via [`ManifestResourceConstraint::is_valid_for`].
///
/// ## Trait Implementations
/// * [`Ord`], [`PartialOrd`] - Satisfies `AmountInclusive(Zero) < NonZero < AmountInclusive(AnyPositive)`
#[derive(Debug, Clone, Copy, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub enum LowerBound {
    NonZero,
    Inclusive(Decimal),
}

impl PartialOrd for LowerBound {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LowerBound {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match (self, other) {
            (
                LowerBound::Inclusive(self_lower_inclusive),
                LowerBound::Inclusive(other_lower_inclusive),
            ) => self_lower_inclusive.cmp(other_lower_inclusive),
            (LowerBound::Inclusive(self_lower_inclusive), LowerBound::NonZero) => {
                if self_lower_inclusive.is_positive() {
                    core::cmp::Ordering::Greater
                } else {
                    core::cmp::Ordering::Less
                }
            }
            (LowerBound::NonZero, LowerBound::Inclusive(other_lower_inclusive)) => {
                if other_lower_inclusive.is_positive() {
                    core::cmp::Ordering::Less
                } else {
                    core::cmp::Ordering::Greater
                }
            }
            (LowerBound::NonZero, LowerBound::NonZero) => core::cmp::Ordering::Equal,
        }
    }
}

impl LowerBound {
    pub const fn zero() -> Self {
        Self::Inclusive(Decimal::ZERO)
    }

    pub const fn non_zero() -> Self {
        Self::NonZero
    }

    pub fn cmp_upper(&self, other: &UpperBound) -> core::cmp::Ordering {
        match (self, other) {
            (
                LowerBound::Inclusive(lower_bound_inclusive),
                UpperBound::Inclusive(upper_bound_inclusive),
            ) => lower_bound_inclusive.cmp(upper_bound_inclusive),
            (_, UpperBound::Unbounded) => core::cmp::Ordering::Less,
            (LowerBound::NonZero, UpperBound::Inclusive(upper_bound_inclusive)) => {
                if upper_bound_inclusive.is_zero() {
                    core::cmp::Ordering::Greater
                } else {
                    core::cmp::Ordering::Less
                }
            }
        }
    }

    /// ## Panics
    /// * Panics if the decimal is not resolvable or is non-negative
    pub fn at_least(decimal: Decimal) -> Self {
        if decimal.is_negative() {
            panic!("An at_least bound is negative");
        }
        Self::Inclusive(decimal)
    }

    pub fn is_zero(&self) -> bool {
        self.eq(&Self::zero())
    }

    pub fn is_positive(&self) -> bool {
        !self.is_zero()
    }

    pub fn add_from(&mut self, other: Self) -> Result<(), BoundAdjustmentError> {
        let new_bound = match (*self, other) {
            (LowerBound::Inclusive(self_lower_bound), LowerBound::Inclusive(other_lower_bound)) => {
                let lower_bound_inclusive = self_lower_bound
                    .checked_add(other_lower_bound)
                    .ok_or(BoundAdjustmentError::DecimalOverflow)?;
                LowerBound::Inclusive(lower_bound_inclusive)
            }
            (LowerBound::Inclusive(amount), LowerBound::NonZero)
            | (LowerBound::NonZero, LowerBound::Inclusive(amount)) => {
                if amount.is_zero() {
                    LowerBound::NonZero
                } else {
                    LowerBound::Inclusive(amount)
                }
            }
            (LowerBound::NonZero, LowerBound::NonZero) => LowerBound::NonZero,
        };

        *self = new_bound;
        Ok(())
    }

    /// PRECONDITION: take_amount must be positive
    pub fn take_amount(&mut self, take_amount: Decimal) {
        let new_bound = match *self {
            LowerBound::Inclusive(lower_bound_inclusive) => {
                if take_amount > lower_bound_inclusive {
                    Self::zero()
                } else {
                    LowerBound::Inclusive(lower_bound_inclusive - take_amount)
                }
            }
            LowerBound::NonZero => {
                if take_amount.is_zero() {
                    LowerBound::NonZero
                } else {
                    Self::zero()
                }
            }
        };

        *self = new_bound;
    }

    pub fn constrain_to(&mut self, other_bound: LowerBound) {
        let new_bound = (*self).max(other_bound);
        *self = new_bound;
    }

    pub fn equivalent_decimal(&self) -> Decimal {
        match self {
            LowerBound::Inclusive(decimal) => *decimal,
            LowerBound::NonZero => Decimal::from_attos(I192::ONE),
        }
    }

    pub fn is_satisfied_by(&self, amount: Decimal) -> bool {
        match self {
            LowerBound::NonZero => amount.is_positive(),
            LowerBound::Inclusive(inclusive_lower_bound) => *inclusive_lower_bound <= amount,
        }
    }
}

/// Represents an upper bound on a non-negative decimal.
///
/// ## Invariants
/// * The `amount` in `LowerBound::Inclusive(amount)` is required to be non-negative before using
///   this model. This can be validated via [`ManifestResourceConstraint::is_valid_for`].
///
/// ## Trait Implementations
/// * [`Ord`], [`PartialOrd`] - Satisfies `AmountInclusive(Any) < Unbounded`
#[derive(Debug, Clone, Copy, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub enum UpperBound {
    Inclusive(Decimal),
    Unbounded,
}

impl PartialOrd for UpperBound {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UpperBound {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match (self, other) {
            (
                UpperBound::Inclusive(upper_bound_inclusive),
                UpperBound::Inclusive(other_upper_bound_inclusive),
            ) => upper_bound_inclusive.cmp(other_upper_bound_inclusive),
            (UpperBound::Inclusive(_), UpperBound::Unbounded) => core::cmp::Ordering::Less,
            (UpperBound::Unbounded, UpperBound::Inclusive(_)) => core::cmp::Ordering::Greater,
            (UpperBound::Unbounded, UpperBound::Unbounded) => core::cmp::Ordering::Equal,
        }
    }
}

impl UpperBound {
    pub const fn unbounded() -> Self {
        Self::Unbounded
    }

    pub const fn zero() -> Self {
        Self::Inclusive(Decimal::ZERO)
    }

    /// ## Panics
    /// * Panics if the decimal is not resolvable or is non-negative
    pub fn at_most(decimal: Decimal) -> Self {
        if decimal.is_negative() {
            panic!("An at_most bound is negative");
        }
        Self::Inclusive(decimal)
    }

    pub fn add_from(&mut self, other: Self) -> Result<(), BoundAdjustmentError> {
        let new_bound = match (*self, other) {
            (
                UpperBound::Inclusive(self_upper_bound_inclusive),
                UpperBound::Inclusive(other_upper_bound_inclusive),
            ) => {
                let upper_bound_inclusive = self_upper_bound_inclusive
                    .checked_add(other_upper_bound_inclusive)
                    .ok_or(BoundAdjustmentError::DecimalOverflow)?;
                UpperBound::Inclusive(upper_bound_inclusive)
            }
            (_, UpperBound::Unbounded) | (UpperBound::Unbounded, _) => UpperBound::Unbounded,
        };

        *self = new_bound;
        Ok(())
    }

    /// PRECONDITION: take_amount must be positive
    pub fn take_amount(&mut self, take_amount: Decimal) -> Result<(), BoundAdjustmentError> {
        let new_bound = match *self {
            UpperBound::Inclusive(upper_bound_inclusive) => {
                if take_amount > upper_bound_inclusive {
                    return Err(BoundAdjustmentError::TakeCannotBeSatisfied);
                }
                UpperBound::Inclusive(upper_bound_inclusive - take_amount)
            }
            UpperBound::Unbounded => UpperBound::Unbounded,
        };

        *self = new_bound;

        Ok(())
    }

    pub fn constrain_to(&mut self, other_bound: UpperBound) {
        let new_bound = (*self).min(other_bound);
        *self = new_bound;
    }

    pub fn equivalent_decimal(&self) -> Decimal {
        match self {
            UpperBound::Inclusive(decimal) => *decimal,
            UpperBound::Unbounded => Decimal::MAX,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub enum AllowedIds {
    Allowlist(IndexSet<NonFungibleLocalId>),
    Any,
}

impl AllowedIds {
    pub fn none() -> Self {
        Self::Allowlist(Default::default())
    }

    pub fn allowlist_equivalent_length(&self) -> usize {
        match self {
            Self::Allowlist(allowlist) => allowlist.len(),
            Self::Any => usize::MAX,
        }
    }

    pub fn is_valid_for_fungible_use(&self) -> bool {
        match self {
            AllowedIds::Allowlist(allowlist) => allowlist.is_empty(),
            AllowedIds::Any => true,
        }
    }
}

pub enum BoundAdjustmentError {
    DecimalOverflow,
    TakeCannotBeSatisfied,
}
