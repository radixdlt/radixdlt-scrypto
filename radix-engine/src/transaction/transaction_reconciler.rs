use crate::blueprints::resource::{
    fungible_vault, non_fungible_vault, BurnFungibleResourceEvent, BurnNonFungibleResourceEvent,
    MintFungibleResourceEvent, MintNonFungibleResourceEvent,
};
use crate::system::system_db_reader::SystemDatabaseReader;
use crate::transaction::{BalanceChange, StateUpdateSummary};
use radix_common::prelude::scrypto_decode;
use radix_common::traits::ScryptoEvent;
use radix_common::types::ResourceAddress;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::types::{Emitter, EventTypeIdentifier};
use radix_rust::prelude::IndexMap;
use radix_rust::{btreeset, indexmap};
use radix_substate_store_interface::interface::SubstateDatabase;
use sbor::rust::ops::AddAssign;
use sbor::rust::ops::Neg;
use sbor::rust::vec::Vec;

pub fn reconcile_resource_state_and_events<'a, S: SubstateDatabase>(
    summary: &StateUpdateSummary,
    events: &Vec<(EventTypeIdentifier, Vec<u8>)>,
    system_db: SystemDatabaseReader<'a, S>,
) {
    let mut resource_changes_from_state = compute_resource_changes_from_state(summary);
    resource_changes_from_state.retain(|_, change| !change.prune_and_check_if_zero());

    let mut resource_changes_from_resman_events =
        compute_resource_changes_from_resman_events(events);
    resource_changes_from_resman_events.retain(|_, change| !change.prune_and_check_if_zero());

    let mut resource_changes_from_vault_events =
        compute_resource_changes_from_vault_events(events, &system_db);
    resource_changes_from_vault_events.retain(|_, change| !change.prune_and_check_if_zero());

    if resource_changes_from_state.ne(&resource_changes_from_vault_events)
        || resource_changes_from_vault_events.ne(&resource_changes_from_resman_events)
    {
        panic!("Txn Resource Reconciliation failed:\nState Changes: {:#?}\nResource Event Changes: {:#?}\nVault Event Changes: {:#?}",
               resource_changes_from_state,
               resource_changes_from_resman_events,
               resource_changes_from_vault_events,
        );
    }
}

fn compute_resource_changes_from_state(
    summary: &StateUpdateSummary,
) -> IndexMap<ResourceAddress, BalanceChange> {
    let mut resource_changes: IndexMap<ResourceAddress, BalanceChange> = indexmap!();
    for (_vault_id, (resource, change)) in &summary.vault_balance_changes {
        resource_changes
            .entry(*resource)
            .and_modify(|cur| {
                cur.add_assign(change.clone());
            })
            .or_insert(change.clone());
    }
    resource_changes
}

fn compute_resource_changes_from_resman_events(
    events: &Vec<(EventTypeIdentifier, Vec<u8>)>,
) -> IndexMap<ResourceAddress, BalanceChange> {
    let mut resource_changes_from_resman_events: IndexMap<ResourceAddress, BalanceChange> =
        indexmap!();

    for (event_id, event) in events {
        let (address, change) = match event_id.0 {
            Emitter::Method(node_id, ObjectModuleId::Main)
                if node_id.is_global_fungible_resource_manager() =>
            {
                let address = ResourceAddress::new_or_panic(node_id.0);

                let change = match event_id.1.as_str() {
                    MintFungibleResourceEvent::EVENT_NAME => {
                        let mint: MintFungibleResourceEvent = scrypto_decode(event).unwrap();
                        BalanceChange::Fungible(mint.amount)
                    }
                    BurnFungibleResourceEvent::EVENT_NAME => {
                        let burn: BurnFungibleResourceEvent = scrypto_decode(event).unwrap();
                        BalanceChange::Fungible(burn.amount.neg())
                    }
                    _ => continue,
                };

                (address, change)
            }

            Emitter::Method(node_id, ObjectModuleId::Main)
                if node_id.is_global_non_fungible_resource_manager() =>
            {
                let address = ResourceAddress::new_or_panic(node_id.0);

                let change = match event_id.1.as_str() {
                    MintNonFungibleResourceEvent::EVENT_NAME => {
                        let mint: MintNonFungibleResourceEvent = scrypto_decode(event).unwrap();
                        BalanceChange::NonFungible {
                            added: mint.ids.into_iter().collect(),
                            removed: btreeset!(),
                        }
                    }
                    BurnNonFungibleResourceEvent::EVENT_NAME => {
                        let burn: BurnNonFungibleResourceEvent = scrypto_decode(event).unwrap();
                        BalanceChange::NonFungible {
                            added: btreeset!(),
                            removed: burn.ids.into_iter().collect(),
                        }
                    }
                    _ => continue,
                };

                (address, change)
            }
            _ => continue,
        };

        resource_changes_from_resman_events
            .entry(address)
            .and_modify(|cur| {
                cur.add_assign(change.clone());
            })
            .or_insert(change.clone());
    }

    resource_changes_from_resman_events
}

fn compute_resource_changes_from_vault_events<'a, S: SubstateDatabase>(
    events: &Vec<(EventTypeIdentifier, Vec<u8>)>,
    system_db: &SystemDatabaseReader<'a, S>,
) -> IndexMap<ResourceAddress, BalanceChange> {
    let mut resource_changes_from_vault_events: IndexMap<ResourceAddress, BalanceChange> =
        indexmap!();

    for (event_id, event) in events {
        let (address, change) = match event_id.0 {
            Emitter::Method(node_id, ObjectModuleId::Main)
                if node_id.is_internal_fungible_vault() =>
            {
                let address: ResourceAddress = system_db
                    .get_object_info(node_id)
                    .unwrap()
                    .get_outer_object()
                    .try_into()
                    .unwrap();

                let change = match event_id.1.as_str() {
                    fungible_vault::DepositEvent::EVENT_NAME => {
                        let deposit: fungible_vault::DepositEvent = scrypto_decode(event).unwrap();
                        BalanceChange::Fungible(deposit.amount)
                    }
                    fungible_vault::WithdrawEvent::EVENT_NAME => {
                        let withdraw: fungible_vault::WithdrawEvent =
                            scrypto_decode(event).unwrap();
                        BalanceChange::Fungible(withdraw.amount.neg())
                    }
                    fungible_vault::RecallEvent::EVENT_NAME => {
                        let recall: fungible_vault::RecallEvent = scrypto_decode(event).unwrap();
                        BalanceChange::Fungible(recall.amount.neg())
                    }
                    fungible_vault::PayFeeEvent::EVENT_NAME => {
                        let recall: fungible_vault::PayFeeEvent = scrypto_decode(event).unwrap();
                        BalanceChange::Fungible(recall.amount.neg())
                    }
                    _ => continue,
                };

                (address, change)
            }

            Emitter::Method(node_id, ObjectModuleId::Main)
                if node_id.is_internal_non_fungible_vault() =>
            {
                let address: ResourceAddress = system_db
                    .get_object_info(node_id)
                    .unwrap()
                    .get_outer_object()
                    .try_into()
                    .unwrap();

                let change = match event_id.1.as_str() {
                    non_fungible_vault::DepositEvent::EVENT_NAME => {
                        let deposit: non_fungible_vault::DepositEvent =
                            scrypto_decode(event).unwrap();
                        BalanceChange::NonFungible {
                            added: deposit.ids.into_iter().collect(),
                            removed: btreeset!(),
                        }
                    }
                    non_fungible_vault::WithdrawEvent::EVENT_NAME => {
                        let withdraw: non_fungible_vault::WithdrawEvent =
                            scrypto_decode(event).unwrap();
                        BalanceChange::NonFungible {
                            added: btreeset!(),
                            removed: withdraw.ids.into_iter().collect(),
                        }
                    }
                    non_fungible_vault::RecallEvent::EVENT_NAME => {
                        let recall: non_fungible_vault::RecallEvent =
                            scrypto_decode(event).unwrap();
                        BalanceChange::NonFungible {
                            added: btreeset!(),
                            removed: recall.ids.into_iter().collect(),
                        }
                    }
                    _ => continue,
                };

                (address, change)
            }

            _ => continue,
        };

        resource_changes_from_vault_events
            .entry(address)
            .and_modify(|cur| {
                cur.add_assign(change.clone());
            })
            .or_insert(change.clone());
    }

    resource_changes_from_vault_events
}
