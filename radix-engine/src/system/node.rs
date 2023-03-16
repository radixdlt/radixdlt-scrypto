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
    pub fn to_substates(self) -> HashMap<SubstateOffset, RuntimeSubstate> {
        let mut substates = HashMap::<SubstateOffset, RuntimeSubstate>::new();
        match self {
            RENodeModuleInit::Metadata(metadata_substates) => {
                substates.extend(metadata_substates);
            }
            RENodeModuleInit::MethodAccessRules(access_rules) => {
                substates.insert(
                    SubstateOffset::AccessRules(AccessRulesOffset::AccessRules),
                    access_rules.into(),
                );
            }
            RENodeModuleInit::TypeInfo(type_info) => {
                substates.insert(
                    SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
                    type_info.into(),
                );
            }
            RENodeModuleInit::ComponentRoyalty(config, accumulator) => {
                substates.insert(
                    SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig),
                    config.into(),
                );
                substates.insert(
                    SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator),
                    accumulator.into(),
                );
            }
        }

        substates
    }
}

#[derive(Debug)]
pub enum RENodeInit {
    GlobalObject(BTreeMap<SubstateOffset, RuntimeSubstate>),
    Object(BTreeMap<SubstateOffset, RuntimeSubstate>),
    KeyValueStore,
    NonFungibleStore,
}

impl RENodeInit {
    pub fn to_substates(self) -> HashMap<SubstateOffset, RuntimeSubstate> {
        let mut substates = HashMap::<SubstateOffset, RuntimeSubstate>::new();
        match self {
            RENodeInit::GlobalObject(object_substates) | RENodeInit::Object(object_substates) => {
                substates.extend(object_substates);
            }
            RENodeInit::KeyValueStore | RENodeInit::NonFungibleStore => {}
        };

        substates
    }
}
