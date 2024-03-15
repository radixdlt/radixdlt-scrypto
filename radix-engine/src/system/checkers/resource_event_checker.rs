use crate::blueprints::resource::{
    fungible_vault, non_fungible_vault, BurnFungibleResourceEvent, BurnNonFungibleResourceEvent,
    MintFungibleResourceEvent, MintNonFungibleResourceEvent,
};
use crate::system::checkers::ApplicationEventChecker;
use radix_common::constants::RESOURCE_PACKAGE;
use radix_common::math::{CheckedAdd, CheckedSub, Decimal};
use radix_common::prelude::{scrypto_decode, ResourceAddress};
use radix_common::traits::ScryptoEvent;
use radix_common::types::NodeId;
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, FUNGIBLE_VAULT_BLUEPRINT,
    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, NON_FUNGIBLE_VAULT_BLUEPRINT,
};
use radix_engine_interface::prelude::{BlueprintInfo, Emitter};
use radix_engine_interface::types::EventTypeIdentifier;
use sbor::rust::collections::BTreeMap;
use sbor::rust::vec::Vec;

#[derive(Debug, Default)]
pub struct ResourceEventChecker {
    resource_tracker: BTreeMap<ResourceAddress, Decimal>,
    vault_tracker: BTreeMap<NodeId, Decimal>,
}

#[derive(Debug, Default)]
pub struct ResourceEventCheckerResults {
    pub total_supply: BTreeMap<ResourceAddress, Decimal>,
    pub vault_amounts: BTreeMap<NodeId, Decimal>,
}

impl ApplicationEventChecker for ResourceEventChecker {
    type ApplicationEventCheckerResults = ResourceEventCheckerResults;

    fn on_event(
        &mut self,
        info: BlueprintInfo,
        event_id: EventTypeIdentifier,
        event_payload: &Vec<u8>,
    ) {
        if info.blueprint_id.package_address.ne(&RESOURCE_PACKAGE) {
            return;
        }

        match info.blueprint_id.blueprint_name.as_str() {
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT => match event_id {
                EventTypeIdentifier(Emitter::Method(node_id, ModuleId::Main), event_name) => {
                    let address = ResourceAddress::new_or_panic(node_id.0);
                    match event_name.as_str() {
                        MintFungibleResourceEvent::EVENT_NAME => {
                            let event: MintFungibleResourceEvent =
                                scrypto_decode(event_payload).unwrap();
                            let tracked = self.resource_tracker.entry(address).or_default();
                            *tracked = tracked.checked_add(event.amount).unwrap();
                        }
                        BurnFungibleResourceEvent::EVENT_NAME => {
                            let event: BurnFungibleResourceEvent =
                                scrypto_decode(event_payload).unwrap();
                            let tracked = self.resource_tracker.entry(address).or_default();
                            *tracked = tracked.checked_sub(event.amount).unwrap();

                            if tracked.is_negative() {
                                panic!("Burnt more resources than was minted.");
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT => match event_id {
                EventTypeIdentifier(Emitter::Method(node_id, ModuleId::Main), event_name) => {
                    let address = ResourceAddress::new_or_panic(node_id.0);
                    match event_name.as_str() {
                        MintNonFungibleResourceEvent::EVENT_NAME => {
                            let event: MintNonFungibleResourceEvent =
                                scrypto_decode(event_payload).unwrap();
                            let amount: Decimal = event.ids.len().into();
                            let tracked = self.resource_tracker.entry(address).or_default();
                            *tracked = tracked.checked_add(amount).unwrap();
                        }
                        BurnNonFungibleResourceEvent::EVENT_NAME => {
                            let event: BurnNonFungibleResourceEvent =
                                scrypto_decode(event_payload).unwrap();
                            let amount: Decimal = event.ids.len().into();
                            let tracked = self.resource_tracker.entry(address).or_default();
                            *tracked = tracked.checked_sub(amount).unwrap();

                            if tracked.is_negative() {
                                panic!("Burnt more resources than was minted.");
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            FUNGIBLE_VAULT_BLUEPRINT => match event_id {
                EventTypeIdentifier(Emitter::Method(node_id, ModuleId::Main), event_name) => {
                    match event_name.as_str() {
                        fungible_vault::DepositEvent::EVENT_NAME => {
                            let event: fungible_vault::DepositEvent =
                                scrypto_decode(event_payload).unwrap();
                            let tracked = self.vault_tracker.entry(node_id).or_default();
                            *tracked = tracked.checked_add(event.amount).unwrap();
                        }
                        fungible_vault::WithdrawEvent::EVENT_NAME => {
                            let event: fungible_vault::WithdrawEvent =
                                scrypto_decode(event_payload).unwrap();
                            let tracked = self.vault_tracker.entry(node_id).or_default();
                            *tracked = tracked.checked_sub(event.amount).unwrap();
                            if tracked.is_negative() {
                                panic!("Removed more resources than exists.");
                            }
                        }
                        fungible_vault::RecallEvent::EVENT_NAME => {
                            let event: fungible_vault::RecallEvent =
                                scrypto_decode(event_payload).unwrap();
                            let tracked = self.vault_tracker.entry(node_id).or_default();
                            *tracked = tracked.checked_sub(event.amount).unwrap();
                            if tracked.is_negative() {
                                panic!("Removed more resources than exists.");
                            }
                        }
                        fungible_vault::PayFeeEvent::EVENT_NAME => {
                            let event: fungible_vault::PayFeeEvent =
                                scrypto_decode(event_payload).unwrap();
                            let tracked = self.vault_tracker.entry(node_id).or_default();
                            *tracked = tracked.checked_sub(event.amount).unwrap();
                            if tracked.is_negative() {
                                panic!("Removed more resources than exists.");
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            NON_FUNGIBLE_VAULT_BLUEPRINT => match event_id {
                EventTypeIdentifier(Emitter::Method(node_id, ModuleId::Main), event_name) => {
                    match event_name.as_str() {
                        non_fungible_vault::DepositEvent::EVENT_NAME => {
                            let event: non_fungible_vault::DepositEvent =
                                scrypto_decode(event_payload).unwrap();
                            let tracked = self.vault_tracker.entry(node_id).or_default();
                            let amount: Decimal = event.ids.len().into();
                            *tracked = tracked.checked_add(amount).unwrap();
                        }
                        non_fungible_vault::WithdrawEvent::EVENT_NAME => {
                            let event: non_fungible_vault::WithdrawEvent =
                                scrypto_decode(event_payload).unwrap();
                            let tracked = self.vault_tracker.entry(node_id).or_default();
                            let amount: Decimal = event.ids.len().into();
                            *tracked = tracked.checked_sub(amount).unwrap();
                            if tracked.is_negative() {
                                panic!("Removed more resources than exists.");
                            }
                        }
                        fungible_vault::RecallEvent::EVENT_NAME => {
                            let event: non_fungible_vault::RecallEvent =
                                scrypto_decode(event_payload).unwrap();
                            let tracked = self.vault_tracker.entry(node_id).or_default();
                            let amount: Decimal = event.ids.len().into();
                            *tracked = tracked.checked_sub(amount).unwrap();
                            if tracked.is_negative() {
                                panic!("Removed more resources than exists.");
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn on_finish(&self) -> Self::ApplicationEventCheckerResults {
        ResourceEventCheckerResults {
            total_supply: self.resource_tracker.clone(),
            vault_amounts: self.vault_tracker.clone(),
        }
    }
}
