use crate::component::ObjectStubHandle;
use crate::prelude::{AnyComponent, Global};
use radix_engine_common::math::Decimal;
use radix_engine_common::types::GlobalAddressReservation;
use radix_engine_interface::api::{ACTOR_REF_AUTH_ZONE, ACTOR_REF_GLOBAL};
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerGetCurrentEpochInput, CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
};
use radix_engine_interface::blueprints::resource::{
    AccessRule, AuthZoneAssertAccessRuleInput, NonFungibleGlobalId,
    AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT,
};
use radix_engine_interface::constants::CONSENSUS_MANAGER;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDescribe, ScryptoEncode,
};
use radix_engine_interface::traits::ScryptoEvent;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use scrypto::engine::scrypto_env::ScryptoVmV1Api;

/// The transaction runtime.
#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    /// Returns the current epoch
    pub fn current_epoch() -> Epoch {
        let rtn = ScryptoVmV1Api.object_call(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
            scrypto_encode(&ConsensusManagerGetCurrentEpochInput).unwrap(),
        );

        scrypto_decode(&rtn).unwrap()
    }

    pub fn global_component() -> Global<AnyComponent> {
        let id = ScryptoVmV1Api.actor_get_object_id(ACTOR_REF_GLOBAL);
        Global(AnyComponent(ObjectStubHandle::Global(
            GlobalAddress::new_or_panic(id.0),
        )))
    }

    pub fn global_address() -> ComponentAddress {
        let address = ScryptoVmV1Api.actor_get_object_id(ACTOR_REF_GLOBAL);
        ComponentAddress::new_or_panic(address.into())
    }

    /// Returns the current package address.
    pub fn package_address() -> PackageAddress {
        ScryptoVmV1Api.actor_get_blueprint_id().package_address
    }

    pub fn package_token() -> NonFungibleGlobalId {
        NonFungibleGlobalId::package_of_direct_caller_badge(Runtime::package_address())
    }

    /// Get the global address an address reservation is associated with
    pub fn get_reservation_address(reservation: &GlobalAddressReservation) -> GlobalAddress {
        ScryptoVmV1Api.get_reservation_address(reservation.0.as_node_id())
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        ScryptoVmV1Api.get_transaction_hash()
    }

    /// Returns the transaction hash.
    pub fn generate_ruid() -> [u8; 32] {
        ScryptoVmV1Api.generate_ruid()
    }

    /// Emits an application event
    pub fn emit_event<T: ScryptoEncode + ScryptoDescribe + ScryptoEvent>(event: T) {
        ScryptoVmV1Api
            .actor_emit_event(T::event_name().to_owned(), scrypto_encode(&event).unwrap());
    }

    pub fn assert_access_rule(rule: AccessRule) {
        let object_id = ScryptoVmV1Api.actor_get_object_id(ACTOR_REF_AUTH_ZONE);
        ScryptoVmV1Api.object_call(
            &object_id,
            AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT,
            scrypto_encode(&AuthZoneAssertAccessRuleInput { rule }).unwrap(),
        );
    }

    pub fn allocate_component_address(
        blueprint_id: BlueprintId,
    ) -> (GlobalAddressReservation, ComponentAddress) {
        let (ownership, global_address) = ScryptoVmV1Api.allocate_global_address(blueprint_id);
        (ownership, unsafe {
            ComponentAddress::new_unchecked(global_address.as_node_id().0)
        })
    }

    pub fn execution_cost_unit_limit() -> u32 {
        ScryptoVmV1Api.execution_cost_unit_limit()
    }

    pub fn execution_cost_unit_price() -> Decimal {
        ScryptoVmV1Api.execution_cost_unit_price()
    }

    pub fn finalization_cost_unit_limit() -> u32 {
        ScryptoVmV1Api.finalization_cost_unit_limit()
    }

    pub fn finalization_cost_unit_price() -> Decimal {
        ScryptoVmV1Api.finalization_cost_unit_price()
    }

    pub fn usd_price() -> Decimal {
        ScryptoVmV1Api.usd_price()
    }

    pub fn tip_percentage() -> u32 {
        ScryptoVmV1Api.tip_percentage()
    }

    pub fn fee_balance() -> Decimal {
        ScryptoVmV1Api.fee_balance()
    }

    pub fn panic(message: String) -> ! {
        ScryptoVmV1Api.panic(message);
        loop {}
    }
}
