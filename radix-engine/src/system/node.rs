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
    MethodAccessRules(MethodAccessRulesSubstate),

    /* Royalty */
    ComponentRoyalty(
        ComponentRoyaltyConfigSubstate,
        ComponentRoyaltyAccumulatorSubstate,
    ),
}

impl ModuleInit {
    pub fn to_substates(self) -> HashMap<SubstateKey, RuntimeSubstate> {
        let mut substates = HashMap::<SubstateKey, RuntimeSubstate>::new();
        match self {
            ModuleInit::Metadata(metadata_substates) => {
                substates.extend(metadata_substates);
            }
            ModuleInit::MethodAccessRules(access_rules) => {
                substates.insert(AccessRulesOffset::AccessRules.into(), access_rules.into());
            }
            ModuleInit::TypeInfo(type_info) => {
                substates.insert(TypeInfoOffset::TypeInfo.into(), type_info.into());
            }
            ModuleInit::ComponentRoyalty(config, accumulator) => {
                substates.insert(RoyaltyOffset::Royalty.into(), config.into());
                substates.insert(RoyaltyOffset::Royalty.into(), accumulator.into());
            }
        }

        substates
    }
}

#[derive(Debug)]
pub enum NodeInit {
    GlobalObject(BTreeMap<SubstateKey, RuntimeSubstate>),
    Object(BTreeMap<SubstateKey, RuntimeSubstate>),
    KeyValueStore,
}

impl NodeInit {
    pub fn to_substates(self) -> HashMap<SubstateKey, RuntimeSubstate> {
        let mut substates = HashMap::<SubstateKey, RuntimeSubstate>::new();
        match self {
            NodeInit::GlobalObject(object_substates) | NodeInit::Object(object_substates) => {
                substates.extend(object_substates);
            }
            NodeInit::KeyValueStore => {}
        };

        substates
    }
}
