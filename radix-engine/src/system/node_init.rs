use crate::system::node_modules::access_rules::*;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::types::*;
use radix_engine_interface::api::node_modules::royalty::*;
use radix_engine_interface::types::SubstateKey;

#[derive(Debug)]
pub enum ModuleInit {
    /* Type info */
    TypeInfo(TypeInfoSubstate),

    /* Metadata */
    Metadata(BTreeMap<SubstateKey, IndexedScryptoValue>),

    /* Access rules */
    AccessRules(MethodAccessRulesSubstate),

    /* Royalty */
    Royalty(
        ComponentRoyaltyConfigSubstate,
        ComponentRoyaltyAccumulatorSubstate,
    ),
}

impl ModuleInit {
    pub fn to_substates(self) -> BTreeMap<SubstateKey, IndexedScryptoValue> {
        match self {
            ModuleInit::Metadata(metadata_substates) => metadata_substates,
            ModuleInit::AccessRules(access_rules) => BTreeMap::from([(
                AccessRulesField::AccessRules.into(),
                IndexedScryptoValue::from_typed(&access_rules),
            )]),
            ModuleInit::TypeInfo(type_info) => BTreeMap::from([(
                TypeInfoField::TypeInfo.into(),
                IndexedScryptoValue::from_typed(&type_info),
            )]),
            ModuleInit::Royalty(config, accumulator) => BTreeMap::from([
                (
                    RoyaltyField::RoyaltyConfig.into(),
                    IndexedScryptoValue::from_typed(&config),
                ),
                (
                    RoyaltyField::RoyaltyAccumulator.into(),
                    IndexedScryptoValue::from_typed(&accumulator),
                ),
            ]),
        }
    }
}
