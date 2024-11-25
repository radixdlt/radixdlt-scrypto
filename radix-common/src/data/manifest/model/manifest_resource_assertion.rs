use core::ops::AddAssign;

use crate::internal_prelude::*;

// This file isn't part of the Manifest SBOR value model, but is included here
// for consolidation of the manifest types

#[derive(Debug, Clone, PartialEq, Eq, Default, ManifestSbor, ScryptoSbor)]
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
        self,
        resource_address: ResourceAddress,
        constraint: ManifestResourceConstraint,
    ) -> Self {
        if !constraint.is_valid_for(&resource_address) {
            panic!("Constraint isn't valid for the resource address");
        }
        self.with_unchecked(resource_address, constraint)
    }

    /// Unlike `with`, this does not validate the constraint.
    ///
    /// ## Panics
    /// * Panics if constraints have already been specified against the resource
    pub fn with_unchecked(
        mut self,
        resource_address: ResourceAddress,
        constraint: ManifestResourceConstraint,
    ) -> Self {
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
        amount: impl Resolve<Decimal>,
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
        amount: impl Resolve<Decimal>,
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
        lower_bound: impl Resolve<LowerBound>,
        upper_bound: impl Resolve<UpperBound>,
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

    pub fn len(&self) -> usize {
        self.specified_resources().len()
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

    pub fn validate(
        self,
        balances: AggregateResourceBalances,
        prevent_unspecified_resource_balances: bool,
    ) -> Result<(), ResourceConstraintsError> {
        let AggregateResourceBalances {
            fungible_resources,
            non_fungible_resources,
        } = balances;

        if prevent_unspecified_resource_balances {
            for (resource_address, amount) in fungible_resources.iter() {
                if !self.specified_resources.contains_key(resource_address) && amount.is_positive()
                {
                    return Err(
                        ResourceConstraintsError::UnexpectedNonZeroBalanceOfUnspecifiedResource {
                            resource_address: *resource_address,
                        },
                    );
                }
            }

            for (resource_address, ids) in non_fungible_resources.iter() {
                if !self.specified_resources.contains_key(resource_address) && !ids.is_empty() {
                    return Err(
                        ResourceConstraintsError::UnexpectedNonZeroBalanceOfUnspecifiedResource {
                            resource_address: *resource_address,
                        },
                    );
                }
            }
        }

        let zero_balance = Decimal::ZERO;
        let empty_ids: IndexSet<NonFungibleLocalId> = Default::default();
        for (resource_address, constraint) in self.specified_resources {
            if resource_address.is_fungible() {
                let amount = fungible_resources
                    .get(&resource_address)
                    .unwrap_or(&zero_balance);
                constraint.validate_fungible(*amount).map_err(|error| {
                    ResourceConstraintsError::ResourceConstraintFailed {
                        resource_address,
                        error,
                    }
                })?;
            } else {
                let ids = non_fungible_resources
                    .get(&resource_address)
                    .unwrap_or(&empty_ids);
                constraint.validate_non_fungible(ids).map_err(|error| {
                    ResourceConstraintsError::ResourceConstraintFailed {
                        resource_address,
                        error,
                    }
                })?;
            }
        }

        Ok(())
    }
}

pub struct AggregateResourceBalances {
    fungible_resources: IndexMap<ResourceAddress, Decimal>,
    non_fungible_resources: IndexMap<ResourceAddress, IndexSet<NonFungibleLocalId>>,
}

impl AggregateResourceBalances {
    pub fn new() -> Self {
        Self {
            fungible_resources: Default::default(),
            non_fungible_resources: Default::default(),
        }
    }

    pub fn add_fungible(&mut self, resource_address: ResourceAddress, amount: Decimal) {
        if amount.is_positive() {
            self.fungible_resources
                .entry(resource_address)
                .or_default()
                .add_assign(amount);
        }
    }

    pub fn add_non_fungible(
        &mut self,
        resource_address: ResourceAddress,
        ids: IndexSet<NonFungibleLocalId>,
    ) {
        if !ids.is_empty() {
            self.non_fungible_resources
                .entry(resource_address)
                .or_default()
                .extend(ids);
        }
    }

    pub fn validate_only(
        self,
        constraints: ManifestResourceConstraints,
    ) -> Result<(), ResourceConstraintsError> {
        constraints.validate(self, true)
    }

    pub fn validate_includes(
        self,
        constraints: ManifestResourceConstraints,
    ) -> Result<(), ResourceConstraintsError> {
        constraints.validate(self, false)
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

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
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

    pub fn validate_non_fungible(
        self,
        ids: &IndexSet<NonFungibleLocalId>,
    ) -> Result<(), ResourceConstraintError> {
        let amount = Decimal::from(ids.len());
        match self {
            ManifestResourceConstraint::NonZeroAmount => {
                if ids.is_empty() {
                    return Err(ResourceConstraintError::ExpectedNonZeroAmount);
                }
            }
            ManifestResourceConstraint::ExactAmount(expected_exact_amount) => {
                if amount.ne(&expected_exact_amount) {
                    return Err(ResourceConstraintError::ExpectedExactAmount {
                        actual_amount: amount,
                        expected_amount: expected_exact_amount,
                    });
                }
            }
            ManifestResourceConstraint::AtLeastAmount(expected_at_least_amount) => {
                if amount < expected_at_least_amount {
                    return Err(ResourceConstraintError::ExpectedAtLeastAmount {
                        expected_at_least_amount,
                        actual_amount: amount,
                    });
                }
            }
            ManifestResourceConstraint::ExactNonFungibles(expected_exact_ids) => {
                if let Some(missing_id) = expected_exact_ids.difference(ids).next() {
                    return Err(ResourceConstraintError::NonFungibleMissing {
                        missing_id: missing_id.clone(),
                    });
                }
                if let Some(disallowed_id) = ids.difference(&expected_exact_ids).next() {
                    return Err(ResourceConstraintError::NonFungibleNotAllowed {
                        disallowed_id: disallowed_id.clone(),
                    });
                }
            }
            ManifestResourceConstraint::AtLeastNonFungibles(expected_at_least_ids) => {
                if let Some(missing_id) = expected_at_least_ids.difference(ids).next() {
                    return Err(ResourceConstraintError::NonFungibleMissing {
                        missing_id: missing_id.clone(),
                    });
                }
            }
            ManifestResourceConstraint::General(constraint) => {
                constraint.validate_non_fungible_ids(ids)?;
            }
        }

        Ok(())
    }

    pub fn validate_fungible(self, amount: Decimal) -> Result<(), ResourceConstraintError> {
        match self {
            ManifestResourceConstraint::NonZeroAmount => {
                if amount.is_zero() {
                    return Err(ResourceConstraintError::ExpectedNonZeroAmount);
                }
            }
            ManifestResourceConstraint::ExactAmount(expected_exact_amount) => {
                if amount.ne(&expected_exact_amount) {
                    return Err(ResourceConstraintError::ExpectedExactAmount {
                        actual_amount: amount,
                        expected_amount: expected_exact_amount,
                    });
                }
            }
            ManifestResourceConstraint::AtLeastAmount(expected_at_least_amount) => {
                if amount < expected_at_least_amount {
                    return Err(ResourceConstraintError::ExpectedAtLeastAmount {
                        expected_at_least_amount,
                        actual_amount: amount,
                    });
                }
            }
            ManifestResourceConstraint::ExactNonFungibles(..) => {
                return Err(
                    ResourceConstraintError::NonFungibleConstraintNotValidForFungibleResource,
                );
            }
            ManifestResourceConstraint::AtLeastNonFungibles(..) => {
                return Err(
                    ResourceConstraintError::NonFungibleConstraintNotValidForFungibleResource,
                );
            }
            ManifestResourceConstraint::General(constraint) => {
                constraint.validate_fungible(amount)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ResourceConstraintsError {
    UnexpectedNonZeroBalanceOfUnspecifiedResource {
        resource_address: ResourceAddress,
    },
    ResourceConstraintFailed {
        resource_address: ResourceAddress,
        error: ResourceConstraintError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ResourceConstraintError {
    NonFungibleConstraintNotValidForFungibleResource,
    ExpectedNonZeroAmount,
    ExpectedExactAmount {
        expected_amount: Decimal,
        actual_amount: Decimal,
    },
    ExpectedAtLeastAmount {
        expected_at_least_amount: Decimal,
        actual_amount: Decimal,
    },
    ExpectedAtMostAmount {
        expected_at_most_amount: Decimal,
        actual_amount: Decimal,
    },
    // We purposefully don't have an `ExpectedExactNonFungibles` to avoid
    // a malicious transaction creating a 2MB native error with a massive
    // list of required non-fungibles. Instead, we return one of
    // `RequiredNonFungibleMissing` or `NonFungibleNotAllowed`.
    NonFungibleMissing {
        missing_id: NonFungibleLocalId,
    },
    NonFungibleNotAllowed {
        disallowed_id: NonFungibleLocalId,
    },
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
/// * Constraints on the [`allowed_ids`][Self::allowed_ids]. These are either:
///   * [`AllowedIds::Any`]
///   * [`AllowedIds::Allowlist(allowlist)`][AllowedIds::Allowlist] of [`NonFungibleLocalId`]s.
///     If this case, the ids in the resource balance must be a subset of the allowlist.
///
/// Fungible resources are viewed as a specialization of non-fungible resources where we disregard
/// ids and permit non-integer balances. So you must use [`AllowedIds::Any`] with fungible resources.
/// An empty allowlist is also permitted if the balance is exactly zero.
///
/// ## Trait implementations
///
/// * The [`PartialEq`] / [`Eq`] implementations both are correctly order-independent on the id sets,
///   from the order-independent implementation of [`IndexSet`].
///
/// ## Validity
///
/// To be valid, the following checks must be satisfied:
///
/// * The numeric bounds must be satisfiable:
///   * [`lower_bound`][Self::lower_bound] `<=` [`upper_bound`][Self::upper_bound]`
///
/// * The id bounds must be satisfiable:
///   * Either [`allowed_ids`][Self::allowed_ids] is [`AllowedIds::Any`]
///   * Or [`allowed_ids`][Self::allowed_ids] is [`AllowedIds::Allowlist(allowlist)`][AllowedIds::Allowlist]
///     and [`required_ids`][Self::required_ids] is a subset of `allowlist`.
///
/// * The numeric and id bounds must be jointly satisfiable, that is, they must overlap:
///   * `required_ids.len() <= upper_bound.equivalent_decimal()`
///   * If there is an allowlist, `lower_bound.equivalent_decimal() <= allowlist.len()`
///
/// Also, depending on the resource type, further checks must be satisfied:
///
/// * If the constraints are for a fungible resource, then [`required_ids`][Self::required_ids] must be
/// empty, and [`allowed_ids`][Self::allowed_ids] must be [`AllowedIds::Any`] (or, if the upper bound is
/// zero, [`AllowedIds::Allowlist`] with an empty list is also acceptable).
///
/// * If the constraints are for a non-fungible resource, then any decimal balances must be integers.
///
/// ## Normalization
///
/// Normalization takes a valid [`GeneralResourceConstraint`] and internally tightens it into a canonical
/// form. The resultant fields satisfies these tighter conditions:
///
/// * Strict ordering of constraints:
///   * `required_ids.len() <= lower_bound <= upper_bound <= allowlist.len()`
/// * Detection of exact definition:
///   * If `required_ids.len() == upper_bound`, then `allowed_ids == AllowedIds::Allowlist(required_ids)`
///   * If `lower_bound == allowlist.len()`, then `required_ids == allowlist`
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
pub struct GeneralResourceConstraint {
    pub required_ids: IndexSet<NonFungibleLocalId>,
    pub lower_bound: LowerBound,
    pub upper_bound: UpperBound,
    pub allowed_ids: AllowedIds,
}

impl GeneralResourceConstraint {
    pub fn fungible(
        lower_bound: impl Resolve<LowerBound>,
        upper_bound: impl Resolve<UpperBound>,
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
        lower_bound: impl Resolve<LowerBound>,
        upper_bound: impl Resolve<UpperBound>,
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
        lower_bound: impl Resolve<LowerBound>,
        upper_bound: impl Resolve<UpperBound>,
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
            && self.is_valid_independent_of_resource_type()
    }

    pub fn is_valid_for_non_fungible_use(&self) -> bool {
        self.lower_bound.is_valid_for_non_fungible_use()
            && self.upper_bound.is_valid_for_non_fungible_use()
            && self.is_valid_independent_of_resource_type()
    }

    pub fn validate_fungible(&self, amount: Decimal) -> Result<(), ResourceConstraintError> {
        self.validate_amount(amount)?;
        // Static checker should have validated that there are no invalid non fungible checks
        Ok(())
    }

    pub fn validate_non_fungible_ids(
        &self,
        ids: &IndexSet<NonFungibleLocalId>,
    ) -> Result<(), ResourceConstraintError> {
        self.validate_amount(Decimal::from(ids.len()))?;

        if let Some(missing_id) = self.required_ids.difference(ids).next() {
            return Err(ResourceConstraintError::NonFungibleMissing {
                missing_id: missing_id.clone(),
            });
        }

        self.allowed_ids.validate_ids(ids)?;

        Ok(())
    }

    fn validate_amount(&self, amount: Decimal) -> Result<(), ResourceConstraintError> {
        self.lower_bound.validate_amount(&amount)?;
        self.upper_bound.validate_amount(&amount)?;
        Ok(())
    }

    pub fn is_valid_independent_of_resource_type(&self) -> bool {
        // Part 1 - Verify numeric bounds
        if self.lower_bound.equivalent_decimal() > self.upper_bound.equivalent_decimal() {
            return false;
        }

        let required_ids_amount = Decimal::from(self.required_ids.len());

        // Part 3a - Verify there exists an overlap with the required ids
        if required_ids_amount > self.upper_bound.equivalent_decimal() {
            return false;
        }

        match &self.allowed_ids {
            AllowedIds::Allowlist(allowlist) => {
                let allowlist_ids_amount = Decimal::from(allowlist.len());

                // Part 3b - Verify the exists an overlap with the allowed ids
                if self.lower_bound.equivalent_decimal() > allowlist_ids_amount {
                    return false;
                }

                // Part 2 - Verify id bounds
                if !self.required_ids.is_subset(allowlist) {
                    return false;
                }
            }
            AllowedIds::Any => {}
        }

        true
    }

    /// The process of normalization defined under [`GeneralResourceConstraint`].
    ///
    /// This method is assumed to apply to a *valid* [`GeneralResourceConstraint`] - else the result is non-sensical.
    pub fn normalize(&mut self) {
        let required_ids_len = Decimal::from(self.required_ids.len());

        // First, constrain the numeric bounds by the id bounds
        if self.lower_bound.equivalent_decimal() < required_ids_len {
            self.lower_bound = LowerBound::Inclusive(required_ids_len);
        }
        if let AllowedIds::Allowlist(allowlist) = &self.allowed_ids {
            let allowlist_len = Decimal::from(allowlist.len());
            if allowlist_len < self.upper_bound.equivalent_decimal() {
                self.upper_bound = UpperBound::Inclusive(allowlist_len);
            }
        }

        // Next, constrain the id bounds if we detect there must be equality of ids.
        // First, we check they're not already equal...
        if self.allowed_ids.allowlist_equivalent_length() > self.required_ids.len() {
            if required_ids_len == self.upper_bound.equivalent_decimal() {
                // Note - this can change a zero non-fungible amount to have an
                // empty allowlist. This is allowed under the validity rules.
                self.allowed_ids = AllowedIds::Allowlist(self.required_ids.clone());
            } else if let AllowedIds::Allowlist(allowlist) = &self.allowed_ids {
                if Decimal::from(allowlist.len()) == self.lower_bound.equivalent_decimal() {
                    self.required_ids = allowlist.clone();
                }
            }
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
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

    pub fn of(lower_bound: impl Resolve<LowerBound>) -> Self {
        lower_bound.resolve()
    }

    pub fn validate_amount(&self, amount: &Decimal) -> Result<(), ResourceConstraintError> {
        match self {
            LowerBound::NonZero => {
                if amount.is_zero() {
                    return Err(ResourceConstraintError::ExpectedNonZeroAmount);
                }
            }
            LowerBound::Inclusive(inclusive) => {
                if amount < inclusive {
                    return Err(ResourceConstraintError::ExpectedAtLeastAmount {
                        expected_at_least_amount: *inclusive,
                        actual_amount: *amount,
                    });
                }
            }
        }
        Ok(())
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
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

    pub fn of(upper_bound: impl Resolve<UpperBound>) -> Self {
        upper_bound.resolve()
    }

    pub fn validate_amount(&self, amount: &Decimal) -> Result<(), ResourceConstraintError> {
        match self {
            UpperBound::Inclusive(inclusive) => {
                if amount > inclusive {
                    return Err(ResourceConstraintError::ExpectedAtMostAmount {
                        expected_at_most_amount: *inclusive,
                        actual_amount: *amount,
                    });
                }
            }
            UpperBound::Unbounded => {}
        }
        Ok(())
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
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
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

    pub fn validate_ids(
        &self,
        ids: &IndexSet<NonFungibleLocalId>,
    ) -> Result<(), ResourceConstraintError> {
        match self {
            AllowedIds::Allowlist(allowed) => {
                for id in ids {
                    if !allowed.contains(id) {
                        return Err(ResourceConstraintError::NonFungibleNotAllowed {
                            disallowed_id: id.clone(),
                        });
                    }
                }
            }
            AllowedIds::Any => {}
        }
        Ok(())
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

    pub fn is_allow_list_and(
        &self,
        callback: impl FnOnce(&IndexSet<NonFungibleLocalId>) -> bool,
    ) -> bool {
        match self {
            AllowedIds::Allowlist(index_set) => callback(index_set),
            AllowedIds::Any => false,
        }
    }
}

pub enum BoundAdjustmentError {
    DecimalOverflow,
    TakeCannotBeSatisfied,
}

resolvable_with_identity_impl!(LowerBound);

impl<T: Resolve<Decimal>> ResolveFrom<T> for LowerBound {
    fn resolve_from(value: T) -> Self {
        LowerBound::Inclusive(value.resolve())
    }
}

resolvable_with_identity_impl!(UpperBound);

impl<T: Resolve<Decimal>> ResolveFrom<T> for UpperBound {
    fn resolve_from(value: T) -> Self {
        UpperBound::Inclusive(value.resolve())
    }
}
