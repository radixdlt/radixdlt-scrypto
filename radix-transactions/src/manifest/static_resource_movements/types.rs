use super::*;
use crate::internal_prelude::*;
use indexmap::IndexSet;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::prelude::*;

/// Used for the worktop, and for any instruction input/output.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceBounds {
    bounds: IndexMap<ResourceAddress, ResourceBound>,
    unspecified_resource_sources: IndexSet<ChangeSource>,
}

impl ResourceBounds {
    // Constructors
    pub fn new_empty() -> Self {
        Default::default()
    }

    pub fn new_including_unspecified_resources(
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        Self {
            bounds: Default::default(),
            unspecified_resource_sources: change_sources.into_iter().collect(),
        }
    }

    // Deconstructors
    pub fn deconstruct(
        self,
    ) -> (
        IndexMap<ResourceAddress, ResourceBound>,
        IndexSet<ChangeSource>,
    ) {
        (self.bounds, self.unspecified_resource_sources)
    }

    // &self methods
    pub fn known_resource_bounds(&self) -> &IndexMap<ResourceAddress, ResourceBound> {
        &self.bounds
    }

    pub fn can_include_unspecified_resources(&self) -> bool {
        !self.unspecified_resource_sources.is_empty()
    }

    pub fn unspecified_resource_sources(&self) -> impl Iterator<Item = &ChangeSource> {
        self.unspecified_resource_sources.iter()
    }

    // &mut self methods (check that resource bound aligns with resource type)
    fn resource_bound_mut(&mut self, resource: ResourceAddress) -> &mut ResourceBound {
        match self.bounds.entry(resource) {
            indexmap::map::Entry::Occupied(occupied_entry) => occupied_entry.into_mut(),
            indexmap::map::Entry::Vacant(vacant_entry) => {
                if self.unspecified_resource_sources.is_empty() {
                    vacant_entry.insert(ResourceBound::zero())
                } else {
                    vacant_entry.insert(ResourceBound::zero_or_more(
                        self.unspecified_resource_sources.iter().cloned(),
                    ))
                }
            }
        }
    }

    /// Removes any specific resources whose bounds are identical to the default.
    pub fn normalize(self) -> Self {
        let default_is_any = self.can_include_unspecified_resources();
        let mut unspecified_resource_sources = self.unspecified_resource_sources;
        let mut normalized_bounds: IndexMap<ResourceAddress, ResourceBound> = Default::default();
        for (resource_address, bound) in self.bounds {
            let filter_out = if default_is_any {
                bound.is_any()
            } else {
                bound.is_zero()
            };
            if filter_out {
                if bound.is_any() {
                    unspecified_resource_sources.extend(
                        bound
                            .history
                            .all_additive_change_sources_since_was_last_zero(),
                    );
                }
            } else {
                normalized_bounds.insert(resource_address, bound);
            }
        }

        Self {
            unspecified_resource_sources,
            bounds: normalized_bounds,
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
        self.unspecified_resource_sources
            .extend(change_sources.into_iter());
    }

    pub fn add(&mut self, resources: ResourceBounds) -> Result<(), StaticResourceMovementsError> {
        for (resource, resource_bound) in resources.bounds {
            if resource.is_fungible() && resource_bound.known_ids().len() > 0 {
                return Err(
                    StaticResourceMovementsError::NonFungibleIdsSpecifiedAgainstFungibleResource,
                );
            }
            self.resource_bound_mut(resource).add_from(resource_bound)?;
        }
        self.unspecified_resource_sources
            .extend(resources.unspecified_resource_sources);
        Ok(())
    }

    pub fn add_resource(
        mut self,
        resource: ResourceAddress,
        amount: ResourceBound,
    ) -> Result<Self, StaticResourceMovementsError> {
        self.mut_add_resource(resource, amount)?;
        Ok(self)
    }

    pub fn mut_add_resource(
        &mut self,
        resource: ResourceAddress,
        amount: ResourceBound,
    ) -> Result<(), StaticResourceMovementsError> {
        if resource.is_fungible() && amount.known_ids().len() > 0 {
            return Err(
                StaticResourceMovementsError::NonFungibleIdsSpecifiedAgainstFungibleResource,
            );
        }
        self.resource_bound_mut(resource).add_from(amount)
    }

    pub fn take_resource(
        &mut self,
        resource: ResourceAddress,
        amount: ResourceTakeAmount,
        source: ChangeSource,
    ) -> Result<ResourceBound, StaticResourceMovementsError> {
        self.resource_bound_mut(resource).take(amount, source)
    }

    pub fn take_all(&mut self) -> Self {
        core::mem::take(self)
    }

    pub fn handle_worktop_assertion(
        &mut self,
        worktop_assertion: WorktopAssertion,
        source: ChangeSource,
    ) -> Result<(), StaticResourceMovementsError> {
        // FUTURE TWEAK: Could return an optional set of constraints using all_changes
        match worktop_assertion {
            WorktopAssertion::AnyAmountGreaterThanZero { resource_address } => self
                .resource_bound_mut(*resource_address)
                .handle_assertion(ResourceAssertion::non_zero_amount(), source),
            WorktopAssertion::AtLeastAmount {
                resource_address,
                amount,
            } => self
                .resource_bound_mut(*resource_address)
                .handle_assertion(ResourceAssertion::at_least_amount(amount)?, source),
            WorktopAssertion::AtLeastNonFungibles {
                resource_address,
                ids,
            } => self.resource_bound_mut(*resource_address).handle_assertion(
                ResourceAssertion::at_least_non_fungibles(ids.iter().cloned()),
                source,
            ),
            WorktopAssertion::IsEmpty => {
                for bound in self.bounds.values_mut() {
                    bound.handle_assertion(ResourceAssertion::zero(), source)?;
                }
                self.unspecified_resource_sources.clear();
                Ok(())
            }
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
        amount: impl ResolvableDecimal,
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
}

/// Used to track a known quantity of Fungible and NonFungible resources,
/// for example, the content of a bucket.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceBound {
    lower_inclusive: Decimal,
    upper_inclusive: Decimal,
    /// A maintained invariant is that the number of known ids must be <= the upper bound.
    /// Any take by amount will wipe these, because we don't know which will get taken.
    known_ids: IndexSet<NonFungibleLocalId>,
    history: ResourceChangeHistory,
}

impl ResourceBound {
    // Constructors
    pub fn zero() -> Self {
        Self {
            lower_inclusive: Decimal::zero(),
            upper_inclusive: Decimal::zero(),
            known_ids: Default::default(),
            history: ResourceChangeHistory::empty(),
        }
    }

    pub fn exact_amount(
        amount: impl ResolvableDecimal,
        sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Result<Self, StaticResourceMovementsError> {
        Ok(Self::general(
            ResourceAddAmount::exact_amount(amount)?,
            sources,
        ))
    }

    pub fn at_least_amount(
        amount: impl ResolvableDecimal,
        sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Result<Self, StaticResourceMovementsError> {
        Ok(Self::general(
            ResourceAddAmount::at_least_amount(amount)?,
            sources,
        ))
    }

    pub fn non_zero(sources: impl IntoIterator<Item = ChangeSource>) -> Self {
        Self::at_least_amount(Decimal(I192::ONE), sources).unwrap()
    }

    pub fn zero_or_more(sources: impl IntoIterator<Item = ChangeSource>) -> Self {
        Self::at_least_amount(Decimal::ZERO, sources).unwrap()
    }

    pub fn non_fungibles(
        ids: impl IntoIterator<Item = NonFungibleLocalId>,
        sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        Self::general(ResourceAddAmount::exact_non_fungibles(ids), sources)
    }

    pub fn at_least_non_fungibles(
        ids: impl IntoIterator<Item = NonFungibleLocalId>,
        sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        Self::general(ResourceAddAmount::at_least_non_fungibles(ids), sources)
    }

    pub fn general(
        add_amount: ResourceAddAmount,
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        Self::new_advanced(
            add_amount.clone(),
            ResourceChangeHistory::empty().record_add(add_amount, change_sources),
        )
    }

    /// This is only pub so that it can be used in tests
    pub fn new_advanced(add_amount: ResourceAddAmount, history: ResourceChangeHistory) -> Self {
        Self {
            lower_inclusive: add_amount.lower_inclusive,
            upper_inclusive: add_amount.upper_inclusive,
            known_ids: add_amount.known_ids,
            history,
        }
    }

    // Deconstructors
    pub fn deconstruct(self) -> (ResourceAddAmount, ResourceChangeHistory) {
        (
            ResourceAddAmount {
                lower_inclusive: self.lower_inclusive,
                upper_inclusive: self.upper_inclusive,
                known_ids: self.known_ids,
            },
            self.history,
        )
    }

    // &self methods
    pub fn inclusive_bounds(&self) -> (Decimal, Decimal) {
        (self.lower_inclusive, self.upper_inclusive)
    }

    pub fn known_ids(&self) -> &IndexSet<NonFungibleLocalId> {
        &self.known_ids
    }

    /// Returns true if there is no knowledge about this amount
    pub fn is_any(&self) -> bool {
        self.lower_inclusive == Decimal::ZERO
            && self.upper_inclusive == Decimal::MAX
            && self.known_ids.is_empty()
    }

    /// Returns true if the bound is known to be zero
    pub fn is_zero(&self) -> bool {
        self.lower_inclusive == Decimal::ZERO
            && self.upper_inclusive == Decimal::ZERO
            && self.known_ids.is_empty()
    }

    pub fn history(&self) -> &ResourceChangeHistory {
        &self.history
    }

    // &mut self methods
    pub fn add_from(
        &mut self,
        existing: ResourceBound,
    ) -> Result<(), StaticResourceMovementsError> {
        self.lower_inclusive = self
            .lower_inclusive
            .checked_add(existing.lower_inclusive)
            .ok_or(StaticResourceMovementsError::DecimalOverflow)?;
        self.upper_inclusive = self
            .upper_inclusive
            .saturating_add(existing.upper_inclusive);
        for id in existing.known_ids.into_iter() {
            if !self.known_ids.insert(id) {
                return Err(StaticResourceMovementsError::DuplicateNonFungibleId);
            }
        }
        if self.is_zero() {
            self.history.mut_clear();
        } else {
            self.history.mut_append_history(existing.history);
        }
        Ok(())
    }

    pub fn add(
        &mut self,
        amount: ResourceAddAmount,
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Result<(), StaticResourceMovementsError> {
        self.lower_inclusive = self
            .lower_inclusive
            .checked_add(amount.lower_inclusive)
            .ok_or(StaticResourceMovementsError::DecimalOverflow)?;
        self.upper_inclusive = self.upper_inclusive.saturating_add(amount.upper_inclusive);
        for id in amount.known_ids.iter() {
            if !self.known_ids.insert(id.clone()) {
                return Err(StaticResourceMovementsError::DuplicateNonFungibleId);
            }
        }

        if self.is_zero() {
            self.history.mut_clear();
        } else {
            self.history.mut_record_add(amount, change_sources)
        }

        Ok(())
    }

    pub fn take(
        &mut self,
        amount: ResourceTakeAmount,
        source: ChangeSource,
    ) -> Result<ResourceBound, StaticResourceMovementsError> {
        let taken_amount = match amount.clone() {
            ResourceTakeAmount::Amount(taken_amount) => {
                if taken_amount.is_negative() {
                    return Err(StaticResourceMovementsError::DecimalAmountIsNegative);
                }
                if taken_amount > self.upper_inclusive {
                    return Err(StaticResourceMovementsError::TakeCannotBeSatisfied);
                }
                self.upper_inclusive -= taken_amount;
                self.lower_inclusive = Decimal::zero().max(self.lower_inclusive - taken_amount);
                if taken_amount > Decimal::zero() {
                    // We don't know which ids were taken, so we have to clear them
                    self.known_ids.clear();
                }
                // Taken amount
                ResourceBound::exact_amount(taken_amount, [source])?
            }
            ResourceTakeAmount::NonFungibles(taken_ids) => {
                let taken_count = Decimal::from(taken_ids.len());
                if Decimal::from(taken_ids.len()) > self.upper_inclusive {
                    return Err(StaticResourceMovementsError::TakeCannotBeSatisfied);
                }
                self.upper_inclusive -= taken_count;
                self.lower_inclusive = Decimal::zero().max(self.lower_inclusive - taken_count);
                for taken_non_fungible in taken_ids.iter() {
                    self.known_ids.swap_remove(taken_non_fungible);
                }
                let known_ids_left = self.known_ids.len();
                if Decimal::from(known_ids_left) > self.upper_inclusive {
                    return Err(StaticResourceMovementsError::TakeCannotBeSatisfied);
                }
                // Taken amount
                ResourceBound::non_fungibles(taken_ids, [source])
            }
            ResourceTakeAmount::All => {
                // We don't add history, we just take it
                return Ok(self.take_all());
            }
        };

        if self.is_zero() {
            self.history.mut_clear();
        } else {
            self.history.mut_record_take(amount, source);
        }

        // FUTURE TWEAK: Can output an inequality constraint using history.all_changes()
        Ok(taken_amount)
    }

    pub fn take_all(&mut self) -> Self {
        core::mem::replace(self, Self::zero())
    }

    pub fn handle_assertion(
        &mut self,
        assertion: ResourceAssertion,
        source: ChangeSource,
    ) -> Result<(), StaticResourceMovementsError> {
        // An invariant of the ResourceAssertion is that satisfies the following inequalities:
        // `required_ids.len() <= lower_inclusive <= upper_inclusive`

        // Expand known ids
        for required_id in assertion.required_ids.iter() {
            self.known_ids.insert(required_id.clone());
        }

        // Possibly increase lower bound and decrease upper bound
        self.lower_inclusive = self
            .lower_inclusive
            .max(assertion.lower_inclusive)
            .max(Decimal::from(self.known_ids.len()));
        self.upper_inclusive = self.upper_inclusive.min(assertion.upper_inclusive);

        if self.lower_inclusive > self.upper_inclusive {
            return Err(StaticResourceMovementsError::AssertionCannotBeSatisfied);
        }

        if self.is_zero() {
            self.history.mut_clear();
        } else {
            self.history.mut_record_assertion(assertion, source);
        }

        // FUTURE TWEAK: Can output an inequality constraint using history.all_changes()
        Ok(())
    }
}

/// ## Invariants
/// The following inequalities are upheld by all constructors:
/// * `required_ids.len() <= lower_inclusive <= upper_inclusive`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAssertion {
    lower_inclusive: Decimal,
    upper_inclusive: Decimal,
    required_ids: IndexSet<NonFungibleLocalId>,
}

impl ResourceAssertion {
    pub fn zero_or_more() -> Self {
        Self::at_least_amount(0).unwrap()
    }

    pub fn zero() -> Self {
        Self::exact_amount(0).unwrap()
    }

    pub fn exact_amount(
        amount: impl ResolvableDecimal,
    ) -> Result<Self, StaticResourceMovementsError> {
        let amount = amount.resolve();
        if amount.is_negative() {
            return Err(StaticResourceMovementsError::DecimalAmountIsNegative);
        }
        Ok(Self {
            lower_inclusive: amount,
            upper_inclusive: amount,
            required_ids: Default::default(),
        })
    }

    pub fn at_least_amount(
        amount: impl ResolvableDecimal,
    ) -> Result<Self, StaticResourceMovementsError> {
        let amount = amount.resolve();
        if amount.is_negative() {
            return Err(StaticResourceMovementsError::DecimalAmountIsNegative);
        }
        Ok(Self {
            lower_inclusive: amount,
            upper_inclusive: Decimal::MAX,
            required_ids: Default::default(),
        })
    }

    pub fn non_zero_amount() -> Self {
        Self::at_least_amount(Decimal(I192::ONE)).unwrap()
    }

    pub fn exact_non_fungibles(required_ids: impl IntoIterator<Item = NonFungibleLocalId>) -> Self {
        let required_ids = required_ids.into_iter().collect::<IndexSet<_>>();
        Self {
            lower_inclusive: required_ids.len().into(),
            upper_inclusive: required_ids.len().into(),
            required_ids,
        }
    }

    pub fn at_least_non_fungibles(
        required_ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Self {
        let required_ids = required_ids.into_iter().collect::<IndexSet<_>>();
        Self {
            lower_inclusive: required_ids.len().into(),
            upper_inclusive: Decimal::MAX,
            required_ids,
        }
    }

    pub fn general(
        lower_inclusive: Decimal,
        upper_inclusive: Decimal,
        required_ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Result<Self, StaticResourceMovementsError> {
        let required_ids = required_ids.into_iter().collect::<IndexSet<_>>();
        let required_ids_len = required_ids.len();
        let lower_inclusive = lower_inclusive.max(required_ids_len.into());
        if lower_inclusive > upper_inclusive {
            return Err(StaticResourceMovementsError::AssertionBoundsInvalid);
        }
        Ok(Self {
            lower_inclusive,
            upper_inclusive,
            required_ids,
        })
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
        add_amount: ResourceAddAmount,
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) -> Self {
        self.mut_record_add(add_amount, change_sources);
        self
    }

    pub fn mut_record_add(
        &mut self,
        add_amount: ResourceAddAmount,
        change_sources: impl IntoIterator<Item = ChangeSource>,
    ) {
        self.0.push(ResourceChange::Add {
            add_amount,
            change_sources: change_sources.into_iter().collect(),
        });
    }

    pub fn record_assertion(
        mut self,
        assertion: ResourceAssertion,
        change_source: ChangeSource,
    ) -> Self {
        self.mut_record_assertion(assertion, change_source);
        self
    }

    pub fn mut_record_assertion(
        &mut self,
        assertion: ResourceAssertion,
        change_source: ChangeSource,
    ) {
        self.0.push(ResourceChange::Assertion {
            assertion,
            change_source,
        })
    }

    pub fn mut_append_history(&mut self, change_history: ResourceChangeHistory) {
        self.0.extend(change_history.0)
    }

    pub fn all_changes(&self) -> impl Iterator<Item = &ResourceChange> {
        self.0.iter()
    }

    pub fn all_additive_change_sources_since_was_last_zero(&self) -> IndexSet<ChangeSource> {
        // This could be done more efficiently if we cache the partial totals at each stage.
        let mut cumulative = ResourceBound::zero();
        let mut all_change_sources: IndexSet<ChangeSource> = Default::default();
        for resource_change in self.all_changes() {
            match resource_change {
                ResourceChange::Add {
                    add_amount,
                    change_sources,
                } => {
                    cumulative.add(add_amount.clone(), []).unwrap();
                    all_change_sources.extend(change_sources);
                }
                ResourceChange::Take {
                    take_amount,
                    change_source,
                } => {
                    cumulative
                        .take(take_amount.clone(), change_source.clone())
                        .unwrap();
                }
                ResourceChange::Assertion {
                    assertion,
                    change_source,
                } => {
                    cumulative
                        .handle_assertion(assertion.clone(), change_source.clone())
                        .unwrap();
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
pub struct ResourceAddAmount {
    lower_inclusive: Decimal,
    upper_inclusive: Decimal, // Unbounded = Decimal::MAX and we can use saturating add when adding upper bounds.
    /// A maintained invariant is that the number of known ids must be <= the upper bound.
    /// Any take by amount will wipe these, because we don't know which will get taken.
    known_ids: IndexSet<NonFungibleLocalId>,
}

impl ResourceAddAmount {
    pub fn any() -> Self {
        Self::at_least_amount(0).unwrap()
    }

    pub fn exact_amount(
        amount: impl ResolvableDecimal,
    ) -> Result<Self, StaticResourceMovementsError> {
        let amount = amount.resolve();
        if amount.is_negative() {
            return Err(StaticResourceMovementsError::DecimalAmountIsNegative);
        }
        Ok(Self {
            lower_inclusive: amount,
            upper_inclusive: amount,
            known_ids: Default::default(),
        })
    }

    pub fn at_least_amount(
        amount: impl ResolvableDecimal,
    ) -> Result<Self, StaticResourceMovementsError> {
        let amount = amount.resolve();
        if amount.is_negative() {
            return Err(StaticResourceMovementsError::DecimalAmountIsNegative);
        }
        Ok(Self {
            lower_inclusive: amount,
            upper_inclusive: Decimal::MAX,
            known_ids: Default::default(),
        })
    }

    pub fn non_zero_amount() -> Self {
        Self::at_least_amount(Decimal(I192::ONE)).unwrap()
    }

    pub fn zero_or_more() -> Self {
        Self::at_least_amount(Decimal::ZERO).unwrap()
    }

    pub fn exact_non_fungibles(known_ids: impl IntoIterator<Item = NonFungibleLocalId>) -> Self {
        let known_ids = known_ids.into_iter().collect::<IndexSet<_>>();
        Self {
            lower_inclusive: known_ids.len().into(),
            upper_inclusive: known_ids.len().into(),
            known_ids,
        }
    }

    pub fn at_least_non_fungibles(known_ids: impl IntoIterator<Item = NonFungibleLocalId>) -> Self {
        let known_ids = known_ids.into_iter().collect::<IndexSet<_>>();
        Self {
            lower_inclusive: known_ids.len().into(),
            upper_inclusive: Decimal::MAX,
            known_ids,
        }
    }

    pub fn general(
        lower_inclusive: Decimal,
        upper_inclusive: Decimal,
        known_ids: impl IntoIterator<Item = NonFungibleLocalId>,
    ) -> Result<Self, StaticResourceMovementsError> {
        let known_ids = known_ids.into_iter().collect::<IndexSet<_>>();
        if lower_inclusive > upper_inclusive || Decimal::from(known_ids.len()) > upper_inclusive {
            return Err(StaticResourceMovementsError::AssertionBoundsInvalid);
        }
        Ok(Self {
            lower_inclusive,
            upper_inclusive,
            known_ids,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceChange {
    Add {
        add_amount: ResourceAddAmount,
        change_sources: IndexSet<ChangeSource>,
    },
    Take {
        take_amount: ResourceTakeAmount,
        change_source: ChangeSource,
    },
    Assertion {
        assertion: ResourceAssertion,
        change_source: ChangeSource,
    },
}

//====================================================

#[derive(Debug)]
pub struct StaticResourceMovementsOutput {
    pub invocation_static_information: IndexMap<usize, InvocationStaticInformation>,
}

impl StaticResourceMovementsOutput {
    pub fn account_withdraws(&self) -> IndexMap<ComponentAddress, Vec<AccountWithdraw>> {
        let mut withdrawals: IndexMap<ComponentAddress, Vec<AccountWithdraw>> = Default::default();

        for invocation in self.invocation_static_information.values() {
            let Some((account_address, method)) = invocation.as_account_method() else {
                continue;
            };
            let is_fungible_withdraw = matches!(
                method,
                ACCOUNT_WITHDRAW_IDENT | ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT
            );
            let is_non_fungible_withdraw = matches!(
                method,
                ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT
                    | ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT
            );
            if !(is_fungible_withdraw || is_non_fungible_withdraw) {
                continue;
            }
            let account_withdrawal = {
                if invocation.output.can_include_unspecified_resources() {
                    panic!("Account withdraw output should not have unspecified resources");
                }
                let resources = invocation.output.known_resource_bounds();
                if resources.len() != 1 {
                    panic!("Account withdraw output should have exactly one resource");
                }
                let (resource_address, bound) = resources.first().unwrap();
                if is_non_fungible_withdraw {
                    AccountWithdraw::Ids(*resource_address, bound.known_ids().clone())
                } else {
                    AccountWithdraw::Amount(*resource_address, bound.lower_inclusive)
                }
            };
            withdrawals
                .entry(account_address)
                .or_default()
                .push(account_withdrawal);
        }

        withdrawals
    }

    pub fn account_deposits(&self) -> IndexMap<ComponentAddress, Vec<AccountDeposit>> {
        let mut deposits: IndexMap<ComponentAddress, Vec<AccountDeposit>> = Default::default();

        for invocation in self.invocation_static_information.values() {
            let Some((account_address, method)) = invocation.as_account_method() else {
                continue;
            };

            let is_deposit = matches!(
                method,
                ACCOUNT_DEPOSIT_IDENT
                    | ACCOUNT_DEPOSIT_BATCH_IDENT
                    | ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT
                    | ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT
                    | ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT
                    | ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT
            );

            if !is_deposit {
                continue;
            }

            let account_deposit = AccountDeposit(invocation.input.clone().normalize());

            deposits
                .entry(account_address)
                .or_default()
                .push(account_deposit);
        }

        deposits
    }
}

#[derive(Clone, Debug)]
pub struct InvocationStaticInformation {
    pub kind: OwnedInvocationKind,
    pub input: ResourceBounds,
    pub output: ResourceBounds,
}

impl InvocationStaticInformation {
    pub fn as_account_method(&self) -> Option<(ComponentAddress, &str)> {
        let InvocationStaticInformation {
            kind:
                OwnedInvocationKind::Method {
                    address: DynamicGlobalAddress::Static(global_address),
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
pub struct AccountDeposit(pub ResourceBounds);
