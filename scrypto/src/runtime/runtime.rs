use crate::component::ObjectStubHandle;
use crate::prelude::{AnyComponent, Global};
use radix_engine_common::math::Decimal;
use radix_engine_common::types::GlobalAddressReservation;
use radix_engine_interface::api::system_modules::auth_api::ClientAuthApi;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerGetCurrentEpochInput, CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
};
use radix_engine_interface::blueprints::resource::{AccessRule, NonFungibleGlobalId};
use radix_engine_interface::constants::CONSENSUS_MANAGER;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDescribe, ScryptoEncode,
};
use radix_engine_interface::traits::ScryptoEvent;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use scrypto::engine::scrypto_env::ScryptoEnv;

/// The transaction runtime.
#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    /// Returns the current epoch
    pub fn current_epoch() -> Epoch {
        let rtn = ScryptoEnv
            .call_method(
                CONSENSUS_MANAGER.as_node_id(),
                CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
                scrypto_encode(&ConsensusManagerGetCurrentEpochInput).unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
    }

    pub fn global_component() -> Global<AnyComponent> {
        let address: GlobalAddress = ScryptoEnv.actor_get_global_address().unwrap();
        Global(AnyComponent(ObjectStubHandle::Global(address)))
    }

    pub fn global_address() -> ComponentAddress {
        let address: GlobalAddress = ScryptoEnv.actor_get_global_address().unwrap();
        ComponentAddress::new_or_panic(address.into())
    }

    /// Returns the running entity.
    pub fn blueprint_id() -> BlueprintId {
        ScryptoEnv.actor_get_blueprint().unwrap()
    }

    pub fn node_id() -> NodeId {
        ScryptoEnv.actor_get_node_id().unwrap()
    }

    /// Returns the current package address.
    pub fn package_address() -> PackageAddress {
        Self::blueprint_id().package_address
    }

    pub fn package_token() -> NonFungibleGlobalId {
        NonFungibleGlobalId::package_of_direct_caller_badge(Runtime::package_address())
    }

    /// Returns the transaction hash.
    pub fn transaction_hash() -> Hash {
        ScryptoEnv.get_transaction_hash().unwrap()
    }

    /// Emits an application event
    pub fn emit_event<T: ScryptoEncode + ScryptoDescribe + ScryptoEvent>(event: T) {
        ScryptoEnv
            .emit_event(T::event_name().to_owned(), scrypto_encode(&event).unwrap())
            .unwrap();
    }

    pub fn assert_access_rule(access_rule: AccessRule) {
        let mut env = ScryptoEnv;
        env.assert_access_rule(access_rule).unwrap();
    }

    pub fn allocate_component_address(
        blueprint_id: BlueprintId,
    ) -> (GlobalAddressReservation, ComponentAddress) {
        let mut env = ScryptoEnv;
        let (ownership, global_address) = env.allocate_global_address(blueprint_id).unwrap();
        (ownership, unsafe {
            ComponentAddress::new_unchecked(global_address.as_node_id().0)
        })
    }

    pub fn cost_unit_limit() -> u32 {
        ScryptoEnv.cost_unit_limit().unwrap()
    }

    pub fn cost_unit_price() -> Decimal {
        ScryptoEnv.cost_unit_price().unwrap()
    }

    pub fn tip_percentage() -> u32 {
        ScryptoEnv.tip_percentage().unwrap()
    }

    pub fn fee_balance() -> Decimal {
        ScryptoEnv.fee_balance().unwrap()
    }

    pub fn panic(message: String) -> ! {
        ScryptoEnv.panic(message).unwrap();
        loop {}
    }
}
