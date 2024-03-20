use crate::blueprints::resource::{FungibleVaultBalanceFieldPayload, FungibleVaultField};
use crate::internal_prelude::*;
use crate::system::system_db_reader::SystemDatabaseReader;
use crate::track::TrackedSubstateValue;
use crate::track::{TrackedNode, Write};
use radix_common::data::scrypto::model::*;
use radix_common::math::*;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_substate_store_interface::{
    db_key_mapper::SpreadPrefixKeyMapper, interface::SubstateDatabase,
};
use sbor::rust::prelude::*;

#[derive(Default, Debug, Clone, ScryptoSbor, PartialEq, Eq)]
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

impl AddAssign for BalanceChange {
    fn add_assign(&mut self, rhs: Self) {
        match self {
            BalanceChange::Fungible(self_value) => {
                let BalanceChange::Fungible(value) = rhs else {
                    panic!("cannot {:?} + {:?}", self, rhs);
                };
                *self_value = self_value.checked_add(value).unwrap();
            }
            BalanceChange::NonFungible {
                added: self_added,
                removed: self_removed,
            } => {
                let BalanceChange::NonFungible { added, removed } = rhs else {
                    panic!("cannot {:?} + {:?}", self, rhs);
                };

                for remove in removed {
                    if !self_added.remove(&remove) {
                        self_removed.insert(remove);
                    }
                }

                for add in added {
                    if !self_removed.remove(&add) {
                        self_added.insert(add);
                    }
                }
            }
        }
    }
}

impl BalanceChange {
    pub fn prune_and_check_if_zero(&mut self) -> bool {
        match self {
            BalanceChange::Fungible(x) => x.is_zero(),
            BalanceChange::NonFungible { added, removed } => {
                let cancelled_out = added
                    .intersection(&removed)
                    .cloned()
                    .collect::<BTreeSet<_>>();
                added.retain(|id| !cancelled_out.contains(id));
                removed.retain(|id| !cancelled_out.contains(id));

                added.is_empty() && removed.is_empty()
            }
        }
    }

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
        let object_info = self
            .system_reader
            .get_object_info(*vault_id)
            .expect("Missing vault info");

        let resource_address = ResourceAddress::new_or_panic(object_info.get_outer_object().into());

        let change = if resource_address.is_fungible() {
            self.calculate_fungible_vault_balance_change(vault_id)
        } else {
            self.calculate_non_fungible_vault_balance_change(vault_id)
        };

        change.map(|change| (resource_address, change))
    }

    fn calculate_fungible_vault_balance_change(&self, vault_id: &NodeId) -> Option<BalanceChange> {
        self
            .system_reader
            .fetch_substate_from_state_updates::<SpreadPrefixKeyMapper, FieldSubstate<FungibleVaultBalanceFieldPayload>>(
                vault_id,
                MAIN_BASE_PARTITION,
                &FungibleVaultField::Balance.into(),
            )
            .map(|new_substate| new_substate.into_payload().into_latest().amount())
            .map(|new_balance| {
                let old_balance = self
                    .system_reader
                    .fetch_substate_from_database::<SpreadPrefixKeyMapper, FieldSubstate<FungibleVaultBalanceFieldPayload>>(
                        vault_id,
                        MAIN_BASE_PARTITION,
                        &FungibleVaultField::Balance.into(),
                    )
                    .map(|old_balance| old_balance.into_payload().into_latest().amount())
                    .unwrap_or(Decimal::ZERO);

                // TODO: Handle potential Decimal arithmetic operation (safe_sub) errors instead of panicking.
                new_balance.checked_sub(old_balance).unwrap()
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
