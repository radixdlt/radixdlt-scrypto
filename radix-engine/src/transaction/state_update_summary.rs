use radix_engine_common::prelude::scrypto_decode;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::math::*;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_store_interface::{
    db_key_mapper::SpreadPrefixKeyMapper, interface::SubstateDatabase,
};
use sbor::rust::ops::Add;
use sbor::rust::prelude::*;

use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system::FieldSubstate;
use crate::system::system_db_reader::SystemDatabaseReader;
use crate::track::TrackedSubstateValue;
use crate::track::{TrackedNode, Write};

#[derive(Default, Debug, Clone, ScryptoSbor)]
pub struct StateUpdateSummary {
    pub new_packages: IndexSet<PackageAddress>,
    pub new_components: IndexSet<ComponentAddress>,
    pub new_resources: IndexSet<ResourceAddress>,
    pub new_vaults: IndexSet<InternalAddress>,
    pub vault_balance_changes: IndexMap<NodeId, (ResourceAddress, BalanceChange)>,
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

        let vault_balance_changes = BalanceAccounter::new(substate_db, &updates).run();

        StateUpdateSummary {
            new_packages,
            new_components,
            new_resources,
            new_vaults,
            vault_balance_changes,
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

impl Add for BalanceChange {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        match &mut self {
            BalanceChange::Fungible(self_value) => {
                let BalanceChange::Fungible(value) = rhs else {
                    panic!("cannot {:?} + {:?}", self, rhs);
                };
                *self_value = self_value.safe_add(value).unwrap();
            }
            BalanceChange::NonFungible {
                added: self_added,
                removed: self_removed,
            } => {
                let BalanceChange::NonFungible { added, removed } = rhs else {
                    panic!("cannot {:?} + {:?}", self, rhs);
                };
                self_added.extend(added);
                self_removed.extend(removed);
            }
        }
        self
    }
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

    pub fn run(&self) -> IndexMap<NodeId, (ResourceAddress, BalanceChange)> {
        self.tracked
            .keys()
            .filter(|node_id| node_id.is_internal_vault())
            .filter_map(|vault_id| {
                self.calculate_vault_balance_change(vault_id)
                    .map(|change| (*vault_id, change))
            })
            .collect::<IndexMap<_, _>>()
    }

    fn calculate_vault_balance_change(
        &self,
        vault_id: &NodeId,
    ) -> Option<(ResourceAddress, BalanceChange)> {
        let type_info: TypeInfoSubstate = self
            .system_reader
            .fetch_substate::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
                vault_id,
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

        let is_fungible = resource_address
            .as_node_id()
            .is_global_fungible_resource_manager();
        let change = if is_fungible {
            self.calculate_fungible_vault_balance_change(vault_id)
        } else {
            self.calculate_non_fungible_vault_balance_change(vault_id)
        };

        change.map(|change| (resource_address, change))
    }

    fn calculate_fungible_vault_balance_change(&self, vault_id: &NodeId) -> Option<BalanceChange> {
        self
            .system_reader
            .fetch_substate_from_state_updates::<SpreadPrefixKeyMapper, FieldSubstate<LiquidFungibleResource>>(
                vault_id,
                MAIN_BASE_PARTITION,
                &FungibleVaultField::LiquidFungible.into(),
            )
            .map(|new_substate| new_substate.value.0.amount())
            .map(|new_balance| {
                let old_balance = self
                    .system_reader
                    .fetch_substate_from_database::<SpreadPrefixKeyMapper, FieldSubstate<LiquidFungibleResource>>(
                        vault_id,
                        MAIN_BASE_PARTITION,
                        &FungibleVaultField::LiquidFungible.into(),
                    )
                    .map(|old_balance| old_balance.value.0.amount())
                    .unwrap_or(Decimal::ZERO);

                new_balance.safe_sub(old_balance).unwrap()
            })
            .filter(|change| change != &Decimal::ZERO) // prune
            .map(|change| BalanceChange::Fungible(change))
    }

    fn calculate_non_fungible_vault_balance_change(
        &self,
        vault_id: &NodeId,
    ) -> Option<BalanceChange> {
        self.tracked
            .get(vault_id)
            .and_then(|vault_node| {
                vault_node
                    .tracked_partitions
                    .get(&MAIN_BASE_PARTITION.at_offset(PartitionOffset(1u8)).unwrap())
            })
            .map(|vault_updates| {
                let mut added = BTreeSet::new();
                let mut removed = BTreeSet::new();

                for tracked_substate in vault_updates.substates.values() {
                    let id: NonFungibleLocalId =
                        scrypto_decode(tracked_substate.substate_key.for_map().unwrap()).unwrap();

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

                (added, removed)
            })
            .map(|(mut added, mut removed)| {
                // prune
                let cancelled_out = added
                    .intersection(&removed)
                    .cloned()
                    .collect::<BTreeSet<_>>();
                added.retain(|id| !cancelled_out.contains(id));
                removed.retain(|id| !cancelled_out.contains(id));
                (added, removed)
            })
            .filter(|(added, removed)| !added.is_empty() || !removed.is_empty())
            .map(|(added, removed)| BalanceChange::NonFungible { added, removed })
    }
}
