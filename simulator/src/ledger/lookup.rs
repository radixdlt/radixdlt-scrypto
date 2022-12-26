use radix_engine::engine::ScryptoInterpreter;
use radix_engine::ledger::ReadableSubstateStore;
use radix_engine::model::GlobalAddressSubstate;
use radix_engine::types::{
    GlobalAddress, GlobalOffset, NonFungibleIdType, RENodeId, ResourceAddress, ResourceManagerId,
    ResourceManagerOffset, ResourceType, SubstateId, SubstateOffset,
};
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine_stores::rocks_db::RadixEngineDB;

use crate::resim::get_data_dir;

pub fn lookup_id_type(
    resource_address: &ResourceAddress,
) -> Result<NonFungibleIdType, LedgerLookupError> {
    let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
    let substate_store = RadixEngineDB::with_bootstrap(
        get_data_dir().map_err(|_| LedgerLookupError::FailedToGetLocalSubstateStorePath)?,
        &scrypto_interpreter,
    );

    // Reading the global address substate to get the ResourceManagerId from there
    let global_address = GlobalAddress::Resource(*resource_address);
    let node_id = RENodeId::Global(global_address);
    let offset = SubstateOffset::Global(GlobalOffset::Global);
    let substate_id = SubstateId(node_id, offset);
    let global_address_substate = substate_store.get_substate(&substate_id).map_or(
        Err(LedgerLookupError::GlobalAddressNotFound(global_address)),
        |value| Ok(value.substate.global().clone()),
    )?;

    let resource_manager_id = match global_address_substate {
        GlobalAddressSubstate::Resource(id) => id,
        _ => panic!(
            "A global resource address can not point to anything other than a resource manager"
        ),
    };

    // Reading the resource manager substate from the substate store
    let node_id = RENodeId::ResourceManager(resource_manager_id);
    let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
    let substate_id = SubstateId(node_id, offset);
    let resource_manager_substate = substate_store.get_substate(&substate_id).map_or(
        Err(LedgerLookupError::ResourceManagerNotFound(
            global_address,
            resource_manager_id,
        )),
        |value| Ok(value.substate.resource_manager().clone()),
    )?;

    // Getting the non-fungible id type for this resource if it is a non-fungible resource
    match resource_manager_substate.resource_type {
        ResourceType::NonFungible { id_type } => Ok(id_type),
        _ => Err(LedgerLookupError::ResourceIsNotNonFungible),
    }
}

// =======
// Errors
// =======

#[derive(Debug, Clone)]
pub enum LedgerLookupError {
    GlobalAddressNotFound(GlobalAddress),
    ResourceManagerNotFound(GlobalAddress, ResourceManagerId),
    ResourceIsNotNonFungible,
    FailedToGetLocalSubstateStorePath,
}

// ======
// Tests
// ======

#[cfg(test)]
mod tests {
    use radix_engine::types::{NonFungibleIdType, ECDSA_SECP256K1_TOKEN};

    use super::lookup_id_type;

    #[test]
    pub fn non_fungible_id_type_ledger_lookup_matches_expected() {
        // Arrange
        let resource_address = ECDSA_SECP256K1_TOKEN;

        // Act
        let non_fungible_id_type = lookup_id_type(&resource_address).unwrap();

        // Assert
        assert_eq!(non_fungible_id_type, NonFungibleIdType::Bytes)
    }
}
