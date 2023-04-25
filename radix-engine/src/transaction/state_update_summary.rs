use radix_engine_common::data::scrypto::ScryptoDecode;
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleVault,
};
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::math::*;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_stores::interface::{DatabaseMapper, SubstateDatabase};
use radix_engine_stores::jmt_support::JmtMapper;
use sbor::rust::ops::AddAssign;
use sbor::rust::prelude::*;

use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::track::{TrackedNode, TrackedSubstateKey, Write};

#[derive(Debug, Clone, ScryptoSbor)]
pub struct StateUpdateSummary {
    pub new_packages: Vec<PackageAddress>,
    pub new_components: Vec<ComponentAddress>,
    pub new_resources: Vec<ResourceAddress>,
    pub balance_changes: IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
    /// This field accounts for Direct vault recalls (and the owner is not loaded during the transaction);
    pub direct_vault_updates: IndexMap<NodeId, IndexMap<ResourceAddress, BalanceChange>>,
}

impl StateUpdateSummary {
    pub fn new<S: SubstateDatabase>(
        substate_db: &S,
        updates: &IndexMap<NodeId, TrackedNode>,
    ) -> Self {
        let mut new_packages = index_set_new();
        let mut new_components = index_set_new();
        let mut new_resources = index_set_new();

        for (node_id, tracked) in updates {
            if tracked.is_new {
                if node_id.is_global_package() {
                    new_packages.insert(PackageAddress::new_unchecked(node_id.0));
                }
                if node_id.is_global_component() {
                    new_components.insert(ComponentAddress::new_unchecked(node_id.0));
                }
                if node_id.is_global_resource() {
                    new_resources.insert(ResourceAddress::new_unchecked(node_id.0));
                }
            }
        }

        let (balance_changes, direct_vault_updates) =
            BalanceAccounter::new(substate_db, &updates).run();

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
pub struct BalanceAccounter<'a, S: SubstateDatabase> {
    substate_db: &'a S,
    tracked: &'a IndexMap<NodeId, TrackedNode>,
}

impl<'a, S: SubstateDatabase> BalanceAccounter<'a, S> {
    pub fn new(substate_db: &'a S, tracked: &'a IndexMap<NodeId, TrackedNode>) -> Self {
        Self {
            substate_db,
            tracked,
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

        self.tracked
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

        self.tracked
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
        if let Some(tracked_node) = self.tracked.get(current_node) {
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
                for (_module_id, tracked_module) in &tracked_node.tracked_modules {
                    for (_substate_key, tracked_key) in &tracked_module.substates {
                        if let Some(value) = tracked_key.get() {
                            for own in value.owned_node_ids() {
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
    }

    fn calculate_vault_balance_change(
        &self,
        node_id: &NodeId,
    ) -> Option<(ResourceAddress, BalanceChange)> {
        let type_info: TypeInfoSubstate = self
            .fetch_substate::<JmtMapper, TypeInfoSubstate>(
                node_id,
                SysModuleId::TypeInfo.into(),
                TypeInfoOffset::TypeInfo.into(),
            )
            .expect("Missing vault info");

        let resource_address = match type_info {
            TypeInfoSubstate::Object(ObjectInfo {
                type_parent: Some(x),
                ..
            }) => ResourceAddress::new_unchecked(x.into()),
            _ => panic!("Unexpected"),
        };

        if resource_address.as_node_id().is_global_fungible_resource() {
            // If there is an update to the liquid resource
            if let Some(substate) = self
                .fetch_substate_from_state_updates::<JmtMapper, LiquidFungibleResource>(
                    node_id,
                    SysModuleId::Object.into(),
                    FungibleVaultOffset::LiquidFungible.into(),
                )
            {
                let old_substate = self
                    .fetch_substate_from_database::<JmtMapper, LiquidFungibleResource>(
                        node_id,
                        SysModuleId::Object.into(),
                        FungibleVaultOffset::LiquidFungible.into(),
                    );

                let old_balance = if let Some(s) = old_substate {
                    s.amount()
                } else {
                    Decimal::ZERO
                };
                let new_balance = substate.amount();

                Some(BalanceChange::Fungible(new_balance - old_balance))
            } else {
                None
            }
        } else {
            // If there is an update to the liquid resource
            if let Some(vault) = self
                .fetch_substate_from_state_updates::<JmtMapper, LiquidNonFungibleVault>(
                    node_id,
                    SysModuleId::Object.into(),
                    NonFungibleVaultOffset::LiquidNonFungible.into(),
                )
            {
                let vault_updates = self.tracked.get(vault.ids.as_node_id()).and_then(|n| {
                    let module_id: ModuleId = SysModuleId::Object.into();
                    n.tracked_modules.get(&module_id)
                });

                if let Some(tracked_module) = vault_updates {
                    let mut added = BTreeSet::new();
                    let mut removed = BTreeSet::new();

                    for (_key, tracked) in &tracked_module.substates {
                        match tracked {
                            TrackedSubstateKey::New(substate)
                            | TrackedSubstateKey::ReadNonExistAndWrite(substate) => {
                                let id: NonFungibleLocalId = substate.value.as_typed().unwrap();
                                added.insert(id);
                            }
                            TrackedSubstateKey::ReadExistAndWrite(old, write) => match write {
                                Write::Update(..) => {}
                                Write::Delete => {
                                    let id: NonFungibleLocalId = old.as_typed().unwrap();
                                    removed.insert(id);
                                }
                            },
                            TrackedSubstateKey::WriteOnly(write) => match write {
                                Write::Update(substate) => {
                                    let id: NonFungibleLocalId = substate.value.as_typed().unwrap();
                                    added.insert(id);
                                }
                                Write::Delete => {
                                    panic!("Should never occur");
                                }
                            },
                            TrackedSubstateKey::ReadOnly(..) | TrackedSubstateKey::Garbage => {}
                        }
                    }

                    Some(BalanceChange::NonFungible { added, removed })
                } else {
                    None
                }
            } else {
                None
            }
        }
        .map(|x| (resource_address, x))
    }

    fn fetch_substate<M: DatabaseMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        key: SubstateKey,
    ) -> Option<D> {
        // TODO: we should not need to load substates form substate database
        // - Part of the engine still reads/writes substates without touching the TypeInfo;
        // - Track does not store the initial value of substate.

        self.fetch_substate_from_state_updates::<M, D>(node_id, module_id, key.clone())
            .or_else(|| self.fetch_substate_from_database::<M, D>(node_id, module_id, key))
    }

    fn fetch_substate_from_database<M: DatabaseMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        key: SubstateKey,
    ) -> Option<D> {
        self.substate_db
            .read_mapped_substate::<M, D>(node_id, module_id, key)
    }

    fn fetch_substate_from_state_updates<M: DatabaseMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        key: SubstateKey,
    ) -> Option<D> {
        self.tracked
            .get(node_id)
            .and_then(|tracked_node| tracked_node.tracked_modules.get(&module_id))
            .and_then(|tracked_module| tracked_module.substates.get(&M::map_to_db_key(key)))
            .and_then(|tracked_key| tracked_key.get().map(|e| e.as_typed().unwrap()))
    }
}
