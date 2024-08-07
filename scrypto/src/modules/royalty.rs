use crate::engine::scrypto_env::ScryptoVmV1Api;
use crate::modules::ModuleHandle;
use crate::runtime::*;
use crate::*;
use radix_common::constants::ROYALTY_MODULE_PACKAGE;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::types::RoyaltyAmount;
use radix_engine_interface::api::AttachedModuleId;
use radix_engine_interface::blueprints::resource::FungibleBucket;
use radix_engine_interface::object_modules::royalty::{
    ComponentClaimRoyaltiesInput, ComponentRoyaltyCreateInput, ComponentRoyaltyLockInput,
    ComponentRoyaltySetInput, COMPONENT_ROYALTY_BLUEPRINT, COMPONENT_ROYALTY_CLAIMER_ROLE,
    COMPONENT_ROYALTY_CLAIMER_UPDATER_ROLE, COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT,
    COMPONENT_ROYALTY_CREATE_IDENT, COMPONENT_ROYALTY_LOCKER_ROLE,
    COMPONENT_ROYALTY_LOCKER_UPDATER_ROLE, COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT,
    COMPONENT_ROYALTY_SETTER_ROLE, COMPONENT_ROYALTY_SETTER_UPDATER_ROLE,
    COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
};
use radix_engine_interface::types::ComponentRoyaltyConfig;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use scrypto::modules::Attachable;

pub trait HasComponentRoyalties {
    fn set_royalty<M: ToString>(&self, method: M, amount: RoyaltyAmount);
    fn lock_royalty<M: ToString>(&self, method: M);
    fn claim_component_royalties(&self) -> FungibleBucket;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Royalty(pub ModuleHandle);

impl Attachable for Royalty {
    const MODULE_ID: AttachedModuleId = AttachedModuleId::Royalty;

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
        let rtn = ScryptoVmV1Api::blueprint_call(
            ROYALTY_MODULE_PACKAGE,
            COMPONENT_ROYALTY_BLUEPRINT,
            COMPONENT_ROYALTY_CREATE_IDENT,
            scrypto_encode(&ComponentRoyaltyCreateInput { royalty_config }).unwrap(),
        );

        let royalty: Own = scrypto_decode(&rtn).unwrap();
        Self(ModuleHandle::Own(royalty))
    }

    pub fn set_royalty<M: ToString>(&self, method: M, amount: RoyaltyAmount) {
        self.call_ignore_rtn(
            COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
            &ComponentRoyaltySetInput {
                method: method.to_string(),
                amount,
            },
        );
    }

    pub fn lock_royalty<M: ToString>(&self, method: M) {
        self.call_ignore_rtn(
            COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT,
            &ComponentRoyaltyLockInput {
                method: method.to_string(),
            },
        );
    }

    pub fn claim_royalties(&self) -> FungibleBucket {
        self.call(
            COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT,
            &ComponentClaimRoyaltiesInput {},
        )
    }
}

pub struct RoyaltyRoles<T> {
    pub royalty_setter: T,
    pub royalty_setter_updater: T,
    pub royalty_locker: T,
    pub royalty_locker_updater: T,
    pub royalty_claimer: T,
    pub royalty_claimer_updater: T,
}

impl<T> RoyaltyRoles<T> {
    pub fn list(self) -> Vec<(&'static str, T)> {
        vec![
            (COMPONENT_ROYALTY_SETTER_ROLE, self.royalty_setter),
            (
                COMPONENT_ROYALTY_SETTER_UPDATER_ROLE,
                self.royalty_setter_updater,
            ),
            (COMPONENT_ROYALTY_LOCKER_ROLE, self.royalty_locker),
            (
                COMPONENT_ROYALTY_LOCKER_UPDATER_ROLE,
                self.royalty_locker_updater,
            ),
            (COMPONENT_ROYALTY_CLAIMER_ROLE, self.royalty_claimer),
            (
                COMPONENT_ROYALTY_CLAIMER_UPDATER_ROLE,
                self.royalty_claimer_updater,
            ),
        ]
    }
}
