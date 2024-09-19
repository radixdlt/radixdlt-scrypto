use crate::blueprints::resource::{FungibleVaultBalanceFieldPayload, FungibleVaultField};
use crate::internal_prelude::*;
use crate::system::system_db_reader::SystemDatabaseReader;
use radix_common::data::scrypto::model::*;
use radix_common::math::*;
use radix_engine_interface::types::*;
use radix_substate_store_interface::interface::*;
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
        new_node_ids: IndexSet<NodeId>,
        updates: &StateUpdates,
    ) -> Self {
        let mut new_packages = index_set_new();
        let mut new_components = index_set_new();
        let mut new_resources = index_set_new();
        let mut new_vaults = index_set_new();

        for node_id in new_node_ids {
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

        let vault_balance_changes = BalanceAccounter::new(substate_db, &updates).run();

        StateUpdateSummary {
            new_packages,
            new_components,
            new_resources,
            new_vaults,
            vault_balance_changes,
        }
    }

    pub fn new_from_state_updates_on_db(
        base_substate_db: &impl SubstateDatabase,
        updates: &StateUpdates,
    ) -> Self {
        let mut new_packages = index_set_new();
        let mut new_components = index_set_new();
        let mut new_resources = index_set_new();
        let mut new_vaults = index_set_new();

        let new_node_ids = updates
            .by_node
            .iter()
            .filter(|(node_id, updates)| {
                let type_id_partition_number = TYPE_INFO_FIELD_PARTITION;
                let type_id_substate_key = TypeInfoField::TypeInfo.into();
                let possible_creation = updates
                    .of_partition_ref(type_id_partition_number)
                    .is_some_and(|partition_updates| {
                        partition_updates.contains_set_update_for(&type_id_substate_key)
                    });
                if !possible_creation {
                    return false;
                }
                let node_previously_existed = base_substate_db
                    .get_raw_substate(node_id, type_id_partition_number, type_id_substate_key)
                    .is_some();
                return !node_previously_existed;
            })
            .map(|(node_id, _)| node_id);

        for node_id in new_node_ids {
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

        let vault_balance_changes = BalanceAccounter::new(base_substate_db, &updates).run();

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
    state_updates: &'a StateUpdates,
}

impl<'a, S: SubstateDatabase> BalanceAccounter<'a, S> {
    pub fn new(substate_db: &'a S, state_updates: &'a StateUpdates) -> Self {
        Self {
            system_reader: SystemDatabaseReader::new_with_overlay(substate_db, state_updates),
            state_updates,
        }
    }

    pub fn run(&self) -> IndexMap<NodeId, (ResourceAddress, BalanceChange)> {
        self.state_updates
            .by_node
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
            .fetch_substate::<FieldSubstate<FungibleVaultBalanceFieldPayload>>(
                vault_id,
                MAIN_BASE_PARTITION,
                &FungibleVaultField::Balance.into(),
            )
            .map(|new_substate| new_substate.into_payload().fully_update_and_into_latest_version().amount())
            .map(|new_balance| {
                let old_balance = self
                    .system_reader
                    .fetch_substate_from_database::<FieldSubstate<FungibleVaultBalanceFieldPayload>>(
                        vault_id,
                        MAIN_BASE_PARTITION,
                        &FungibleVaultField::Balance.into(),
                    )
                    .map(|old_balance| old_balance.into_payload().fully_update_and_into_latest_version().amount())
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
        let partition_num = MAIN_BASE_PARTITION.at_offset(PartitionOffset(1u8)).unwrap();

        self.state_updates
            .by_node
            .get(vault_id)
            .map(|node_updates| match node_updates {
                NodeStateUpdates::Delta { by_partition } => by_partition,
            })
            .and_then(|partitions| partitions.get(&partition_num))
            .map(|partition_update| {
                let mut added = BTreeSet::new();
                let mut removed = BTreeSet::new();

                match partition_update {
                    PartitionStateUpdates::Delta { by_substate } => {
                        for (substate_key, substate_update) in by_substate {
                            let id: NonFungibleLocalId =
                                scrypto_decode(substate_key.for_map().unwrap()).unwrap();
                            let previous_value = self
                                .system_reader
                                .fetch_substate_from_database::<ScryptoValue>(
                                    vault_id,
                                    partition_num,
                                    substate_key,
                                );

                            match substate_update {
                                DatabaseUpdate::Set(_) => {
                                    if previous_value.is_none() {
                                        added.insert(id);
                                    }
                                }
                                DatabaseUpdate::Delete => {
                                    if previous_value.is_some() {
                                        removed.insert(id);
                                    }
                                }
                            }
                        }
                    }
                    PartitionStateUpdates::Batch(_) => {
                        panic!("Invariant: vault partitions are never batch removed")
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
