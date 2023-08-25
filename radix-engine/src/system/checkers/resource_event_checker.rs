use std::collections::BTreeMap;
use radix_engine_common::constants::RESOURCE_PACKAGE;
use radix_engine_common::math::{Decimal, SafeAdd, SafeSub};
use radix_engine_common::prelude::{ResourceAddress, scrypto_decode};
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::resource::FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT;
use radix_engine_interface::prelude::{BlueprintInfo, Emitter};
use radix_engine_interface::traits::ScryptoEvent;
use radix_engine_interface::types::EventTypeIdentifier;
use crate::blueprints::resource::{BurnFungibleResourceEvent, MintFungibleResourceEvent};
use crate::system::checkers::ApplicationEventChecker;

#[derive(Debug, Default)]
pub struct ResourceEventChecker {
    tracker: BTreeMap<ResourceAddress, Decimal>,
}

#[derive(Debug, Default)]
pub struct ResourceEventCheckerResults {
}

impl ApplicationEventChecker for ResourceEventChecker {
    type ApplicationEventCheckerResults = ResourceEventCheckerResults;

    fn on_event(&mut self, info: BlueprintInfo, event_id: EventTypeIdentifier, event_payload: &Vec<u8>) {
        if info.blueprint_id.package_address.ne(&RESOURCE_PACKAGE) {
            return;
        }


        match info.blueprint_id.blueprint_name.as_str() {
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT => {
                match event_id {
                    EventTypeIdentifier(Emitter::Method(node_id, ObjectModuleId::Main), event_name) => {
                        let address = ResourceAddress::new_or_panic(node_id.0);
                        match event_name.as_str() {
                            MintFungibleResourceEvent::EVENT_NAME => {
                                let event: MintFungibleResourceEvent = scrypto_decode(event_payload).unwrap();
                                self.tracker.entry(address).or_default().safe_add(event.amount).unwrap();
                            }
                            BurnFungibleResourceEvent::EVENT_NAME => {
                                let event: BurnFungibleResourceEvent = scrypto_decode(event_payload).unwrap();
                                self.tracker.get_mut(&address).unwrap().safe_sub(event.amount).expect("Burnt more resources than was minted.");
                            }
                            _ => {

                            }
                        }
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    }

    fn on_finish(&self) -> Self::ApplicationEventCheckerResults {
        ResourceEventCheckerResults {}
    }
}
