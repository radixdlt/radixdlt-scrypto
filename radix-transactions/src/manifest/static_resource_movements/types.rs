use super::*;
use crate::internal_prelude::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::prelude::*;

/// A type representing partial knowledge of the balances of some number of resources.
///
/// This type can be used to model the worktop, and can be used for modelling the inbound/
/// outbound resources for any instruction.
///
/// The knowledge is split between specified resources (where we store a [`ResourceBound`]
/// for each resource), and unspecified resources, captured by an [`UnspecifiedResourceKnowledge`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrackedResources {
    /// Captures the bounds of explicitly tracked resources.
    /// Some of these may be
    specified_resources: IndexMap<ResourceAddress, TrackedResource>,
    /// Captures the bounds of unspecified resources.
    unspecified_resources: UnspecifiedResources,
}

impl TrackedResources {
    // Constructors
    pub fn new_empty() -> Self {
        Default::default()
    }

    pub fn new_with_possible_balance_of_unspecified_resources(
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        Self {
            specified_resources: Default::default(),
            unspecified_resources: UnspecifiedResources::MayBePresent(
                change_sources.into_iter().collect(),
            ),
        }
    }

    // Deconstructors
    pub fn deconstruct(
        self,
    ) -> (
        IndexMap<ResourceAddress, TrackedResource>,
        UnspecifiedResources,
    ) {
        (self.specified_resources, self.unspecified_resources)
    }

    // &self methods
    pub fn specified_resources(&self) -> &IndexMap<ResourceAddress, TrackedResource> {
        &self.specified_resources
    }

    pub fn unspecified_resources(&self) -> &UnspecifiedResources {
        &self.unspecified_resources
    }

    /// Verifies that the bounds are equal, but ignores the sources of those bounds.
    pub fn eq_ignoring_history(&self, other: &Self) -> bool {
        if !self
            .unspecified_resources
            .eq_ignoring_history(&other.unspecified_resources)
        {
            return false;
        }

        // We can't assume self or other are normalized, so it may be that self has a specified resource
        // with a bound equivalent to an unspecified resource bound. Such a resource doesn't need to
        // exist as specified in B.
        // Therefore, instead of just comparing specified_resources, we instead simply check that all
        // bounds of a specified resource in A have the same bound in B (specified or unspecified),
        // AND we check the other way around too.
        for (resource, details) in self.specified_resources.iter() {
            if !other.resource_status(resource).eq_ignoring_history(details) {
                return false;
            }
        }
        for (resource, details) in other.specified_resources.iter() {
            if !self.resource_status(resource).eq_ignoring_history(details) {
                return false;
            }
        }
        return true;
    }

    /// Works for any resource, specified and unspecified.
    fn resource_status(&self, resource: &ResourceAddress) -> Cow<TrackedResource> {
        match self.specified_resources.get(resource) {
            Some(bound) => Cow::Borrowed(bound),
            None => Cow::Owned(self.unspecified_resources.resource_status()),
        }
    }

    /// Works for any resource, specified and unspecified.
    /// If the resource is unspecified, it makes it specified, then returns a reference to the entry.
    fn resource_status_mut(&mut self, resource: ResourceAddress) -> &mut TrackedResource {
        match self.specified_resources.entry(resource) {
            indexmap::map::Entry::Occupied(occupied_entry) => occupied_entry.into_mut(),
            indexmap::map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(self.unspecified_resources.resource_status())
            }
        }
    }

    /// If the unspecified bound is exactly zero, we remove any specific resources which are also zero.
    pub fn normalize(mut self) -> Self {
        self.mut_normalize();
        self
    }

    /// If the unspecified bound is exactly zero, we remove any specific resources which are also zero.
    pub fn mut_normalize(&mut self) {
        let unspecified_resource_details = self.unspecified_resources.resource_status();
        if !unspecified_resource_details.is_zero() {
            return;
        }

        // Minor optimization - prevent recreation of the indexmap if it's not needed
        if !self.specified_resources.values().any(|r| r.is_zero()) {
            return;
        }

        // We wipe self.specified_resources and add back to it as we go.
        // With an index map, if we want to maintain the ordering, this can be more efficient than using swap_remove/
        let existing_specified_resources = core::mem::take(&mut self.specified_resources);

        for (resource_address, tracked_details) in existing_specified_resources {
            if !tracked_details.is_zero() {
                self.specified_resources
                    .insert(resource_address, tracked_details);
            }
        }
    }

    pub fn add_unspecified_resources(
        mut self,
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        self.mut_add_unspecified_resources(change_sources);
        self
    }

    pub fn mut_add_unspecified_resources(
        &mut self,
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) {
        self.unspecified_resources
            .mut_add(UnspecifiedResources::MayBePresent(
                change_sources.into_iter().collect(),
            ));
    }

    pub fn mut_add(&mut self, other: TrackedResources) -> Result<(), StaticResourceMovementsError> {
        // For efficiency, we first handle unspecified resources in other.
        // This if statement isn't necessary for correct logic, but offers a small optimization.
        if other.unspecified_resources.may_be_present() {
            for (resource, resource_bound) in &mut self.specified_resources {
                // If an existing resource isn't specified in other, we have to add its unspecified constraints instead
                if !other.specified_resources.contains_key(resource) {
                    resource_bound.add_from(other.unspecified_resources.resource_status())?;
                }
            }
        }

        for (other_resource, other_resource_bound) in other.specified_resources {
            self.resource_status_mut(other_resource)
                .add_from(other_resource_bound)?;
        }

        // Order is important here - we have to handle unspecified once we've handled any specified resources,
        // else the `resource_status_mut` will incorrectly pick up the other's unspecified resources.
        self.unspecified_resources
            .mut_add(other.unspecified_resources);

        Ok(())
    }

    pub fn add_resource(
        mut self,
        resource: ResourceAddress,
        amount: TrackedResource,
    ) -> Result<Self, StaticResourceMovementsError> {
        self.mut_add_resource(resource, amount)?;
        Ok(self)
    }

    pub fn mut_add_resource(
        &mut self,
        resource: ResourceAddress,
        amount: TrackedResource,
    ) -> Result<(), StaticResourceMovementsError> {
        if !amount.bounds().is_valid_for(&resource) {
            return Err(StaticResourceMovementsError::BoundsInvalidForResourceKind);
        }
        self.resource_status_mut(resource).add_from(amount)
    }

    pub fn mut_take_resource(
        &mut self,
        resource: ResourceAddress,
        amount: ResourceTakeAmount,
        source: ChangeSource,
    ) -> Result<TrackedResource, StaticResourceMovementsError> {
        if resource.is_fungible() && !amount.aligns_with_fungible_use() {
            return Err(StaticResourceMovementsError::BoundsInvalidForResourceKind);
        }
        self.resource_status_mut(resource).take(amount, source)
    }

    pub fn take_all(&mut self) -> Self {
        core::mem::take(self)
    }

    pub fn handle_resource_assertion(
        &mut self,
        resource_address: ResourceAddress,
        assertion: ResourceBounds,
        source: ChangeSource,
    ) -> Result<(), StaticResourceMovementsError> {
        if !assertion.is_valid_for(&resource_address) {
            return Err(StaticResourceMovementsError::BoundsInvalidForResourceKind);
        }

        self.resource_status_mut(resource_address)
            .handle_assertion(assertion, source)?;

        self.mut_normalize();

        Ok(())
    }

    pub fn handle_resources_only_assertion(
        &mut self,
        constraints: &ManifestResourceConstraints,
        source: ChangeSource,
    ) -> Result<(), StaticResourceMovementsError> {
        // First, we handle the ONLY by asserting all specified resources not included in the constraints are zero.
        for (resource_address, tracked_resource) in self.specified_resources.iter_mut() {
            if !constraints.contains_specified_resource(resource_address) {
                tracked_resource.handle_assertion(ResourceBounds::zero(), source)?;
            }
        }

        // Now we handle the explict assertions
        for (resource_address, constraint) in constraints.iter() {
            self.resource_status_mut(*resource_address)
                .handle_assertion(
                    ResourceBounds::new_for_manifest_constraint(constraint)?,
                    source,
                )?;
        }

        // Any unspecified resources not explicitly handled above are now set to zero.
        self.unspecified_resources.clear();

        // Finally, let's normalize, to get rid of any unneeded constraints which are identically zero
        self.mut_normalize();

        Ok(())
    }

    pub fn handle_resources_include_assertion(
        &mut self,
        constraints: &ManifestResourceConstraints,
        source: ChangeSource,
    ) -> Result<(), StaticResourceMovementsError> {
        for (resource_address, constraint) in constraints.iter() {
            self.resource_status_mut(*resource_address)
                .handle_assertion(
                    ResourceBounds::new_for_manifest_constraint(constraint)?,
                    source,
                )?;
        }

        // Finally, let's normalize, to get rid of any unneeded constraints which are identically zero
        self.mut_normalize();

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UnspecifiedResources {
    /// There are no unspecified resources present
    #[default]
    NonePresent,
    /// There might be non-zero balances of unspecified resources present
    MayBePresent(IndexSet<ChangeSource>),
}

impl UnspecifiedResources {
    pub fn none() -> Self {
        Self::NonePresent
    }

    pub fn some(change_sources: impl IntoIterator<Item = ChangeSource>) -> Self {
        Self::MayBePresent(change_sources.into_iter().collect())
    }

    pub fn clear(&mut self) {
        *self = Self::NonePresent;
    }

    pub fn resource_status(&self) -> TrackedResource {
        match self {
            Self::NonePresent => TrackedResource::zero(),
            Self::MayBePresent(sources) => TrackedResource::zero_or_more(sources.iter().cloned()),
        }
    }

    pub fn resource_bounds(&self) -> ResourceBounds {
        match self {
            Self::NonePresent => ResourceBounds::zero(),
            Self::MayBePresent(_) => ResourceBounds::zero_or_more(),
        }
    }

    pub fn none_are_present(&self) -> bool {
        match self {
            Self::NonePresent => true,
            Self::MayBePresent(_) => false,
        }
    }

    pub fn may_be_present(&self) -> bool {
        match self {
            Self::NonePresent => false,
            Self::MayBePresent(_) => true,
        }
    }

    pub fn add_possible_resource_balance(
        mut self,
        sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        self.mut_add_possible_resource_balance(sources);
        self
    }

    pub fn mut_add_possible_resource_balance(
        &mut self,
        sources: impl IntoIterator<Item = ChangeSource>,
    ) {
        match self {
            mutself @ Self::NonePresent => {
                *mutself = Self::MayBePresent(sources.into_iter().collect());
            }
            Self::MayBePresent(self_sources) => {
                self_sources.extend(sources);
            }
        }
    }

    pub fn add(mut self, other: Self) -> Self {
        self.mut_add(other);
        self
    }

    pub fn mut_add(&mut self, other: Self) {
        match other {
            Self::NonePresent => {}
            Self::MayBePresent(other_sources) => {
                self.mut_add_possible_resource_balance(other_sources);
            }
        }
    }

    /// Verifies that the bounds are equal, but ignores the sources of those bounds.
    pub fn eq_ignoring_history(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NonePresent, Self::NonePresent)
            | (Self::MayBePresent(_), Self::MayBePresent(_)) => true,
            (Self::NonePresent, Self::MayBePresent(_))
            | (Self::MayBePresent(_), Self::NonePresent) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceTakeAmount {
    Amount(Decimal),
    NonFungibles(IndexSet<NonFungibleLocalId>),
    All,
}

impl ResourceTakeAmount {
    pub fn exact_non_fungibles(ids: impl IntoIterator<Item = NonFungibleLocalId>) -> Self {
        Self::NonFungibles(ids.into_iter().collect())
    }

    pub fn exact_amount(
        amount: impl Resolve<Decimal>,
    ) -> Result<Self, StaticResourceMovementsError> {
        let amount = amount.resolve();
        if amount.is_negative() {
            return Err(StaticResourceMovementsError::DecimalAmountIsNegative);
        }
        Ok(Self::Amount(amount))
    }

    pub fn all() -> Self {
        Self::All
    }

    pub fn aligns_with_fungible_use(&self) -> bool {
        match self {
            ResourceTakeAmount::Amount(_) => true,
            ResourceTakeAmount::NonFungibles(_) => false,
            ResourceTakeAmount::All => true,
        }
    }
}

/// Used to track a known quantity of Fungible and NonFungible resources,
/// for example, the content of a bucket.
///
/// ## Invariants
/// The following inequalities are upheld by all constructors:
/// * `required_ids.len() <= lower_inclusive <= upper_inclusive`
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackedResource {
    /// The current known bounds on the resource balance.
    bounds: ResourceBounds,
    /// This history is only maintained since the last time we knew the balance was zero.
    history: ResourceChangeHistory,
}

impl TrackedResource {
    // Constructors
    pub fn zero() -> Self {
        Self {
            bounds: ResourceBounds::zero(),
            history: ResourceChangeHistory::empty(),
        }
    }

    pub fn exact_amount(
        amount: impl Resolve<Decimal>,
        sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Result<Self, StaticResourceMovementsError> {
        Ok(Self::general(
            ResourceBounds::exact_amount(amount)?,
            sources,
        ))
    }

    pub fn at_least_amount(
        amount: impl Resolve<Decimal>,
        sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Result<Self, StaticResourceMovementsError> {
        Ok(Self::general(
            ResourceBounds::at_least_amount(amount)?,
            sources,
        ))
    }

    pub fn non_zero(sources: impl IntoIterator<Item = ChangeSource>) -> Self {
        Self::general(ResourceBounds::non_zero(), sources)
    }

    pub fn zero_or_more(sources: impl IntoIterator<Item = ChangeSource>) -> Self {
        Self::general(ResourceBounds::zero_or_more(), sources)
    }

    pub fn exact_non_fungibles(
        ids: impl IntoIterator<Item = NonFungibleLocalId>,
        sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        Self::general(ResourceBounds::exact_non_fungibles(ids), sources)
    }

    pub fn at_least_non_fungibles(
        ids: impl IntoIterator<Item = NonFungibleLocalId>,
        sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        Self::general(ResourceBounds::at_least_non_fungibles(ids), sources)
    }

    pub fn general(
        add_amount: ResourceBounds,
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        Self::new_advanced(
            add_amount.clone(),
            ResourceChangeHistory::empty().record_add(add_amount, change_sources),
        )
    }

    /// This is only pub so that it can be used in tests
    pub fn new_advanced(add_amount: ResourceBounds, history: ResourceChangeHistory) -> Self {
        Self {
            bounds: add_amount,
            history,
        }
    }

    // Deconstructors
    pub fn deconstruct(self) -> (ResourceBounds, ResourceChangeHistory) {
        (self.bounds, self.history)
    }

    // &self methods
    pub fn bounds(&self) -> &ResourceBounds {
        &self.bounds
    }

    /// Returns true if the bound is known to be zero
    pub fn is_zero(&self) -> bool {
        self.bounds.is_zero()
    }

    /// Verifies that the bounds are equal, but ignores the sources of those bounds.
    pub fn eq_ignoring_history(&self, other: &TrackedResource) -> bool {
        self.bounds == other.bounds
    }

    pub fn history(&self) -> &ResourceChangeHistory {
        &self.history
    }

    // &mut self methods

    /// Adds the quantity from the tracked resource, storing its history separately.
    pub fn add_from(
        &mut self,
        existing: TrackedResource,
    ) -> Result<(), StaticResourceMovementsError> {
        self.bounds.mut_add(existing.bounds.clone())?;
        if self.is_zero() {
            self.history.mut_clear();
        } else {
            self.history
                .mut_record_add_with_history(existing.bounds, existing.history);
        }
        Ok(())
    }

    pub fn add(
        &mut self,
        amount: ResourceBounds,
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Result<(), StaticResourceMovementsError> {
        self.bounds.mut_add(amount.clone())?;

        if self.is_zero() {
            self.history.mut_clear();
        } else {
            self.history.mut_record_add(amount, change_sources)
        }

        Ok(())
    }

    pub fn take(
        &mut self,
        take_amount: ResourceTakeAmount,
        source: ChangeSource,
    ) -> Result<TrackedResource, StaticResourceMovementsError> {
        match take_amount {
            ResourceTakeAmount::All => {
                // In the case of a "take all" we just return the existing contents and history,
                // without changing it - and we replace with a blank slate.
                return Ok(core::mem::replace(self, Self::zero()));
            }
            _ => {
                let taken_amount = self.bounds.mut_take(take_amount.clone())?;
                if self.is_zero() {
                    self.history.mut_clear();
                } else {
                    self.history.mut_record_take(take_amount, source);
                }

                // FUTURE TWEAK: Can output an inequality constraint using history.all_changes()
                Ok(Self::general(taken_amount, [source]))
            }
        }
    }

    pub fn take_all(&mut self) -> Self {
        core::mem::replace(self, Self::zero())
    }

    pub fn handle_assertion(
        &mut self,
        assertion: ResourceBounds,
        source: ChangeSource,
    ) -> Result<(), StaticResourceMovementsError> {
        self.bounds.mut_handle_assertion(assertion.clone())?;

        if self.is_zero() {
            self.history.mut_clear();
        } else {
            self.history.mut_record_assertion(assertion, source);
        }

        // FUTURE TWEAK: Can output an inequality constraint using history.all_changes()
        Ok(())
    }
}

/// [`ResourceBounds`] captures constraints on the balance of a single fungible or non-fungible
/// resource.
///
/// It is a specialization of a [`GeneralResourceConstraint`], and wraps that type, ensuring that:
/// * It is always valid and normalized
/// * We can add methods/concepts specific to bounds tracking and constraining
///
/// ## Trait implementations
/// * The [`PartialEq`] / [`Eq`] implementations both are correctly order-independent on the id sets,
///   from the order-independent implementation of [`IndexSet`].
///
/// ## Invariants
///
/// All methods/functions on this class must guarantee that the wrapped [`GeneralResourceConstraint`]
/// is valid and normalized (according to the descriptions of validity and normalization on its documentation).
///
/// That is, that the following invariants are upheld:
///
/// * If `allowed_ids` is [`AllowedIds::Any`]:
///   * `known_ids.len() <= lower_inclusive <= upper_inclusive`
///
/// * If `allowed_ids` is [`AllowedIds::Allowlist(allowlist)`][AllowedIds::Allowlist]:
///   * `known_ids.len() <= lower_inclusive <= upper_inclusive <= allowlist.len()`
///   * `known_ids.is_subset(allowlist)`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBounds {
    constraints: GeneralResourceConstraint,
}

impl Default for ResourceBounds {
    fn default() -> Self {
        Self::zero()
    }
}

impl ResourceBounds {
    fn of_valid_normalized(constraints: GeneralResourceConstraint) -> Self {
        Self { constraints }
    }

    pub fn zero() -> Self {
        Self::of_valid_normalized(GeneralResourceConstraint {
            required_ids: Default::default(),
            lower_bound: LowerBound::zero(),
            upper_bound: UpperBound::zero(),
            allowed_ids: AllowedIds::Allowlist(Default::default()),
        })
    }

    pub fn zero_or_more() -> Self {
        Self::of_valid_normalized(GeneralResourceConstraint {
            required_ids: Default::default(),
            lower_bound: LowerBound::zero(),
            upper_bound: UpperBound::unbounded(),
            allowed_ids: AllowedIds::Any,
        })
    }

    pub fn non_zero() -> Self {
        Self::of_valid_normalized(GeneralResourceConstraint {
            required_ids: Default::default(),
            lower_bound: LowerBound::non_zero(),
            upper_bound: UpperBound::unbounded(),
            allowed_ids: AllowedIds::Any,
        })
    }

    pub fn exact_amount(
        amount: impl Resolve<Decimal>,
    ) -> Result<Self, StaticResourceMovementsError> {
        let amount = amount.resolve();
        if amount.is_negative() {
            return Err(StaticResourceMovementsError::DecimalAmountIsNegative);
        }
        Ok(Self::of_valid_normalized(GeneralResourceConstraint {
            required_ids: Default::default(),
            lower_bound: LowerBound::at_least(amount),
            upper_bound: UpperBound::at_most(amount),
            allowed_ids: AllowedIds::Any,
        }))
    }

    pub fn at_least_amount(
        amount: impl Resolve<Decimal>,
    ) -> Result<Self, StaticResourceMovementsError> {
        let amount = amount.resolve();
        if amount.is_negative() {
            return Err(StaticResourceMovementsError::DecimalAmountIsNegative);
        }

        Ok(Self::of_valid_normalized(GeneralResourceConstraint {
            required_ids: Default::default(),
            lower_bound: LowerBound::at_least(amount),
            upper_bound: UpperBound::unbounded(),
            allowed_ids: AllowedIds::Any,
        }))
    }

    pub fn exact_non_fungibles(ids: impl IntoIterator<Item = NonFungibleLocalId>) -> Self {
        let ids = ids.into_iter().collect::<IndexSet<_>>();
        let amount_of_ids: Decimal = ids.len().into();

        Self::of_valid_normalized(GeneralResourceConstraint {
            required_ids: ids.clone(),
            lower_bound: LowerBound::at_least(amount_of_ids),
            upper_bound: UpperBound::at_most(amount_of_ids),
            allowed_ids: AllowedIds::Allowlist(ids),
        })
    }

    pub fn at_least_non_fungibles(
        required_ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Self {
        let ids = required_ids.into_iter().collect::<IndexSet<_>>();
        let amount_of_ids: Decimal = ids.len().into();

        Self::of_valid_normalized(GeneralResourceConstraint {
            required_ids: ids,
            lower_bound: LowerBound::at_least(amount_of_ids),
            upper_bound: UpperBound::unbounded(),
            allowed_ids: AllowedIds::Any,
        })
    }

    pub fn general_fungible(
        lower_bound: impl Resolve<LowerBound>,
        upper_bound: impl Resolve<UpperBound>,
    ) -> Result<Self, StaticResourceMovementsError> {
        let mut constraints = GeneralResourceConstraint {
            required_ids: Default::default(),
            lower_bound: lower_bound.resolve(),
            upper_bound: upper_bound.resolve(),
            allowed_ids: AllowedIds::Any,
        };

        if !constraints.is_valid_for_fungible_use() {
            return Err(StaticResourceMovementsError::ConstraintBoundsInvalid);
        }

        constraints.normalize();

        Ok(Self::of_valid_normalized(constraints))
    }

    pub fn general_non_fungible_no_allowlist(
        required_ids: impl IntoIterator<Item = NonFungibleLocalId>,
        lower_bound: impl Resolve<LowerBound>,
        upper_bound: impl Resolve<UpperBound>,
    ) -> Result<Self, StaticResourceMovementsError> {
        let mut constraints = GeneralResourceConstraint {
            required_ids: required_ids.into_iter().collect::<IndexSet<_>>(),
            lower_bound: lower_bound.resolve(),
            upper_bound: upper_bound.resolve(),
            allowed_ids: AllowedIds::Any,
        };

        if !constraints.is_valid_for_non_fungible_use() {
            return Err(StaticResourceMovementsError::ConstraintBoundsInvalid);
        }

        constraints.normalize();

        Ok(Self::of_valid_normalized(constraints))
    }

    pub fn general_non_fungible_with_allowlist(
        required_ids: impl IntoIterator<Item = NonFungibleLocalId>,
        lower_bound: impl Resolve<LowerBound>,
        upper_bound: impl Resolve<UpperBound>,
        id_allowlist: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Result<Self, StaticResourceMovementsError> {
        let mut constraints = GeneralResourceConstraint {
            required_ids: required_ids.into_iter().collect(),
            lower_bound: lower_bound.resolve(),
            upper_bound: upper_bound.resolve(),
            allowed_ids: AllowedIds::Allowlist(id_allowlist.into_iter().collect()),
        };

        if !constraints.is_valid_for_non_fungible_use() {
            return Err(StaticResourceMovementsError::ConstraintBoundsInvalid);
        }

        constraints.normalize();

        Ok(Self::of_valid_normalized(constraints))
    }

    pub fn new_for_manifest_constraint(
        constraint: &ManifestResourceConstraint,
    ) -> Result<Self, StaticResourceMovementsError> {
        match constraint {
            ManifestResourceConstraint::NonZeroAmount => Ok(Self::non_zero()),
            ManifestResourceConstraint::ExactAmount(amount) => Self::exact_amount(*amount),
            ManifestResourceConstraint::AtLeastAmount(amount) => Self::at_least_amount(*amount),
            ManifestResourceConstraint::ExactNonFungibles(ids) => {
                Ok(Self::exact_non_fungibles(ids.iter().cloned()))
            }
            ManifestResourceConstraint::AtLeastNonFungibles(ids) => {
                Ok(Self::at_least_non_fungibles(ids.iter().cloned()))
            }
            ManifestResourceConstraint::General(constraints) => {
                // We don't know if it's fungible or non-fungible, so we at least verify the bounds.
                if !constraints.is_valid_independent_of_resource_type() {
                    return Err(StaticResourceMovementsError::ConstraintBoundsInvalid);
                }

                let mut constraints = constraints.clone();

                constraints.normalize();

                Ok(Self::of_valid_normalized(constraints))
            }
        }
    }

    pub fn deconstruct(
        self,
    ) -> (
        IndexSet<NonFungibleLocalId>,
        LowerBound,
        UpperBound,
        AllowedIds,
    ) {
        let constraints = self.constraints;
        (
            constraints.required_ids,
            constraints.lower_bound,
            constraints.upper_bound,
            constraints.allowed_ids,
        )
    }

    // &self methods
    pub fn numeric_bounds(&self) -> (LowerBound, UpperBound) {
        (self.constraints.lower_bound, self.constraints.upper_bound)
    }

    pub fn required_ids(&self) -> &IndexSet<NonFungibleLocalId> {
        &self.constraints.required_ids
    }

    pub fn lower_bound(&self) -> LowerBound {
        self.constraints.lower_bound
    }

    pub fn upper_bound(&self) -> UpperBound {
        self.constraints.upper_bound
    }

    pub fn allowed_ids(&self) -> &AllowedIds {
        &self.constraints.allowed_ids
    }

    pub fn is_valid_for(&self, resource_address: &ResourceAddress) -> bool {
        if resource_address.is_fungible() {
            self.is_valid_for_fungible_use()
        } else {
            self.is_valid_for_non_fungible_use()
        }
    }

    pub fn is_valid_for_fungible_use(&self) -> bool {
        return self.constraints.is_valid_for_fungible_use();
    }

    pub fn is_valid_for_non_fungible_use(&self) -> bool {
        return self.constraints.is_valid_for_non_fungible_use();
    }

    /// Returns true if the bound is known to be zero
    pub fn is_zero(&self) -> bool {
        // We don't just compare to zero; we do an equivalent upper-bounds check.
        // For more flexibility, this also works with unnormalized `ResourceBounds`
        // (e.g. if the allowed bound has `AllowedIds::Any`, such as in the `AggregatedBalanceChange`)
        self.upper_bound() == UpperBound::zero()
    }

    pub fn is_exact_ids(&self) -> bool {
        self.get_exact_ids().is_some()
    }

    pub fn get_exact_ids(&self) -> Option<&IndexSet<NonFungibleLocalId>> {
        let required_ids_count = Decimal::from(self.constraints.required_ids.len());
        match self.constraints.upper_bound {
            UpperBound::Inclusive(count) if count == required_ids_count => {
                Some(&self.constraints.required_ids)
            }
            _ => None,
        }
    }

    pub fn is_exact_amount(&self) -> bool {
        self.get_exact_amount().is_some()
    }

    pub fn get_exact_amount(&self) -> Option<Decimal> {
        match self.numeric_bounds() {
            (LowerBound::Inclusive(lower), UpperBound::Inclusive(upper)) if lower == upper => {
                Some(lower)
            }
            _ => None,
        }
    }

    pub fn mut_clear_required_ids(&mut self) {
        self.constraints.required_ids.clear();
    }

    pub fn mut_clear_allowed_ids(&mut self) {
        self.constraints.allowed_ids = AllowedIds::Any;
    }

    pub fn add(mut self, other: Self) -> Result<Self, StaticResourceMovementsError> {
        self.mut_add(other)?;
        Ok(self)
    }

    pub fn mut_add(&mut self, other: Self) -> Result<(), StaticResourceMovementsError> {
        let this = &mut self.constraints;
        let other = other.constraints;

        this.lower_bound.add_from(other.lower_bound)?;
        this.upper_bound.add_from(other.upper_bound)?;
        for id in other.required_ids.into_iter() {
            if !this.required_ids.insert(id) {
                return Err(StaticResourceMovementsError::DuplicateNonFungibleId);
            }
        }

        match (&mut this.allowed_ids, other.allowed_ids) {
            (AllowedIds::Any, _) => {} // If all ids are allowed, keep it that way
            (self_permitted_ids, AllowedIds::Any) => *self_permitted_ids = AllowedIds::Any,
            (AllowedIds::Allowlist(allow_list), AllowedIds::Allowlist(other_allow_list)) => {
                // Unlike the known ids, it's fine for the allow lists to overlap, so don't error on duplicates.
                allow_list.extend(other_allow_list);
            }
        }

        this.normalize();

        Ok(())
    }

    pub fn take(
        mut self,
        amount: ResourceTakeAmount,
    ) -> Result<Self, StaticResourceMovementsError> {
        self.mut_take(amount)?;
        Ok(self)
    }

    pub fn mut_take(
        &mut self,
        amount: ResourceTakeAmount,
    ) -> Result<Self, StaticResourceMovementsError> {
        let this = &mut self.constraints;

        match amount {
            ResourceTakeAmount::Amount(take_amount) => {
                if take_amount.is_negative() {
                    return Err(StaticResourceMovementsError::DecimalAmountIsNegative);
                }
                this.lower_bound.take_amount(take_amount);
                this.upper_bound.take_amount(take_amount)?;

                // For known ids, we don't know which ids were taken, so we have to clear them.
                // But the allowed ids stay as-is
                if take_amount.is_positive() {
                    this.required_ids.clear();
                }

                this.normalize();

                // Taken amount
                Self::exact_amount(take_amount)
            }
            ResourceTakeAmount::NonFungibles(taken_ids) => {
                let take_amount = Decimal::from(taken_ids.len());

                this.lower_bound.take_amount(take_amount);
                this.upper_bound.take_amount(take_amount)?;

                // Remove any taken ids from the list of known/required ids.
                // It's okay if some of the taken ids weren't required to be present.
                this.required_ids = this.required_ids.difference(&taken_ids).cloned().collect();

                // Finally, we check all the taken ids are in the allow list (if it exists) and these ids
                // are removed from the allow list.
                if let AllowedIds::Allowlist(allow_list) = &mut this.allowed_ids {
                    if !taken_ids.is_subset(allow_list) {
                        return Err(StaticResourceMovementsError::TakeCannotBeSatisfied);
                    }
                    *allow_list = allow_list.difference(&taken_ids).cloned().collect();
                }

                // We check remaining invariants: it's an error if, after taking all matching ids,
                // we now are required to have too many.
                // e.g. This catches "Add A, 1 of 1; Take B, C"
                if Decimal::from(this.required_ids.len()) > this.lower_bound.equivalent_decimal() {
                    return Err(StaticResourceMovementsError::TakeCannotBeSatisfied);
                }

                this.normalize();

                // Taken amount
                Ok(Self::exact_non_fungibles(taken_ids))
            }
            ResourceTakeAmount::All => {
                // Taken amount
                Ok(core::mem::replace(self, Self::zero()))
            }
        }
    }

    pub fn handle_assertion(
        mut self,
        assertion: ResourceBounds,
    ) -> Result<Self, StaticResourceMovementsError> {
        self.mut_handle_assertion(assertion)?;
        Ok(self)
    }

    pub fn mut_handle_assertion(
        &mut self,
        assertion: ResourceBounds,
    ) -> Result<(), StaticResourceMovementsError> {
        let this = &mut self.constraints;
        let assertion = assertion.constraints;

        // Possibly increase lower bound and decrease upper bound
        this.lower_bound.constrain_to(assertion.lower_bound);
        this.upper_bound.constrain_to(assertion.upper_bound);

        // Handle the allow list
        if let AllowedIds::Allowlist(assertion_allowlist) = assertion.allowed_ids {
            // Check the known/required ids are a subset of the assertion allowlist
            if !this.required_ids.is_subset(&assertion_allowlist) {
                return Err(StaticResourceMovementsError::AssertionCannotBeSatisfied);
            }
            // Intersect the allow lists
            match &mut this.allowed_ids {
                allowed_ids @ AllowedIds::Any => {
                    *allowed_ids = AllowedIds::Allowlist(assertion_allowlist);
                }
                AllowedIds::Allowlist(allowlist) => {
                    *allowlist = allowlist
                        .intersection(&assertion_allowlist)
                        .cloned()
                        .collect();
                }
            }
        }

        // We've already checked that our certain ids are in the assertion allowlist
        // (and therefore, using the invariant, are in the intersection).
        // We now need to complete processing by expanding the known ids list according to the assertion.
        for required_id in assertion.required_ids.iter() {
            this.required_ids.insert(required_id.clone());
        }

        // Finally, we verify the validity invariants are still upheld.
        // At this point, assuming self and the assertion satisfied the original invariants,
        // given the work we've done above, we know that:
        // * self.required_ids.len() <= self.lower_inclusive
        // * self.required_ids.is_subset(self.allowlist)
        //
        // We still need to check two more which could now have been invalidated:
        // * self.lower_inclusive <= self.upper_inclusive
        // * self.upper_inclusive <= self.allowlist.len()
        if this.lower_bound.equivalent_decimal() > this.upper_bound.equivalent_decimal() {
            return Err(StaticResourceMovementsError::AssertionCannotBeSatisfied);
        }

        if let AllowedIds::Allowlist(allowlist) = &this.allowed_ids {
            if this.upper_bound.equivalent_decimal() > Decimal::from(allowlist.len()) {
                return Err(StaticResourceMovementsError::AssertionCannotBeSatisfied);
            }
        }

        this.normalize();

        Ok(())
    }

    /// For situations where someone has taken an unknown amount from the balance.
    pub fn replace_lower_bounds_with_zero(mut self) -> Self {
        self.mut_replace_lower_bounds_with_zero();
        self
    }

    /// For situations where someone has taken an unknown amount from the balance.
    pub fn mut_replace_lower_bounds_with_zero(&mut self) {
        self.constraints.required_ids = Default::default();
        self.constraints.lower_bound = LowerBound::zero();
    }
}

/// Intended to save all history since the balance was known to be zero.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceChangeHistory(Vec<ResourceChange>);

impl ResourceChangeHistory {
    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn mut_clear(&mut self) {
        self.0.clear();
    }

    pub fn mut_record(&mut self, change: ResourceChange) {
        self.0.push(change);
    }

    pub fn record_take(
        mut self,
        take_amount: ResourceTakeAmount,
        change_source: ChangeSource,
    ) -> Self {
        self.mut_record_take(take_amount, change_source);
        self
    }

    pub fn mut_record_take(
        &mut self,
        take_amount: ResourceTakeAmount,
        change_source: ChangeSource,
    ) {
        self.0.push(ResourceChange::Take {
            take_amount,
            change_source,
        });
    }

    pub fn record_add(
        mut self,
        add_amount: ResourceBounds,
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        self.mut_record_add(add_amount, change_sources);
        self
    }

    pub fn mut_record_add(
        &mut self,
        add_amount: ResourceBounds,
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) {
        self.0.push(ResourceChange::Add {
            add_amount,
            change_sources: change_sources.into_iter().collect(),
        });
    }

    /// It only records a forked `AddWithOwnHistory` if the timeline is non-trivial (i.e. not just a single add).
    /// We try our best to avoid forks, and only fork if we have to
    pub fn mut_record_add_with_history(
        &mut self,
        add_amount: ResourceBounds,
        change_history: ResourceChangeHistory,
    ) {
        match change_history.0.len() {
            0 => {
                // Only exists if add_amount is 0
            }
            1 => {
                let Ok([single_history_item]) = <[ResourceChange; 1]>::try_from(change_history.0)
                else {
                    unreachable!()
                };
                // Only exists if add_amount is Add or AddWithOwnHistory
                self.mut_record(single_history_item);
            }
            _ => {
                if self.0.len() == 0 {
                    *self = change_history
                } else {
                    self.0.push(ResourceChange::AddWithForkedHistory {
                        add_amount,
                        change_history,
                    });
                }
            }
        }
    }

    pub fn record_assertion(
        mut self,
        assertion: ResourceBounds,
        change_source: ChangeSource,
    ) -> Self {
        self.mut_record_assertion(assertion, change_source);
        self
    }

    pub fn mut_record_assertion(&mut self, assertion: ResourceBounds, change_source: ChangeSource) {
        self.0.push(ResourceChange::Assertion {
            assertion,
            change_source,
        })
    }

    pub fn all_changes(&self) -> impl Iterator<Item = &ResourceChange> {
        self.0.iter()
    }

    pub fn all_additive_change_sources_since_was_last_zero(&self) -> IndexSet<ChangeSource> {
        // This could be done more efficiently if we cache the partial totals at each stage.
        let mut cumulative = ResourceBounds::zero();
        let mut all_change_sources: IndexSet<ChangeSource> = Default::default();
        for resource_change in self.all_changes() {
            match resource_change {
                ResourceChange::Add {
                    add_amount,
                    change_sources,
                } => {
                    cumulative.mut_add(add_amount.clone()).unwrap();
                    all_change_sources.extend(change_sources);
                }
                ResourceChange::AddWithForkedHistory {
                    add_amount,
                    change_history,
                } => {
                    cumulative.mut_add(add_amount.clone()).unwrap();
                    all_change_sources
                        .extend(change_history.all_additive_change_sources_since_was_last_zero());
                }
                ResourceChange::Take { take_amount, .. } => {
                    cumulative.mut_take(take_amount.clone()).unwrap();
                }
                ResourceChange::Assertion { assertion, .. } => {
                    cumulative.mut_handle_assertion(assertion.clone()).unwrap();
                }
            }
            if cumulative.is_zero() {
                all_change_sources.clear();
            }
        }
        all_change_sources
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceChange {
    Add {
        add_amount: ResourceBounds,
        change_sources: IndexSet<ChangeSource>,
    },
    AddWithForkedHistory {
        add_amount: ResourceBounds,
        change_history: ResourceChangeHistory,
    },
    Take {
        take_amount: ResourceTakeAmount,
        change_source: ChangeSource,
    },
    Assertion {
        assertion: ResourceBounds,
        change_source: ChangeSource,
    },
}

//====================================================

#[derive(Debug, Clone)]
pub struct StaticResourceMovementsOutput {
    pub invocation_static_information: IndexMap<usize, InvocationStaticInformation>,
}

impl StaticResourceMovementsOutput {
    pub fn resolve_account_changes(
        &self,
    ) -> Result<
        (
            IndexMap<ComponentAddress, NetWithdraws>,
            IndexMap<ComponentAddress, NetDeposits>,
        ),
        StaticResourceMovementsError,
    > {
        let mut aggregated_balance_changes_by_account: IndexMap<
            ComponentAddress,
            AllBalanceChanges,
        > = Default::default();

        for invocation in self.invocation_static_information.values() {
            let Some((account_address, method)) = invocation.as_account_method() else {
                continue;
            };

            match method {
                ACCOUNT_DEPOSIT_IDENT
                | ACCOUNT_DEPOSIT_BATCH_IDENT
                | ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT
                | ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT
                | ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT
                | ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT => {
                    aggregated_balance_changes_by_account
                        .entry(account_address)
                        .or_default()
                        .mut_deposit(invocation.input.clone())?;
                }
                ACCOUNT_WITHDRAW_IDENT
                | ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT
                | ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT
                | ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT => {
                    aggregated_balance_changes_by_account
                        .entry(account_address)
                        .or_default()
                        .mut_withdraw(invocation.output.clone())?;
                }
                _ => {}
            }
        }

        let mut account_net_withdraws = index_map_new();
        let mut account_net_deposits = index_map_new();
        for (account_address, balance_changes) in aggregated_balance_changes_by_account {
            let (withdraws, deposits) = balance_changes.split()?;
            if let Some(withdraws) = withdraws {
                account_net_withdraws.insert(account_address, withdraws);
            }
            if let Some(deposits) = deposits {
                account_net_deposits.insert(account_address, deposits);
            }
        }
        Ok((account_net_withdraws, account_net_deposits))
    }

    pub fn resolve_account_deposits(&self) -> IndexMap<ComponentAddress, Vec<AccountDeposit>> {
        let mut account_deposits: IndexMap<ComponentAddress, Vec<AccountDeposit>> =
            Default::default();

        for invocation in self.invocation_static_information.values() {
            let Some((account_address, method)) = invocation.as_account_method() else {
                continue;
            };

            let account_deposit = match method {
                ACCOUNT_DEPOSIT_IDENT
                | ACCOUNT_DEPOSIT_BATCH_IDENT
                | ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT
                | ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT
                | ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT
                | ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT => {
                    let (specified_resources, unspecified_resources) =
                        invocation.input.clone().normalize().deconstruct();

                    let mut account_deposit = AccountDeposit::empty(unspecified_resources);
                    for (resource_address, tracked_resource) in specified_resources {
                        let (bounds, _history) = tracked_resource.deconstruct();
                        account_deposit = account_deposit.set(resource_address, bounds);
                    }

                    account_deposit
                }
                _ => continue,
            };

            account_deposits
                .entry(account_address)
                .or_default()
                .push(account_deposit);
        }

        account_deposits
    }

    pub fn resolve_account_withdraws(&self) -> IndexMap<ComponentAddress, Vec<AccountWithdraw>> {
        let mut account_withdraws: IndexMap<ComponentAddress, Vec<AccountWithdraw>> =
            Default::default();

        for invocation in self.invocation_static_information.values() {
            let Some((account_address, method)) = invocation.as_account_method() else {
                continue;
            };

            // Filter to only withdraws
            let is_non_fungible_withdraw = match method {
                ACCOUNT_WITHDRAW_IDENT | ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT => false,
                ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT
                | ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT => true,
                _ => continue,
            };

            if invocation.output.unspecified_resources().may_be_present() {
                panic!("Account withdraw output should not have unspecified resources");
            }
            let resources = invocation.output.specified_resources();
            if resources.len() != 1 {
                panic!("Account withdraw output should have exactly one resource");
            }
            let (resource_address, specified_resource) = resources.first().unwrap();
            let account_withdraw = if is_non_fungible_withdraw {
                // Account withdraws are for an exact amount, so we can just use required_ids here
                AccountWithdraw::Ids(
                    *resource_address,
                    specified_resource.bounds().required_ids().clone(),
                )
            } else {
                // Account withdraws are for an exact amount, so the two numeric bounds are equivalent
                AccountWithdraw::Amount(
                    *resource_address,
                    specified_resource
                        .bounds
                        .numeric_bounds()
                        .0
                        .equivalent_decimal(),
                )
            };

            account_withdraws
                .entry(account_address)
                .or_default()
                .push(account_withdraw);
        }

        account_withdraws
    }
}

/// Similar to [`TrackedResources`], but can represent a balance change instead
/// of a tracked bound.
///
/// In the general case, `unspecified_resource_deposits` should support unbounded
/// withdrawal, but we don't need this for resolving account balances.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AllBalanceChanges {
    pub specified_resources: IndexMap<ResourceAddress, AggregatedBalanceChange>,
    pub unspecified_resource_deposits: UnspecifiedResources,
}

impl AllBalanceChanges {
    // Copies [`TrackedResources::mut_add`] - see that method for comments.
    pub fn mut_deposit(
        &mut self,
        deposit: TrackedResources,
    ) -> Result<(), StaticResourceMovementsError> {
        if deposit.unspecified_resources.may_be_present() {
            for (resource, resource_bound) in &mut self.specified_resources {
                if !deposit.specified_resources.contains_key(resource) {
                    resource_bound.mut_deposit(deposit.unspecified_resources.resource_bounds())?;
                }
            }
        }

        for (other_resource, other_resource_bound) in deposit.specified_resources {
            self.aggregated_balance_change_mut(other_resource)
                .mut_deposit(other_resource_bound.bounds)?;
        }

        // Order is important here - we have to handle unspecified after we've handled any specified resources,
        // else the `resource_status_mut` will incorrectly pick up the other's unspecified resources.
        self.unspecified_resource_deposits
            .mut_add(deposit.unspecified_resources);

        Ok(())
    }

    pub fn mut_withdraw(
        &mut self,
        withdrawal: TrackedResources,
    ) -> Result<(), StaticResourceMovementsError> {
        if withdrawal.unspecified_resources.may_be_present() {
            return Err(StaticResourceMovementsError::AggregatedBalanceChangeWithdrawDoesNotSupportUnknownResources);
        }
        for (other_resource, other_resource_bound) in withdrawal.specified_resources {
            self.aggregated_balance_change_mut(other_resource)
                .mut_withdraw(other_resource_bound.bounds)?;
        }
        Ok(())
    }

    pub fn split(
        self,
    ) -> Result<(Option<NetWithdraws>, Option<NetDeposits>), StaticResourceMovementsError> {
        let mut withdraws = index_map_new();
        let mut deposits = index_map_new();

        for (
            address,
            AggregatedBalanceChange {
                deposited,
                withdrawn,
            },
        ) in self.specified_resources
        {
            if !withdrawn.is_zero() {
                withdraws.insert(address, NetWithdraw::from_bounds(address, withdrawn)?);
            }
            if !deposited.is_zero() {
                deposits.insert(address, deposited);
            }
        }

        let net_withdraws = if withdraws.len() > 0 {
            Some(NetWithdraws {
                resources: withdraws,
            })
        } else {
            None
        };

        let new_deposits =
            if deposits.len() > 0 || self.unspecified_resource_deposits.may_be_present() {
                Some(NetDeposits {
                    specified_resources: deposits,
                    unspecified_resources: self.unspecified_resource_deposits,
                })
            } else {
                None
            };

        Ok((net_withdraws, new_deposits))
    }

    /// Works for any resource, specified and unspecified.
    /// If the resource is unspecified, it makes it specified, then returns a reference to the entry.
    fn aggregated_balance_change_mut(
        &mut self,
        resource: ResourceAddress,
    ) -> &mut AggregatedBalanceChange {
        match self.specified_resources.entry(resource) {
            indexmap::map::Entry::Occupied(occupied_entry) => occupied_entry.into_mut(),
            indexmap::map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(AggregatedBalanceChange {
                    deposited: self.unspecified_resource_deposits.resource_bounds(),
                    withdrawn: Default::default(),
                })
            }
        }
    }
}

/// Tracks the total amount of a resource deposited and withdrawn from some container,
/// whilst ensuring that the `required_ids` property of the [`ResourceBounds`] is properly
/// maintained to ensure that any guarantees we have about definite non-fungibles are
/// correct.
///
/// Essentially this means:
/// - Tracking the withdrawn and deposited balances separately
/// - Removing `required_ids` if it can't be ensured
/// - Cancelling definite non-fungible ids
/// - Not cancelling arbitrary numerical bounds, because:
///   - For non-fungibles, these may not refer to the same non-fungibles, so a withdrawal of 1
///     and deposit of 1 might result in swapping a non-fungible for a different one
///   - For fungibles, users may want to see the total deposited and total withdrawn
///
/// Some test cases (note - order matters!):
/// - `Deposit {#5#}, Withdraw 1` => `Deposited 1, Withdrawn 1`
/// - `Withdraw 1, Deposit {#5#}` => `Deposited {#5#}, Withdrawn 1`
/// - `Withdraw {#5#}, Deposit 1` => `Deposited 1, Withdrawn 1`
/// - `Deposit 1, Withdraw {#5#}` => `Deposited 1, Withdrawn {#5#}`
/// - `Withdraw 5, Deposit between 3 and 7` => `Deposited between 3 and 7, Withdrawn 5`
/// - `Withdraw {#2}, Deposit {#2#}, Withdraw {#2}` => `Withdraw {#2}`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AggregatedBalanceChange {
    deposited: ResourceBounds,
    withdrawn: ResourceBounds,
}

impl AggregatedBalanceChange {
    pub fn mut_deposit(
        &mut self,
        mut deposit: ResourceBounds,
    ) -> Result<(), StaticResourceMovementsError> {
        Self::revise(&mut self.withdrawn, &mut deposit)?;
        self.deposited.mut_add(deposit)
    }

    pub fn mut_withdraw(
        &mut self,
        mut withdrawal: ResourceBounds,
    ) -> Result<(), StaticResourceMovementsError> {
        Self::revise(&mut self.deposited, &mut withdrawal)?;
        self.withdrawn.mut_add(withdrawal)
    }

    /// Conceptually, the partial balance is some delta on top of an underlying balance, e.g.:
    /// * For deposits, the partial balance is the net deposits from the manifest, and the underlying
    ///   balance is the pre-existing balance of the account, or other indirect deposits
    /// * For withdraws, the partial balance is the net withdraws from the account in the manifest,
    ///   and the underlying balance is remaining liquidity in the world that could potentially be deposited
    fn revise(
        partial_balance: &mut ResourceBounds,
        debited: &mut ResourceBounds,
    ) -> Result<(), StaticResourceMovementsError> {
        // First, we cancel any shared known ids
        let shared_ids = partial_balance
            .required_ids()
            .intersection(debited.required_ids())
            .cloned()
            .collect::<IndexSet<_>>();
        if shared_ids.len() > 0 {
            partial_balance.mut_take(ResourceTakeAmount::NonFungibles(shared_ids.clone()))?;
            debited.mut_take(ResourceTakeAmount::NonFungibles(shared_ids))?;
        }
        // After this:
        // - Any debited known ids can't be in the partial balance, and have to be in the underlying balance,
        //   so don't affect the partial balance's known ids
        // - If there are any _unknown_ ids in the debit, then these could match ids in the balance, so we have
        //   to wipe it
        //
        // We also clear allowed ids because they don't make sense in this model, and so we don't break normalization.
        if !debited.is_exact_ids() {
            partial_balance.mut_clear_required_ids();
            partial_balance.mut_clear_allowed_ids();
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct NetWithdraws {
    resources: IndexMap<ResourceAddress, NetWithdraw>,
}

impl NetWithdraws {
    pub fn empty() -> Self {
        Default::default()
    }

    pub fn set_fungible(
        mut self,
        resource_address: ResourceAddress,
        total_amount: impl Resolve<Decimal>,
    ) -> Self {
        self.resources.insert(
            resource_address,
            NetWithdraw::Fungible {
                total_amount: total_amount.resolve(),
            },
        );
        self
    }

    pub fn set_non_fungible(
        mut self,
        resource_address: ResourceAddress,
        known_ids: impl IntoIterator<Item = NonFungibleLocalId>,
        additional_unknown_ids: usize,
    ) -> Self {
        self.resources.insert(
            resource_address,
            NetWithdraw::NonFungible {
                known_ids: known_ids.into_iter().collect(),
                additional_unknown_ids,
            },
        );
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetWithdraw {
    Fungible {
        total_amount: Decimal,
    },
    NonFungible {
        known_ids: IndexSet<NonFungibleLocalId>,
        additional_unknown_ids: usize,
    },
}

impl NetWithdraw {
    pub fn from_bounds(
        resource_address: ResourceAddress,
        bounds: ResourceBounds,
    ) -> Result<Self, StaticResourceMovementsError> {
        let Some(total_amount) = bounds.get_exact_amount() else {
            return Err(StaticResourceMovementsError::UnexpectedBoundsForNetWithdraw);
        };
        if resource_address.is_fungible() {
            Ok(Self::Fungible { total_amount })
        } else {
            let total_ids: usize = total_amount
                .try_into()
                .map_err(|_| StaticResourceMovementsError::UnexpectedBoundsForNetWithdraw)?;
            let (known_ids, _, _, _) = bounds.deconstruct();
            // Guaranteed to be safe due to the invariant on ResourceBounds
            let additional_unknown_ids = total_ids - known_ids.len();
            Ok(Self::NonFungible {
                known_ids,
                additional_unknown_ids,
            })
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NetDeposits {
    pub specified_resources: IndexMap<ResourceAddress, ResourceBounds>,
    pub unspecified_resources: UnspecifiedResources,
}

impl NetDeposits {
    pub fn empty(unspecified_resources: UnspecifiedResources) -> Self {
        Self {
            specified_resources: Default::default(),
            unspecified_resources,
        }
    }

    /// Should only be used if it doesn't already exist
    pub fn set(mut self, resource_address: ResourceAddress, bounds: ResourceBounds) -> Self {
        self.specified_resources.insert(resource_address, bounds);
        self
    }

    pub fn specified_resources(&self) -> &IndexMap<ResourceAddress, ResourceBounds> {
        &self.specified_resources
    }

    pub fn simple_specified_resources(&self) -> IndexMap<ResourceAddress, SimpleResourceBounds> {
        self.specified_resources
            .iter()
            .map(|(resource_address, resource_bounds)| {
                let simple_bounds =
                    SimpleResourceBounds::from_bound(*resource_address, resource_bounds.clone());
                (*resource_address, simple_bounds)
            })
            .collect()
    }

    pub fn unspecified_resources(&self) -> UnspecifiedResources {
        self.unspecified_resources.clone()
    }

    pub fn bounds_for(&self, resource_address: ResourceAddress) -> ResourceBounds {
        match self.specified_resources.get(&resource_address) {
            Some(bounds) => bounds.clone(),
            None => self.unspecified_resources.resource_bounds(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountWithdraw {
    Amount(ResourceAddress, Decimal),
    Ids(ResourceAddress, IndexSet<NonFungibleLocalId>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountDeposit {
    specified_resources: IndexMap<ResourceAddress, SimpleResourceBounds>,
    unspecified_resources: UnspecifiedResources,
}

impl AccountDeposit {
    pub fn empty(unspecified_resources: UnspecifiedResources) -> Self {
        Self {
            specified_resources: Default::default(),
            unspecified_resources,
        }
    }

    /// Should only be used if it doesn't already exist
    pub fn set(mut self, resource_address: ResourceAddress, bounds: ResourceBounds) -> Self {
        self.specified_resources.insert(
            resource_address,
            SimpleResourceBounds::from_bound(resource_address, bounds),
        );
        self
    }

    pub fn specified_resources(&self) -> &IndexMap<ResourceAddress, SimpleResourceBounds> {
        &self.specified_resources
    }

    pub fn unspecified_resources(&self) -> UnspecifiedResources {
        self.unspecified_resources.clone()
    }

    pub fn bounds_for(&self, resource_address: ResourceAddress) -> SimpleResourceBounds {
        match self.specified_resources.get(&resource_address) {
            Some(bounds) => bounds.clone(),
            None => SimpleResourceBounds::from_bound(
                resource_address,
                self.unspecified_resources.resource_bounds(),
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InvocationStaticInformation {
    pub kind: OwnedInvocationKind,
    pub input: TrackedResources,
    pub output: TrackedResources,
}

impl InvocationStaticInformation {
    pub fn as_account_method(&self) -> Option<(ComponentAddress, &str)> {
        let InvocationStaticInformation {
            kind:
                OwnedInvocationKind::Method {
                    address: ManifestGlobalAddress::Static(global_address),
                    module_id: ModuleId::Main,
                    method,
                },
            ..
        } = self
        else {
            return None;
        };
        let Ok(component_address) = ComponentAddress::try_from(*global_address) else {
            return None;
        };
        if !component_address.as_node_id().is_global_account() {
            return None;
        }
        Some((component_address, method.as_str()))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum OwnedNextCallAssertion {
    ReturnsOnly {
        constraints: ManifestResourceConstraints,
    },
    ReturnsInclude {
        constraints: ManifestResourceConstraints,
    },
}

impl OwnedNextCallAssertion {
    pub fn as_ref(&self) -> NextCallAssertion {
        match self {
            OwnedNextCallAssertion::ReturnsOnly { constraints } => {
                NextCallAssertion::ReturnsOnly { constraints }
            }
            OwnedNextCallAssertion::ReturnsInclude { constraints } => {
                NextCallAssertion::ReturnsInclude { constraints }
            }
        }
    }
}

impl<'a> From<NextCallAssertion<'a>> for OwnedNextCallAssertion {
    fn from(value: NextCallAssertion<'a>) -> Self {
        match value {
            NextCallAssertion::ReturnsOnly { constraints } => OwnedNextCallAssertion::ReturnsOnly {
                constraints: constraints.clone(),
            },
            NextCallAssertion::ReturnsInclude { constraints } => {
                OwnedNextCallAssertion::ReturnsInclude {
                    constraints: constraints.clone(),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum OwnedInvocationKind {
    Method {
        address: ManifestGlobalAddress,
        module_id: ModuleId,
        method: String,
    },
    Function {
        address: ManifestPackageAddress,
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
pub enum SimpleResourceBounds {
    Fungible(SimpleFungibleResourceBounds),
    NonFungible(SimpleNonFungibleResourceBounds),
}

impl SimpleResourceBounds {
    pub fn from_bound(resource_address: ResourceAddress, resource_bounds: ResourceBounds) -> Self {
        match resource_address.is_fungible() {
            true => Self::Fungible(resource_bounds.into()),
            false => Self::NonFungible(resource_bounds.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimpleFungibleResourceBounds {
    Exact(Decimal),
    AtMost(Decimal),
    AtLeast(Decimal),
    Between(Decimal, Decimal),
    UnknownAmount,
}

impl From<ResourceBounds> for SimpleFungibleResourceBounds {
    fn from(bounds: ResourceBounds) -> Self {
        match bounds.numeric_bounds() {
            (LowerBound::Inclusive(lower_bound_inclusive), UpperBound::Unbounded)
                if lower_bound_inclusive == Decimal::ZERO =>
            {
                Self::UnknownAmount
            }
            (LowerBound::NonZero, UpperBound::Unbounded) => Self::UnknownAmount,
            (LowerBound::NonZero, UpperBound::Inclusive(upper_bound_inclusive)) => {
                Self::AtMost(upper_bound_inclusive)
            }
            (
                LowerBound::Inclusive(lower_bound_inclusive),
                UpperBound::Inclusive(upper_bound_inclusive),
            ) if lower_bound_inclusive.is_zero() => Self::AtMost(upper_bound_inclusive),
            (LowerBound::Inclusive(lower_bound_inclusive), UpperBound::Unbounded) => {
                Self::AtLeast(lower_bound_inclusive)
            }
            (
                LowerBound::Inclusive(lower_bound_inclusive),
                UpperBound::Inclusive(upper_bound_inclusive),
            ) if lower_bound_inclusive == upper_bound_inclusive => {
                Self::Exact(lower_bound_inclusive)
            }
            (
                LowerBound::Inclusive(lower_bound_inclusive),
                UpperBound::Inclusive(upper_bound_inclusive),
            ) => Self::Between(lower_bound_inclusive, upper_bound_inclusive),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimpleNonFungibleResourceBounds {
    Exact {
        amount: Decimal,
        certain_ids: IndexSet<NonFungibleLocalId>,
    },
    NotExact {
        certain_ids: IndexSet<NonFungibleLocalId>,
        lower_bound: LowerBound,
        upper_bound: UpperBound,
        allowed_ids: AllowedIds,
    },
}

impl From<ResourceBounds> for SimpleNonFungibleResourceBounds {
    fn from(bounds: ResourceBounds) -> Self {
        let (certain_ids, lower_bound, upper_bound, allowed_ids) = bounds.deconstruct();
        if Decimal::from(certain_ids.len()) == upper_bound.equivalent_decimal() {
            Self::Exact {
                amount: lower_bound.equivalent_decimal(),
                certain_ids,
            }
        } else {
            Self::NotExact {
                certain_ids,
                lower_bound,
                upper_bound,
                allowed_ids,
            }
        }
    }
}

/// This is a type equivalent to the dynamic types defined for use in the manifest such as the
/// [`DynamicGlobalAddress`], [`DynamicPackageAddress`], and so on but with the [`Named`] variant
/// resolved from a [`ManifestNamedAddress`] to a particular [`BlueprintId`] that is known.
///
/// * [`Named`]: ResolvedDynamicAddress::Named
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ScryptoSbor, ManifestSbor)]
pub enum ResolvedDynamicAddress<T: AsRef<NodeId>> {
    StaticAddress(T),
    BlueprintResolvedFromNamedAddress(BlueprintId),
}

impl<T: AsRef<NodeId>> ResolvedDynamicAddress<T> {
    /// This method returns the [`BlueprintId`] of the main module of the object that it addresses.
    ///
    /// In the case that the address type is [`Named`] then resolving the [`BlueprintId`] is simple,
    /// it  is what's stored in that variant which is the blueprint of the named address. In the
    /// case that the address is [`Static`] this method attempts to determine the [`BlueprintId`]
    /// from from the entity type of the address.
    ///
    /// * [`Named`]: ResolvedDynamicAddress::Named
    /// * [`Static`]: ResolvedDynamicAddress::Static
    pub fn main_module_blueprint_id(&self) -> Option<&BlueprintId> {
        match self {
            Self::StaticAddress(global_address) => global_address
                .as_ref()
                .entity_type()
                .and_then(resolve_main_module_blueprint_id),
            Self::BlueprintResolvedFromNamedAddress(blueprint_id) => Some(blueprint_id),
        }
    }

    /// This method attempts the resolve the dynamic address into the [`BlueprintId`] that is
    /// invoked when the passed [`ModuleId`] is invoked on this address.
    pub fn invoked_blueprint_id(&self, module_id: ModuleId) -> Option<&BlueprintId> {
        self.main_module_blueprint_id()
            .and_then(|main_module_blueprint_id| {
                resolve_invoked_blueprint_id(main_module_blueprint_id, module_id)
            })
    }
}
