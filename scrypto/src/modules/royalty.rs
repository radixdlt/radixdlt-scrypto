use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::ModuleHandle;
use crate::runtime::*;
use crate::*;
use radix_engine_common::types::RoyaltyAmount;
use radix_engine_interface::api::node_modules::royalty::{
    ComponentClaimRoyaltiesInput, ComponentLockRoyaltyInput, ComponentRoyaltyCreateInput,
    ComponentSetRoyaltyInput, COMPONENT_ROYALTY_ADMIN_ROLE, COMPONENT_ROYALTY_ADMIN_UPDATER_ROLE,
    COMPONENT_ROYALTY_BLUEPRINT, COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT,
    COMPONENT_ROYALTY_CREATE_IDENT, COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT,
    COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientBlueprintApi;
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::constants::ROYALTY_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::types::ComponentRoyaltyConfig;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use scrypto::modules::Attachable;

pub trait HasComponentRoyalties {
    fn set_royalty<M: ToString>(&self, method: M, amount: RoyaltyAmount);
    fn lock_royalty<M: ToString>(&self, method: M);
    fn claim_component_royalties(&self) -> Bucket;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Royalty(pub ModuleHandle);

impl Attachable for Royalty {
    const MODULE_ID: ObjectModuleId = ObjectModuleId::Royalty;

    fn new(handle: ModuleHandle) -> Self {
        Royalty(handle)
    }

    fn handle(&self) -> &ModuleHandle {
        &self.0
    }
}

impl Default for Royalty {
    fn default() -> Self {
        Royalty::new(ComponentRoyaltyConfig::default())
    }
}

impl Royalty {
    pub fn new(royalty_config: ComponentRoyaltyConfig) -> Self {
        let rtn = ScryptoEnv
            .call_function(
                ROYALTY_MODULE_PACKAGE,
                COMPONENT_ROYALTY_BLUEPRINT,
                COMPONENT_ROYALTY_CREATE_IDENT,
                scrypto_encode(&ComponentRoyaltyCreateInput { royalty_config }).unwrap(),
            )
            .unwrap();

        let royalty: Own = scrypto_decode(&rtn).unwrap();
        Self(ModuleHandle::Own(royalty))
    }

    pub fn set_royalty<M: ToString>(&self, method: M, amount: RoyaltyAmount) {
        self.call_ignore_rtn(
            COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
            &ComponentSetRoyaltyInput {
                method: method.to_string(),
                amount,
            },
        );
    }

    pub fn lock_royalty<M: ToString>(&self, method: M) {
        self.call_ignore_rtn(
            COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT,
            &ComponentLockRoyaltyInput {
                method: method.to_string(),
            },
        );
    }

    pub fn claim_royalties(&self) -> Bucket {
        self.call(
            COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT,
            &ComponentClaimRoyaltiesInput {},
        )
    }
}

pub struct RoyaltyRoles<T> {
    pub royalty_admin: T,
    pub royalty_admin_updater: T,
}

impl<T> RoyaltyRoles<T> {
    pub fn list(self) -> Vec<(&'static str, T)> {
        vec![
            (COMPONENT_ROYALTY_ADMIN_ROLE, self.royalty_admin),
            (
                COMPONENT_ROYALTY_ADMIN_UPDATER_ROLE,
                self.royalty_admin_updater,
            ),
        ]
    }
}
