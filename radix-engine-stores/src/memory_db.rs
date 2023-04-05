use crate::interface::*;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::types::*;
use sbor::rust::ops::Bound::Included;
use sbor::rust::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub struct InMemorySubstateDatabase {
    configs: BTreeMap<ModuleId, ModuleConfig>,
    substates: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl InMemorySubstateDatabase {
    pub fn standard() -> Self {
        Self {
            configs: btreemap!(
                TypedModuleId::TypeInfo.into() => ModuleConfig {
                    iteration_enabled: false,
                },
                TypedModuleId::ObjectState.into() => ModuleConfig {
                    iteration_enabled: true,
                },
                TypedModuleId::Metadata.into() => ModuleConfig {
                    iteration_enabled: true,
                },
                TypedModuleId::Royalty.into() => ModuleConfig {
                    iteration_enabled: false,
                },
                TypedModuleId::AccessRules.into() => ModuleConfig {
                    iteration_enabled: false,
                },
            ),
            substates: btreemap!(),
        }
    }
}

impl SubstateDatabase for InMemorySubstateDatabase {
    fn get_substate(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Result<Option<(Vec<u8>, u32)>, GetSubstateError> {
        if !self.configs.contains_key(&module_id) {
            return Err(GetSubstateError::UnknownModuleId);
        }

        let key = encode_substate_id(node_id, module_id, substate_key);
        let value = self
            .substates
            .get(&key)
            .map(|x| scrypto_decode::<(Vec<u8>, u32)>(x).expect("Failed to decode value"));
        Ok(value)
    }

    fn list_substates(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Result<(Vec<(SubstateKey, Vec<u8>)>, Hash), ListSubstatesError> {
        match self.configs.get(&module_id) {
            None => {
                return Err(ListSubstatesError::UnknownModuleId);
            }
            Some(config) => {
                if !config.iteration_enabled {
                    return Err(ListSubstatesError::IterationNotAllowed);
                }
            }
        }

        let start = encode_substate_id(node_id, module_id, &SubstateKey::min());
        let end = encode_substate_id(node_id, module_id, &SubstateKey::max());
        let mut substates = Vec::<(SubstateKey, Vec<u8>)>::new();

        for (k, v) in self.substates.range((Included(start), Included(end))) {
            let (_, _, substate_key) = decode_substate_id(k).expect("Failed to decode substate ID");
            let value = scrypto_decode::<(Vec<u8>, u32)>(v).expect("Failed to decode value");
            substates.push((substate_key, value.0));
        }

        Ok((substates, Hash([0; Hash::LENGTH])))
    }
}

impl CommittableSubstateDatabase for InMemorySubstateDatabase {
    fn commit(&mut self, state_changes: &StateUpdates) -> Result<(), CommitError> {
        for ((node_id, module_id, substate_key), substate_change) in &state_changes.substate_changes
        {
            let substate_id = encode_substate_id(node_id, *module_id, substate_key);
            let previous_version = match self.get_substate(node_id, *module_id, substate_key) {
                Ok(x) => x.map(|a| a.1),
                Err(GetSubstateError::UnknownModuleId) => {
                    return Err(CommitError::UnknownModuleId);
                }
            };
            match substate_change {
                StateUpdate::Upsert(substate_value, _) => {
                    self.substates.insert(
                        substate_id,
                        scrypto_encode(&(
                            substate_value,
                            match previous_version {
                                Some(v) => v + 1,
                                None => 0u32,
                            },
                        ))
                        .unwrap(),
                    );
                }
            }
        }
        Ok(())
    }
}
