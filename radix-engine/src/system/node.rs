use crate::system::node_modules::access_rules::*;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_substates::*;
use crate::types::*;
use radix_engine_interface::api::component::*;
use radix_engine_interface::types::SubstateKey;

#[derive(Debug)]
pub enum RENodeModuleInit {
    /* Type info */
    TypeInfo(TypeInfoSubstate),

    /* Metadata */
    Metadata(BTreeMap<SubstateKey, RuntimeSubstate>),

    /* Access rules */
    MethodAccessRules(MethodAccessRulesSubstate),

    /* Royalty */
    ComponentRoyalty(
        ComponentRoyaltyConfigSubstate,
        ComponentRoyaltyAccumulatorSubstate,
    ),
}

impl RENodeModuleInit {
    pub fn to_substates(self) -> HashMap<SubstateKey, RuntimeSubstate> {
        let mut substates = HashMap::<SubstateKey, RuntimeSubstate>::new();
        match self {
            RENodeModuleInit::Metadata(metadata_substates) => {
                substates.extend(metadata_substates);
            }
            RENodeModuleInit::MethodAccessRules(access_rules) => {
                substates.insert(AccessRulesOffset::AccessRules.into(), access_rules.into());
            }
            RENodeModuleInit::TypeInfo(type_info) => {
                substates.insert(TypeInfoOffset::TypeInfo.into(), type_info.into());
            }
            RENodeModuleInit::ComponentRoyalty(config, accumulator) => {
                substates.insert(RoyaltyOffset::Royalty.into(), config.into());
                substates.insert(RoyaltyOffset::Royalty.into(), accumulator.into());
            }
        }

        substates
    }
}

#[derive(Debug)]
pub enum RENodeInit {
    GlobalObject(BTreeMap<SubstateKey, RuntimeSubstate>),
    Object(BTreeMap<SubstateKey, RuntimeSubstate>),
    KeyValueStore,
}

impl RENodeInit {
    pub fn to_substates(self) -> HashMap<SubstateKey, RuntimeSubstate> {
        let mut substates = HashMap::<SubstateKey, RuntimeSubstate>::new();
        match self {
            RENodeInit::GlobalObject(object_substates) | RENodeInit::Object(object_substates) => {
                substates.extend(object_substates);
            }
            RENodeInit::KeyValueStore => {}
        };

        substates
    }
}
