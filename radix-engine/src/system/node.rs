use crate::system::node_modules::access_rules::*;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_substates::*;
use crate::types::*;
use radix_engine_interface::api::component::*;
use radix_engine_interface::types::SubstateKey;

#[derive(Debug)]
pub enum ModuleInit {
    /* Type info */
    TypeInfo(TypeInfoSubstate),

    /* Metadata */
    Metadata(BTreeMap<SubstateKey, RuntimeSubstate>),

    /* Access rules */
    AccessRules(MethodAccessRulesSubstate),

    /* Royalty */
    Royalty(
        ComponentRoyaltyConfigSubstate,
        ComponentRoyaltyAccumulatorSubstate,
    ),
}

impl ModuleInit {
    pub fn to_substates(self) -> BTreeMap<SubstateKey, RuntimeSubstate> {
        match self {
            ModuleInit::Metadata(metadata_substates) => metadata_substates,
            ModuleInit::AccessRules(access_rules) => {
                BTreeMap::from([(&AccessRulesOffset::AccessRules.into(), access_rules.into())])
            }
            ModuleInit::TypeInfo(type_info) => BTreeMap::from([(
                SubstateKey::TypeInfo(TypeInfoOffset::TypeInfo),
                type_info.into(),
            )]),
            ModuleInit::Royalty(config, accumulator) => BTreeMap::from([
                (
                    SubstateKey::Royalty(RoyaltyOffset::RoyaltyConfig),
                    config.into(),
                ),
                (
                    SubstateKey::Royalty(RoyaltyOffset::RoyaltyAccumulator),
                    accumulator.into(),
                ),
            ]),
        }
    }
}

#[derive(Debug)]
pub enum NodeInit {
    GlobalObject(BTreeMap<SubstateKey, RuntimeSubstate>),
    Object(BTreeMap<SubstateKey, RuntimeSubstate>),
    KeyValueStore,
}

impl RENodeInit {
    pub fn to_substates(self) -> BTreeMap<SubstateKey, RuntimeSubstate> {
        match self {
            RENodeInit::GlobalObject(object_substates) | RENodeInit::Object(object_substates) => {
                object_substates
            }
            RENodeInit::KeyValueStore => BTreeMap::new(),
        }
    }
}
