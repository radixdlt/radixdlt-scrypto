use crate::internal_prelude::*;

// This file isn't part of the Manifest SBOR value model, but is included here
// for consolidation of the manifest types

#[derive(Debug, Clone, PartialEq, Eq, Default, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct ManifestResourceConstraints {
    specified_resources: IndexMap<ResourceAddress, ManifestResourceConstraint>,
}

impl ManifestResourceConstraints {
    pub fn new() -> Self {
        Default::default()
    }

    /// ## Panics
    /// * Panics if the constraint isn't valid for the resource address
    /// * Panics if constraints have already been specified against the resource
    pub fn with(
        mut self,
        resource_address: ResourceAddress,
        constraint: ManifestResourceConstraint,
    ) -> Self {
        if !constraint.is_valid_for(&resource_address) {
            panic!("Constraint isn't valid for the resource address");
        }
        let replaced = self
            .specified_resources
            .insert(resource_address, constraint);
        if replaced.is_some() {
            panic!("A constraint has already been specified against the resource");
        }
        self
    }

    /// ## Panics
    /// * Panics if the constraint isn't valid for the resource address
    /// * Panics if constraints have already been specified against the resource
    pub fn with_exact_amount(
        self,
        resource_address: ResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        self.with(
            resource_address,
            ManifestResourceConstraint::ExactAmount(amount.resolve()),
        )
    }

    /// ## Panics
    /// * Panics if the constraint isn't valid for the resource address
    /// * Panics if constraints have already been specified against the resource
    pub fn with_at_least_amount(
        self,
        resource_address: ResourceAddress,
        amount: impl ResolvableDecimal,
    ) -> Self {
        self.with(
            resource_address,
            ManifestResourceConstraint::AtLeastAmount(amount.resolve()),
        )
    }

    /// ## Panics
    /// * Panics if the constraint isn't valid for the resource address
    /// * Panics if constraints have already been specified against the resource
    pub fn with_amount_range(
        self,
        resource_address: ResourceAddress,
        lower_bound: impl ResolvableLowerBound,
        upper_bound: impl ResolvableUpperBound,
    ) -> Self {
        self.with_general_constraint(
            resource_address,
            GeneralResourceConstraint {
                required_ids: Default::default(),
                lower_bound: lower_bound.resolve(),
                upper_bound: upper_bound.resolve(),
                allowed_ids: AllowedIds::Any,
            },
        )
    }

    /// ## Panics
    /// * Panics if the constraint isn't valid for the resource address
    /// * Panics if constraints have already been specified against the resource
    pub fn with_exact_non_fungibles(
        self,
        resource_address: ResourceAddress,
        non_fungible_ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Self {
        self.with(
            resource_address,
            ManifestResourceConstraint::ExactNonFungibles(non_fungible_ids.into_iter().collect()),
        )
    }

    /// ## Panics
    /// * Panics if the constraint isn't valid for the resource address
    /// * Panics if constraints have already been specified against the resource
    pub fn with_at_least_non_fungibles(
        self,
        resource_address: ResourceAddress,
        non_fungible_ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Self {
        self.with(
            resource_address,
            ManifestResourceConstraint::AtLeastNonFungibles(non_fungible_ids.into_iter().collect()),
        )
    }

    /// ## Panics
    /// * Panics if the constraint isn't valid for the resource address
    /// * Panics if constraints have already been specified against the resource
    pub fn with_general_constraint(
        self,
        resource_address: ResourceAddress,
        bounds: GeneralResourceConstraint,
    ) -> Self {
        self.with(
            resource_address,
            ManifestResourceConstraint::General(bounds),
        )
    }

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
            ManifestResourceConstraint::ExactAmount(amount) => {
                !amount.is_negative() && amount.checked_floor() == Some(*amount)
            }
            ManifestResourceConstraint::AtLeastAmount(amount) => {
                !amount.is_negative() && amount.checked_floor() == Some(*amount)
            }
            ManifestResourceConstraint::ExactNonFungibles(_) => true,
            ManifestResourceConstraint::AtLeastNonFungibles(_) => true,
            ManifestResourceConstraint::General(general) => general.is_valid_for_non_fungible_use(),
        }
    }
}

/// [`GeneralResourceConstraint`] captures constraints on the balance of a single fungible
/// or non-fungible resource.
///
/// It captures four concepts:
///
/// * A set of [`required_ids`][Self::required_ids] which are [`NonFungibleLocalId`]s which are
///   required to be in the balance.
/// * A [`lower_bound`][Self::lower_bound] on the decimal balance amount.
/// * An [`upper_bound`][Self::upper_bound] on the decimal balance amount.
/// * Constraints on the [`allowed_ids`][Self::allowed_ids]. These are either [`AllowedIds::Any`]
///   or can be constrained to [`AllowedIds::Allowlist`] of [`NonFungibleLocalId`]s.
///   If this case, the ids in the resource balance must be a subset of the allowlist.
///
/// ## Trait implementations
/// * The [`PartialEq`] / [`Eq`] implementations both are correctly order-independent on the id sets,
///   from the order-independent implementation of [`IndexSet`].
///
/// ## Validity
///
/// To be valid, the following checks must be upheld:
///
/// * If `allowed_ids` is [`AllowedIds::Any`]:
///   * `known_ids.len() <= lower_inclusive <= upper_inclusive`
///
/// * If `allowed_ids` is [`AllowedIds::Allowlist(allowlist)`][AllowedIds::Allowlist]:
///   * `known_ids.len() <= lower_inclusive <= upper_inclusive <= allowlist.len()`
///   * `known_ids.is_subset(allowlist)`
///
/// Also, depending on the resource type, further validations are added:
///
/// * If the constraints are for a fungible resource, then [`required_ids`][Self::required_ids] must be
/// empty, and [`allowed_ids`][Self::allowed_ids] must be [`AllowedIds::Any`] (or, if the upper bound is
/// zero, [`AllowedIds::Allowlist`] with an empty list is also acceptable).
///
/// * If the constraints are for a non-fungible resource, then any decimal balances must be integers.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct GeneralResourceConstraint {
    pub required_ids: IndexSet<NonFungibleLocalId>,
    pub lower_bound: LowerBound,
    pub upper_bound: UpperBound,
    pub allowed_ids: AllowedIds,
}

impl GeneralResourceConstraint {
    pub fn fungible(
        lower_bound: impl ResolvableLowerBound,
        upper_bound: impl ResolvableUpperBound,
    ) -> Self {
        let constraint = Self {
            required_ids: Default::default(),
            lower_bound: lower_bound.resolve(),
            upper_bound: upper_bound.resolve(),
            allowed_ids: AllowedIds::Any,
        };

        if !constraint.is_valid_for_fungible_use() {
            panic!("Bounds are invalid for fungible use");
        }

        constraint
    }

    pub fn non_fungible_no_allow_list(
        required_ids: impl IntoIterator<Item = NonFungibleLocalId>,
        lower_bound: impl ResolvableLowerBound,
        upper_bound: impl ResolvableUpperBound,
    ) -> Self {
        let constraint = Self {
            required_ids: required_ids.into_iter().collect(),
            lower_bound: lower_bound.resolve(),
            upper_bound: upper_bound.resolve(),
            allowed_ids: AllowedIds::Any,
        };

        if !constraint.is_valid_for_non_fungible_use() {
            panic!("Bounds are invalid for non-fungible use");
        }

        constraint
    }

    pub fn non_fungible_with_allow_list(
        required_ids: impl IntoIterator<Item = NonFungibleLocalId>,
        lower_bound: impl ResolvableLowerBound,
        upper_bound: impl ResolvableUpperBound,
        allowed_ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Self {
        let constraint = Self {
            required_ids: required_ids.into_iter().collect(),
            lower_bound: lower_bound.resolve(),
            upper_bound: upper_bound.resolve(),
            allowed_ids: AllowedIds::allowlist(allowed_ids),
        };

        if !constraint.is_valid_for_non_fungible_use() {
            panic!("Bounds are invalid for non-fungible use");
        }

        constraint
    }

    pub fn is_valid_for_fungible_use(&self) -> bool {
        self.required_ids.is_empty()
            && self.lower_bound.is_valid_for_fungible_use()
            && self.upper_bound.is_valid_for_fungible_use()
            && self.allowed_ids.is_valid_for_fungible_use()
            && self.are_bounds_valid()
    }

    pub fn is_valid_for_non_fungible_use(&self) -> bool {
        self.lower_bound.is_valid_for_non_fungible_use()
            && self.upper_bound.is_valid_for_non_fungible_use()
            && self.are_bounds_valid()
    }

    pub fn has_amount_constraints(&self) -> bool {
        match self.lower_bound {
            LowerBound::NonZero => return true,
            LowerBound::Inclusive(inclusive) => {
                if inclusive.is_positive() {
                    return true;
                }
            }
        }

        match self.upper_bound {
            UpperBound::Inclusive(_) => {
                return true;
            }
            UpperBound::Unbounded => {}
        }

        false
    }

    pub fn check_amount(&self, amount: Decimal) -> Result<(), GeneralResourceConstraintError> {
        match self.lower_bound {
            LowerBound::NonZero => {
                if amount.is_zero() {
                    return Err(GeneralResourceConstraintError::AmountNonZero);
                }
            }
            LowerBound::Inclusive(inclusive) => {
                if amount < inclusive {
                    return Err(GeneralResourceConstraintError::AmountLowerBound {
                        lower_bound_inclusive: inclusive,
                        actual: amount
                    })
                }
            }
        }
        match self.upper_bound {
            UpperBound::Inclusive(inclusive) => {
                if amount > inclusive {
                    return Err(GeneralResourceConstraintError::AmountUpperBound {
                        upper_bound_inclusive: inclusive,
                        actual: amount,
                    })
                }
            }
            UpperBound::Unbounded => {}
        }

        Ok(())
    }

    pub fn has_non_fungible_id_constraints(&self) -> bool {
        !self.required_ids.is_empty() || self.allowed_ids.has_constraints()
    }

    pub fn check_non_fungibles(&self, ids: &IndexSet<NonFungibleLocalId>) -> Result<(), GeneralResourceConstraintError> {
        for id in &self.required_ids {
            if !ids.contains(id) {
                return Err(GeneralResourceConstraintError::NonFungibleRequired { missing_id: id.clone() })
            }
        }
        match &self.allowed_ids {
            AllowedIds::Allowlist(allowed) => {
                for id in ids {
                    if !allowed.contains(id) {
                        return Err(GeneralResourceConstraintError::NonFungibleAllowed { invalid_id: id.clone() })
                    }
                }
            }
            AllowedIds::Any => {}
        }

        Ok(())
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum GeneralResourceConstraintError {
    AmountNonZero,
    AmountLowerBound {
        lower_bound_inclusive: Decimal,
        actual: Decimal,
    },
    AmountUpperBound {
        upper_bound_inclusive: Decimal,
        actual: Decimal,
    },
    NonFungibleRequired {
        missing_id: NonFungibleLocalId,
    },
    NonFungibleAllowed {
        invalid_id: NonFungibleLocalId,
    },
}

/// Represents a lower bound on a non-negative decimal.
///
/// [`LowerBound::NonZero`] represents a lower bound of an infinitesimal amount above 0,
/// and is included for clarity of intention. Considering `Decimal` has a limited precision
/// of `10^(-18)`, it is roughly equivalent to an inclusive bound of `10^(-18)`,
/// or `Decimal::from_attos(1)`.
///
/// You can extract this equivalent decimal using the [`Self::equivalent_decimal`] method.
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

    /// ## Panics
    /// * Panics if the decimal is not resolvable or is non-negative
    pub fn at_least(decimal: Decimal) -> Self {
        if decimal.is_negative() {
            panic!("An at_least bound is negative");
        }
        Self::Inclusive(decimal)
    }

    pub fn of(lower_bound: impl ResolvableLowerBound) -> Self {
        lower_bound.resolve()
    }

    pub fn is_valid_for_fungible_use(&self) -> bool {
        match self {
            LowerBound::NonZero => true,
            LowerBound::Inclusive(amount) => !amount.is_negative(),
        }
    }

    pub fn is_valid_for_non_fungible_use(&self) -> bool {
        match self {
            LowerBound::NonZero => true,
            LowerBound::Inclusive(amount) => {
                !amount.is_negative() && amount.checked_floor() == Some(*amount)
            }
        }
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
/// [`UpperBound::Unbounded`] represents an upper bound above any possible decimal,
/// and is included for clarity of intention. Considering `Decimal` has a max size,
/// it is effectively equivalent to an inclusive bound of `Decimal::MAX`.
///
/// You can extract this equivalent decimal using the [`Self::equivalent_decimal`] method.
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

    pub fn of(upper_bound: impl ResolvableUpperBound) -> Self {
        upper_bound.resolve()
    }

    pub fn is_valid_for_fungible_use(&self) -> bool {
        match self {
            UpperBound::Inclusive(amount) => !amount.is_negative(),
            UpperBound::Unbounded => true,
        }
    }

    pub fn is_valid_for_non_fungible_use(&self) -> bool {
        match self {
            UpperBound::Inclusive(amount) => {
                !amount.is_negative() && amount.checked_floor() == Some(*amount)
            }
            UpperBound::Unbounded => true,
        }
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

/// Represents which ids are possible in a non-fungible balance.
///
/// [`AllowedIds::Any`] represents that any id is possible.
/// [`AllowedIds::Allowlist`] represents that any ids in the balance have to
/// be in the allowlist.
///
/// For fungible balances, you are permitted to use either [`AllowedIds::Any`]
/// or [`AllowedIds::Allowlist`] with an empty allowlist.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub enum AllowedIds {
    Allowlist(IndexSet<NonFungibleLocalId>),
    Any,
}

impl AllowedIds {
    pub fn none() -> Self {
        Self::Allowlist(Default::default())
    }

    pub fn allowlist(allowlist: impl IntoIterator<Item = NonFungibleLocalId>) -> Self {
        Self::Allowlist(allowlist.into_iter().collect())
    }

    pub fn any() -> Self {
        Self::Any
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

    pub fn has_constraints(&self) -> bool {
        match self {
            AllowedIds::Allowlist(allowlist) => !allowlist.is_empty(),
            AllowedIds::Any => false,
        }
    }
}

pub enum BoundAdjustmentError {
    DecimalOverflow,
    TakeCannotBeSatisfied,
}

pub trait ResolvableLowerBound {
    fn resolve(self) -> LowerBound;
}

impl ResolvableLowerBound for LowerBound {
    fn resolve(self) -> LowerBound {
        self
    }
}

impl<T: ResolvableDecimal> ResolvableLowerBound for T {
    fn resolve(self) -> LowerBound {
        LowerBound::Inclusive(self.resolve())
    }
}

pub trait ResolvableUpperBound {
    fn resolve(self) -> UpperBound;
}

impl ResolvableUpperBound for UpperBound {
    fn resolve(self) -> UpperBound {
        self
    }
}

impl<T: ResolvableDecimal> ResolvableUpperBound for T {
    fn resolve(self) -> UpperBound {
        UpperBound::Inclusive(self.resolve())
    }
}
