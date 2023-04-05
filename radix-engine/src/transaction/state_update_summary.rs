use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource,
};
use radix_engine_interface::data::scrypto::{model::*, scrypto_decode};
use radix_engine_interface::math::*;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_stores::interface::StateUpdate;
use radix_engine_stores::interface::{StateUpdates, SubstateDatabase};
use sbor::rust::ops::AddAssign;
use sbor::rust::prelude::*;

use crate::system::node_modules::type_info::TypeInfoSubstate;

#[derive(Debug, Clone, ScryptoSbor)]
pub struct StateUpdateSummary {
    pub new_packages: Vec<PackageAddress>,
    pub new_components: Vec<ComponentAddress>,
    pub new_resources: Vec<ResourceAddress>,
    pub balance_changes: IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
    /// This field accounts for two conditions:
    /// 1. Direct vault recalls (and the owner is not loaded during the transaction);
    /// 2. Fee payments for failed transactions.
    pub direct_vault_updates: IndexMap<NodeId, IndexMap<ResourceAddress, BalanceChange>>,
}

impl StateUpdateSummary {
    pub fn new<S: SubstateDatabase>(substate_db: &S, state_updates: &StateUpdates) -> Self {
        let mut new_packages = index_set_new();
        let mut new_components = index_set_new();
        let mut new_resources = index_set_new();
        for (k, StateUpdate::Upsert(_, previous_version)) in &state_updates.substate_changes {
            if previous_version.is_none() {
                if k.0.is_global_package() {
                    new_packages.insert(PackageAddress::new_unchecked(k.0.into()));
                }
                if k.0.is_global_component() {
                    new_components.insert(ComponentAddress::new_unchecked(k.0.into()));
                }
                if k.0.is_global_resource() {
                    new_resources.insert(ResourceAddress::new_unchecked(k.0.into()));
                }
            }
        }

        let (balance_changes, direct_vault_updates) =
            BalanceAccounter::new(substate_db, &state_updates).run();

        StateUpdateSummary {
            new_packages: new_packages.into_iter().collect(),
            new_components: new_components.into_iter().collect(),
            new_resources: new_resources.into_iter().collect(),
            balance_changes,
            direct_vault_updates,
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum BalanceChange {
    Fungible(Decimal),
    NonFungible {
        added: BTreeSet<NonFungibleLocalId>,
        removed: BTreeSet<NonFungibleLocalId>,
    },
}

impl BalanceChange {
    pub fn fungible(&mut self) -> &mut Decimal {
        match self {
            BalanceChange::Fungible(x) => x,
            BalanceChange::NonFungible { .. } => panic!("Not fungible"),
        }
    }
    pub fn added_non_fungibles(&mut self) -> &mut BTreeSet<NonFungibleLocalId> {
        match self {
            BalanceChange::Fungible(..) => panic!("Not non fungible"),
            BalanceChange::NonFungible { added, .. } => added,
        }
    }
    pub fn removed_non_fungibles(&mut self) -> &mut BTreeSet<NonFungibleLocalId> {
        match self {
            BalanceChange::Fungible(..) => panic!("Not non fungible"),
            BalanceChange::NonFungible { removed, .. } => removed,
        }
    }
}

/// Note that the implementation below assumes that substate owned objects can not be
/// detached. If this changes, we will have to account for objects that are removed
/// from a substate.
pub struct BalanceAccounter<'a, 'b> {
    substate_db: &'a dyn SubstateDatabase,
    state_updates_indexed: IndexMap<NodeId, IndexMap<ModuleId, IndexMap<SubstateKey, &'b Vec<u8>>>>,
}

impl<'a, 'b> BalanceAccounter<'a, 'b> {
    pub fn new(substate_db: &'a dyn SubstateDatabase, state_updates: &'b StateUpdates) -> Self {
        let mut state_updates_indexed: IndexMap<
            NodeId,
            IndexMap<ModuleId, IndexMap<SubstateKey, &'b Vec<u8>>>,
        > = index_map_new();
        for ((node_id, module_id, substate_key), change) in &state_updates.substate_changes {
            state_updates_indexed
                .entry(*node_id)
                .or_default()
                .entry(*module_id)
                .or_default()
                .insert(
                    substate_key.clone(),
                    match &change {
                        StateUpdate::Upsert(substate_value, ..) => substate_value,
                    },
                );
        }

        Self {
            substate_db,
            state_updates_indexed,
        }
    }

    pub fn run(
        &self,
    ) -> (
        IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
        IndexMap<NodeId, IndexMap<ResourceAddress, BalanceChange>>,
    ) {
        let mut balance_changes = index_map_new();
        let mut direct_vault_updates: IndexMap<NodeId, IndexMap<ResourceAddress, BalanceChange>> =
            index_map_new();
        let mut accounted_vaults = index_set_new();

        self.state_updates_indexed
            .keys()
            .filter_map(|x| GlobalAddress::try_from(x.as_ref()).ok())
            .for_each(|root| {
                self.traverse_state_updates(
                    &mut balance_changes,
                    &mut accounted_vaults,
                    &root,
                    root.as_node_id(),
                )
            });

        self.state_updates_indexed
            .keys()
            .filter(|x| x.is_internal_vault() && !accounted_vaults.contains(*x))
            .for_each(|vault_node_id| {
                if let Some((resource_address, balance_change)) =
                    self.calculate_vault_balance_change(vault_node_id)
                {
                    match balance_change {
                        BalanceChange::Fungible(delta) => {
                            let existing = direct_vault_updates
                                .entry(*vault_node_id)
                                .or_default()
                                .entry(resource_address)
                                .or_insert(BalanceChange::Fungible(Decimal::ZERO))
                                .fungible();
                            existing.add_assign(delta);
                        }
                        BalanceChange::NonFungible { added, removed } => {
                            let existing = direct_vault_updates
                                .entry(*vault_node_id)
                                .or_default()
                                .entry(resource_address)
                                .or_insert(BalanceChange::NonFungible {
                                    added: BTreeSet::new(),
                                    removed: BTreeSet::new(),
                                });
                            existing.added_non_fungibles().extend(added);
                            existing.removed_non_fungibles().extend(removed);
                        }
                    }
                }
            });

        // prune balance changes

        balance_changes.retain(|_, map| {
            map.retain(|_, change| match change {
                BalanceChange::Fungible(delta) => !delta.is_zero(),
                BalanceChange::NonFungible { added, removed } => {
                    added.retain(|x| !removed.contains(x));
                    removed.retain(|x| !added.contains(x));
                    !added.is_empty() || !removed.is_empty()
                }
            });
            !map.is_empty()
        });

        direct_vault_updates.retain(|_, map| {
            map.retain(|_, change| match change {
                BalanceChange::Fungible(delta) => !delta.is_zero(),
                BalanceChange::NonFungible { added, removed } => {
                    added.retain(|x| !removed.contains(x));
                    removed.retain(|x| !added.contains(x));
                    !added.is_empty() || !removed.is_empty()
                }
            });
            !map.is_empty()
        });

        (balance_changes, direct_vault_updates)
    }

    fn traverse_state_updates(
        &self,
        balance_changes: &mut IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
        accounted_vaults: &mut IndexSet<NodeId>,
        root: &GlobalAddress,
        current_node: &NodeId,
    ) -> () {
        if let Some(modules) = self.state_updates_indexed.get(current_node) {
            if current_node.is_internal_vault() {
                accounted_vaults.insert(current_node.clone());

                if let Some((resource_address, balance_change)) =
                    self.calculate_vault_balance_change(current_node)
                {
                    match balance_change {
                        BalanceChange::Fungible(delta) => {
                            let existing = balance_changes
                                .entry(*root)
                                .or_default()
                                .entry(resource_address)
                                .or_insert(BalanceChange::Fungible(Decimal::ZERO))
                                .fungible();
                            existing.add_assign(delta);
                        }
                        BalanceChange::NonFungible { added, removed } => {
                            let existing = balance_changes
                                .entry(*root)
                                .or_default()
                                .entry(resource_address)
                                .or_insert(BalanceChange::NonFungible {
                                    added: BTreeSet::new(),
                                    removed: BTreeSet::new(),
                                });
                            existing.added_non_fungibles().extend(added);
                            existing.removed_non_fungibles().extend(removed);
                        }
                    }
                }
            } else {
                // Scan loaded substates to find children
                for (_module_id, module_substates) in modules {
                    for (_substate_key, update) in module_substates {
                        let substate_value = IndexedScryptoValue::from_slice(update.as_ref())
                            .expect("Failed to decode substate");
                        for own in substate_value.owned_node_ids() {
                            self.traverse_state_updates(
                                balance_changes,
                                accounted_vaults,
                                root,
                                own,
                            );
                        }
                    }
                }
            }
        }
    }

    fn calculate_vault_balance_change(
        &self,
        node_id: &NodeId,
    ) -> Option<(ResourceAddress, BalanceChange)> {
        let type_info: TypeInfoSubstate = scrypto_decode(
            &self
                .fetch_substate(
                    node_id,
                    TypedModuleId::TypeInfo.into(),
                    &TypeInfoOffset::TypeInfo.into(),
                )
                .expect("Missing vault info"),
        )
        .expect("Failed to decode vault info");

        let resource_address = match type_info {
            TypeInfoSubstate::Object(ObjectInfo {
                type_parent: Some(x),
                ..
            }) => ResourceAddress::new_unchecked(x.into()),
            _ => panic!("Unexpected"),
        };

        if resource_address.as_node_id().is_global_fungible_resource() {
            // If there is an update to the liquid resource
            if let Some(substate) = self.fetch_substate_from_state_updates(
                node_id,
                TypedModuleId::ObjectState.into(),
                &VaultOffset::LiquidFungible.into(),
            ) {
                let old_substate = self.fetch_substate_from_database(
                    node_id,
                    TypedModuleId::ObjectState.into(),
                    &VaultOffset::LiquidFungible.into(),
                );

                let old_balance = if let Some(s) = old_substate {
                    scrypto_decode::<LiquidFungibleResource>(&s)
                        .unwrap()
                        .amount()
                } else {
                    Decimal::ZERO
                };
                let new_balance = scrypto_decode::<LiquidFungibleResource>(substate)
                    .unwrap()
                    .amount();

                Some(BalanceChange::Fungible(new_balance - old_balance))
            } else {
                None
            }
        } else {
            // If there is an update to the liquid resource
            if let Some(substate) = self.fetch_substate_from_state_updates(
                node_id,
                TypedModuleId::ObjectState.into(),
                &VaultOffset::LiquidNonFungible.into(),
            ) {
                let old_substate = self.fetch_substate_from_database(
                    node_id,
                    TypedModuleId::ObjectState.into(),
                    &VaultOffset::LiquidNonFungible.into(),
                );

                let mut old_balance = if let Some(s) = old_substate {
                    scrypto_decode::<LiquidNonFungibleResource>(&s)
                        .unwrap()
                        .into_ids()
                } else {
                    BTreeSet::new()
                };
                let mut new_balance = scrypto_decode::<LiquidNonFungibleResource>(substate)
                    .unwrap()
                    .into_ids();

                remove_intersection(&mut new_balance, &mut old_balance);

                Some(BalanceChange::NonFungible {
                    added: new_balance,
                    removed: old_balance,
                })
            } else {
                None
            }
        }
        .map(|x| (resource_address, x))
    }

    fn fetch_substate(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<Vec<u8>> {
        // TODO: we should not need to load substates form substate database
        // - Part of the engine still reads/writes substates without touching the TypeInfo;
        // - Track does not store the initial value of substate.

        self.fetch_substate_from_state_updates(node_id, module_id, substate_key)
            .map(|x| x.clone())
            .or_else(|| self.fetch_substate_from_database(node_id, module_id, substate_key))
    }

    fn fetch_substate_from_database(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<Vec<u8>> {
        self.substate_db
            .get_substate(node_id, module_id, substate_key)
            .expect("Database misconfigured")
            .map(|x| x.0)
    }

    fn fetch_substate_from_state_updates(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<&Vec<u8>> {
        self.state_updates_indexed
            .get(node_id)
            .and_then(|x| x.get(&module_id))
            .and_then(|x| x.get(substate_key).cloned())
    }
}

/// Removes the `left.intersection(right)` from both `left` and `right`, in place, without
/// computing (or allocating) the intersection itself.
/// Implementation note: since Rust has no "iterator with delete" capabilities, the implementation
/// uses a (normally frowned-upon) side-effect of a lambda inside `.retain()`.
/// Performance note: since the `BTreeSet`s are inherently sorted, the implementation _could_ have
/// an `O(n+m)` runtime (i.e. traversing 2 iterators). However, it would then contain significantly
/// more bugs than the `O(n * log(m))` one-liner below.
fn remove_intersection<T: Ord>(left: &mut BTreeSet<T>, right: &mut BTreeSet<T>) {
    left.retain(|id| !right.remove(id));
}
