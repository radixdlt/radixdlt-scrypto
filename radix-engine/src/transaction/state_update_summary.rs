use crate::blueprints::resource::{FungibleVaultBalanceFieldSubstate, FungibleVaultField};
use crate::internal_prelude::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::math::*;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_store_interface::{
    db_key_mapper::SpreadPrefixKeyMapper, interface::SubstateDatabase,
};
use sbor::rust::prelude::*;

use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system_db_reader::SystemDatabaseReader;
use crate::track::TrackedSubstateValue;
use crate::track::{TrackedNode, Write};

#[derive(Default, Debug, Clone, ScryptoSbor)]
pub struct StateUpdateSummary {
    pub new_packages: IndexSet<PackageAddress>,
    pub new_components: IndexSet<ComponentAddress>,
    pub new_resources: IndexSet<ResourceAddress>,
    pub new_vaults: IndexSet<InternalAddress>,
    /// TODO: remove
    pub balance_changes: IndexMap<GlobalAddress, IndexMap<ResourceAddress, BalanceChange>>,
    /// This field accounts for Direct vault recalls (and the owner is not loaded during the transaction);
    /// TODO: remove
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
        let mut new_vaults = index_set_new();

        for (node_id, tracked) in updates {
            if tracked.is_new {
                if node_id.is_global_package() {
                    new_packages.insert(PackageAddress::new_or_panic(node_id.0));
                }
                if node_id.is_global_component() {
                    new_components.insert(ComponentAddress::new_or_panic(node_id.0));
                }
                if node_id.is_global_resource_manager() {
                    new_resources.insert(ResourceAddress::new_or_panic(node_id.0));
                }
                if node_id.is_internal_vault() {
                    new_vaults.insert(InternalAddress::new_or_panic(node_id.0));
                }
            }
        }

        let (balance_changes, direct_vault_updates) =
            BalanceAccounter::new(substate_db, &updates).run();

        StateUpdateSummary {
            new_packages,
            new_components,
            new_resources,
            new_vaults,
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
    system_reader: SystemDatabaseReader<'a, S>,
    tracked: &'a IndexMap<NodeId, TrackedNode>,
}

impl<'a, S: SubstateDatabase> BalanceAccounter<'a, S> {
    pub fn new(substate_db: &'a S, tracked: &'a IndexMap<NodeId, TrackedNode>) -> Self {
        Self {
            system_reader: SystemDatabaseReader::new_with_overlay(substate_db, tracked),
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
                            *existing = existing.safe_add(delta).unwrap();
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
                            *existing = existing.safe_add(delta).unwrap();
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
                for tracked_module in tracked_node.tracked_partitions.values() {
                    for tracked_key in tracked_module.substates.values() {
                        if let Some(value) = tracked_key.substate_value.get() {
                            for own in value.owned_nodes() {
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
            .system_reader
            .fetch_substate::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
                node_id,
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
            )
            .expect("Missing vault info");

        let resource_address = match type_info {
            TypeInfoSubstate::Object(info) => {
                ResourceAddress::new_or_panic(info.get_outer_object().into())
            }
            _ => panic!("Unexpected"),
        };

        if resource_address
            .as_node_id()
            .is_global_fungible_resource_manager()
        {
            // If there is an update to the liquid resource
            if let Some(substate) = self
                .system_reader
                .fetch_substate_from_state_updates::<SpreadPrefixKeyMapper, FungibleVaultBalanceFieldSubstate>(
                    node_id,
                    MAIN_BASE_PARTITION,
                    &FungibleVaultField::Balance.into(),
                )
            {
                let old_substate = self
                    .system_reader
                    .fetch_substate_from_database::<SpreadPrefixKeyMapper, FungibleVaultBalanceFieldSubstate>(
                        node_id,
                        MAIN_BASE_PARTITION,
                        &FungibleVaultField::Balance.into(),
                    );

                let old_balance = if let Some(s) = old_substate {
                    s.into_payload().into_latest().amount()
                } else {
                    Decimal::ZERO
                };
                let new_balance = substate.into_payload().into_latest().amount();

                Some(BalanceChange::Fungible(new_balance.safe_sub(old_balance).unwrap()))
            } else {
                None
            }
        } else {
            // If there is an update to the liquid resource

            let vault_updates = self.tracked.get(node_id).and_then(|n| {
                n.tracked_partitions
                    .get(&MAIN_BASE_PARTITION.at_offset(PartitionOffset(1u8)).unwrap())
            });

            if let Some(tracked_module) = vault_updates {
                let mut added = BTreeSet::new();
                let mut removed = BTreeSet::new();

                for (_db_sort_key,  tracked_substate) in &tracked_module.substates {
                    let id: NonFungibleLocalId = scrypto_decode(tracked_substate.substate_key.for_map().unwrap()).unwrap();

                    match &tracked_substate.substate_value {
                        TrackedSubstateValue::New(..)
                        | TrackedSubstateValue::ReadNonExistAndWrite(..) => {
                            added.insert(id);
                        }
                        TrackedSubstateValue::ReadExistAndWrite(_, write) => match write {
                            Write::Update(..) => {}
                            Write::Delete => {
                                removed.insert(id);
                            }
                        },
                        TrackedSubstateValue::WriteOnly(write) => match write {
                            Write::Update(..) => {
                                added.insert(id);
                            }
                            Write::Delete => {
                                // This may occur if a non fungible is added then removed from the same vault
                            }
                        },
                        TrackedSubstateValue::ReadOnly(..) | TrackedSubstateValue::Garbage => {}
                    }
                }

                Some(BalanceChange::NonFungible { added, removed })
            } else {
                None
            }
        }
        .map(|x| (resource_address, x))
    }
}
