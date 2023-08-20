use crate::component::ObjectStubHandle;
use crate::engine::wasm_api::{addr, copy_buffer};
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
    pub fn allocate_component_address(
        blueprint_id: BlueprintId,
    ) -> (GlobalAddressReservation, ComponentAddress) {
        let blueprint_id = scrypto_encode(&blueprint_id).unwrap();
        let bytes = copy_buffer(unsafe {
            addr::address_allocate(blueprint_id.as_ptr(), blueprint_id.len())
        });
        scrypto_decode(&bytes).unwrap()
    }

    /// Get the global address an address reservation is associated with
    pub fn get_reservation_address(reservation: &GlobalAddressReservation) -> GlobalAddress {
        let node_id = reservation.0.as_node_id();
        let bytes = copy_buffer(unsafe {
            addr::address_get_reservation_address(node_id.as_ref().as_ptr(), node_id.as_ref().len())
        });
        scrypto_decode(&bytes).unwrap()
    }

    pub fn global_component() -> Global<AnyComponent> {
        let id = ScryptoVmV1Api::actor_get_object_id(ACTOR_REF_GLOBAL);
        Global(AnyComponent(ObjectStubHandle::Global(
            GlobalAddress::new_or_panic(id.0),
        )))
    }

    pub fn global_address() -> ComponentAddress {
        let address = ScryptoVmV1Api::actor_get_object_id(ACTOR_REF_GLOBAL);
        ComponentAddress::new_or_panic(address.into())
    }

    /// Returns the current package address.
    pub fn package_address() -> PackageAddress {
        ScryptoVmV1Api::actor_get_blueprint_id().package_address
    }

    pub fn package_token() -> NonFungibleGlobalId {
        NonFungibleGlobalId::package_of_direct_caller_badge(Runtime::package_address())
    }

    /// Emits an application event
    pub fn emit_event<T: ScryptoEncode + ScryptoDescribe + ScryptoEvent>(event: T) {
        ScryptoVmV1Api::actor_emit_event(
            T::event_name().to_owned(),
            scrypto_encode(&event).unwrap(),
        );
    }

    pub fn assert_access_rule(rule: AccessRule) {
        let object_id = ScryptoVmV1Api::actor_get_object_id(ACTOR_REF_AUTH_ZONE);
        ScryptoVmV1Api::object_call(
            &object_id,
            AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT,
            scrypto_encode(&AuthZoneAssertAccessRuleInput { rule }).unwrap(),
        );
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        ScryptoVmV1Api::sys_get_transaction_hash()
    }

    /// Returns the transaction hash.
    pub fn generate_ruid() -> [u8; 32] {
        ScryptoVmV1Api::sys_generate_ruid()
    }

    pub fn panic(message: String) -> ! {
        ScryptoVmV1Api::sys_panic(message);
        loop {}
    }

    /// Returns the current epoch
    pub fn current_epoch() -> Epoch {
        let rtn = ScryptoVmV1Api::object_call(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
            scrypto_encode(&ConsensusManagerGetCurrentEpochInput).unwrap(),
        );

        scrypto_decode(&rtn).unwrap()
    }

    pub fn get_execution_cost_unit_limit() -> u32 {
        ScryptoVmV1Api::costing_get_execution_cost_unit_limit()
    }

    pub fn get_execution_cost_unit_price() -> Decimal {
        ScryptoVmV1Api::costing_get_execution_cost_unit_price()
    }

    pub fn get_finalization_cost_unit_limit() -> u32 {
        ScryptoVmV1Api::costing_get_finalization_cost_unit_limit()
    }

    pub fn get_finalization_cost_unit_price() -> Decimal {
        ScryptoVmV1Api::costing_get_finalization_cost_unit_price()
    }

    pub fn get_usd_price() -> Decimal {
        ScryptoVmV1Api::costing_get_usd_price()
    }

    pub fn get_tip_percentage() -> u32 {
        ScryptoVmV1Api::costing_get_tip_percentage()
    }

    pub fn get_fee_balance() -> Decimal {
        ScryptoVmV1Api::costing_get_fee_balance()
    }
}
