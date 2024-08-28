use crate::network::NetworkDefinition;
use crate::types::EntityType;
use sbor::rust::prelude::*;

/// Represents an HRP set (typically corresponds to a network).
#[derive(Debug, Clone)]
pub struct HrpSet {
    /* Entities */
    pub package: String,
    pub resource: String,
    pub component: String,
    pub account: String,
    pub identity: String,
    pub consensus_manager: String,
    pub validator: String,
    pub access_controller: String,
    pub pool: String,
    pub locker: String,
    pub transaction_tracker: String,
    pub internal_vault: String,
    pub internal_component: String,
    pub internal_key_value_store: String,

    /* Transaction Parts */
    pub transaction_intent: String,
    pub signed_transaction_intent: String,
    pub subintent: String,
    pub notarized_transaction: String,
    pub round_update_transaction: String,
    pub system_transaction: String,
    pub ledger_transaction: String,
}

impl HrpSet {
    pub fn get_entity_hrp(&self, entity: &EntityType) -> &str {
        match entity {
            EntityType::GlobalPackage => &self.package,
            EntityType::GlobalFungibleResourceManager => &self.resource,
            EntityType::GlobalNonFungibleResourceManager => &self.resource,
            EntityType::GlobalConsensusManager => &self.consensus_manager,
            EntityType::GlobalValidator => &self.validator,
            EntityType::GlobalAccessController => &self.access_controller,
            EntityType::GlobalAccount => &self.account,
            EntityType::GlobalIdentity => &self.identity,
            EntityType::GlobalGenericComponent => &self.component,
            EntityType::GlobalPreallocatedSecp256k1Account => &self.account,
            EntityType::GlobalPreallocatedEd25519Account => &self.account,
            EntityType::GlobalPreallocatedSecp256k1Identity => &self.identity,
            EntityType::GlobalPreallocatedEd25519Identity => &self.identity,
            EntityType::InternalFungibleVault => &self.internal_vault,
            EntityType::InternalNonFungibleVault => &self.internal_vault,
            EntityType::InternalGenericComponent => &self.internal_component,
            EntityType::InternalKeyValueStore => &self.internal_key_value_store,
            EntityType::GlobalOneResourcePool
            | EntityType::GlobalTwoResourcePool
            | EntityType::GlobalMultiResourcePool => &self.pool,
            EntityType::GlobalAccountLocker => &self.locker,
            EntityType::GlobalTransactionTracker => &self.transaction_tracker,
        }
    }
}

impl From<&NetworkDefinition> for HrpSet {
    fn from(network_definition: &NetworkDefinition) -> Self {
        let suffix = &network_definition.hrp_suffix;
        HrpSet {
            /* Entities */
            package: format!("package_{}", suffix),
            resource: format!("resource_{}", suffix),
            component: format!("component_{}", suffix),
            account: format!("account_{}", suffix),
            identity: format!("identity_{}", suffix),
            consensus_manager: format!("consensusmanager_{}", suffix),
            validator: format!("validator_{}", suffix),
            access_controller: format!("accesscontroller_{}", suffix),
            pool: format!("pool_{}", suffix),
            locker: format!("locker_{}", suffix),
            transaction_tracker: format!("transactiontracker_{}", suffix),
            internal_vault: format!("internal_vault_{}", suffix),
            internal_component: format!("internal_component_{}", suffix),
            internal_key_value_store: format!("internal_keyvaluestore_{}", suffix),

            /* Transaction Parts */
            transaction_intent: format!("txid_{}", suffix),
            signed_transaction_intent: format!("signedintent_{}", suffix),
            subintent: format!("subtxid_{}", suffix),
            notarized_transaction: format!("notarizedtransaction_{}", suffix),
            round_update_transaction: format!("roundupdatetransaction_{}", suffix),
            system_transaction: format!("systemtransaction_{}", suffix),
            ledger_transaction: format!("ledgertransaction_{}", suffix),
        }
    }
}
