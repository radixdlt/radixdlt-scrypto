use crate::blueprints::resource::{
    BurnFungibleResourceEvent, BurnNonFungibleResourceEvent, MintFungibleResourceEvent,
    MintNonFungibleResourceEvent,
};
use crate::system::checkers::ApplicationEventChecker;
use radix_engine_common::constants::RESOURCE_PACKAGE;
use radix_engine_common::math::{Decimal, SafeAdd, SafeSub};
use radix_engine_common::prelude::{scrypto_decode, ResourceAddress};
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
};
use radix_engine_interface::prelude::{BlueprintInfo, Emitter};
use radix_engine_interface::traits::ScryptoEvent;
use radix_engine_interface::types::EventTypeIdentifier;
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct ResourceEventChecker {
    tracker: BTreeMap<ResourceAddress, Decimal>,
}

#[derive(Debug, Default)]
pub struct ResourceEventCheckerResults {
    pub total_supply: BTreeMap<ResourceAddress, Decimal>,
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
                EventTypeIdentifier(Emitter::Method(node_id, ObjectModuleId::Main), event_name) => {
                    let address = ResourceAddress::new_or_panic(node_id.0);
                    match event_name.as_str() {
                        MintFungibleResourceEvent::EVENT_NAME => {
                            let event: MintFungibleResourceEvent =
                                scrypto_decode(event_payload).unwrap();
                            let tracked = self.tracker.entry(address).or_default();
                            *tracked = tracked.safe_add(event.amount).unwrap();
                        }
                        BurnFungibleResourceEvent::EVENT_NAME => {
                            let event: BurnFungibleResourceEvent =
                                scrypto_decode(event_payload).unwrap();
                            let tracked = self.tracker.get_mut(&address).unwrap();
                            *tracked = tracked.safe_sub(event.amount).unwrap();

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
                EventTypeIdentifier(Emitter::Method(node_id, ObjectModuleId::Main), event_name) => {
                    let address = ResourceAddress::new_or_panic(node_id.0);
                    match event_name.as_str() {
                        MintNonFungibleResourceEvent::EVENT_NAME => {
                            let event: MintNonFungibleResourceEvent =
                                scrypto_decode(event_payload).unwrap();
                            let amount: Decimal = event.ids.len().into();
                            let tracked = self.tracker.entry(address).or_default();
                            *tracked = tracked.safe_add(amount).unwrap();
                        }
                        BurnNonFungibleResourceEvent::EVENT_NAME => {
                            let event: BurnNonFungibleResourceEvent =
                                scrypto_decode(event_payload).unwrap();
                            let amount: Decimal = event.ids.len().into();
                            let tracked = self.tracker.get_mut(&address).unwrap();
                            *tracked = tracked.safe_sub(amount).unwrap();

                            if tracked.is_negative() {
                                panic!("Burnt more resources than was minted.");
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
            total_supply: self.tracker.clone(),
        }
    }
}
