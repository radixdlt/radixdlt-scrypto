use crate::system::node_modules::access_rules::*;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_substates::*;
use crate::types::*;
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::types::SubstateOffset;

#[derive(Debug)]
pub enum RENodeModuleInit {
    /* Type info */
    TypeInfo(TypeInfoSubstate),

    /* Metadata */
    Metadata(BTreeMap<SubstateOffset, RuntimeSubstate>),

    /* Access rules */
    MethodAccessRules(MethodAccessRulesSubstate),

    /* Royalty */
    ComponentRoyalty(
        ComponentRoyaltyConfigSubstate,
        ComponentRoyaltyAccumulatorSubstate,
    ),
}

impl RENodeModuleInit {
    pub fn to_substates(self) -> BTreeMap<SubstateOffset, RuntimeSubstate> {
        match self {
            RENodeModuleInit::Metadata(metadata_substates) => metadata_substates,
            RENodeModuleInit::MethodAccessRules(access_rules) => BTreeMap::from([(
                SubstateOffset::AccessRules(AccessRulesOffset::AccessRules),
                access_rules.into(),
            )]),
            RENodeModuleInit::TypeInfo(type_info) => BTreeMap::from([(
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
                type_info.into(),
            )]),
            RENodeModuleInit::ComponentRoyalty(config, accumulator) => BTreeMap::from([
                (
                    SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig),
                    config.into(),
                ),
                (
                    SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator),
                    accumulator.into(),
                ),
            ]),
        }
    }
}

#[derive(Debug)]
pub enum RENodeInit {
    GlobalObject(BTreeMap<SubstateOffset, RuntimeSubstate>),
    Object(BTreeMap<SubstateOffset, RuntimeSubstate>),
    KeyValueStore,
    IterableMap,
}

impl RENodeInit {
    pub fn to_substates(self) -> BTreeMap<SubstateOffset, RuntimeSubstate> {
        match self {
            RENodeInit::GlobalObject(object_substates) | RENodeInit::Object(object_substates) => {
                object_substates
            }
            RENodeInit::KeyValueStore => BTreeMap::new(),
            RENodeInit::IterableMap => BTreeMap::new(),
        }
    }
}
