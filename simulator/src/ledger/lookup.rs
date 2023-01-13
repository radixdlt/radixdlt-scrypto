use radix_engine::engine::ScryptoInterpreter;
use radix_engine::ledger::ReadableSubstateStore;
use radix_engine::model::GlobalAddressSubstate;
use radix_engine::types::{
    GlobalAddress, GlobalOffset, NonFungibleIdTypeId, RENodeId, ResourceAddress,
    ResourceManagerOffset, ResourceType, SubstateId, SubstateOffset,
};
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine_stores::rocks_db::RadixEngineDB;

use crate::resim::get_data_dir;

pub fn lookup_non_fungible_id_type(
    resource_address: &ResourceAddress,
) -> Result<NonFungibleIdTypeId, LedgerLookupError> {
    let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
    let substate_store = RadixEngineDB::with_bootstrap(
        get_data_dir().map_err(|_| LedgerLookupError::FailedToGetLocalSubstateStorePath)?,
        &scrypto_interpreter,
    );

    // Reading the global address substate to get the ResourceManagerId from there
    let resource_manager_id = {
        let global_address = GlobalAddress::Resource(*resource_address);
        let node_id = RENodeId::Global(global_address);
        let offset = SubstateOffset::Global(GlobalOffset::Global);
        let substate_id = SubstateId(node_id, offset);
        let global_address_substate = substate_store.get_substate(&substate_id).map_or(
            Err(LedgerLookupError::GlobalAddressNotFound(global_address)),
            |value| Ok(value.substate.global().clone()),
        )?;

        match global_address_substate {
            GlobalAddressSubstate::Resource(id) => id,
            _ => panic!(
                "A global resource address can not point to anything other than a resource manager"
            ),
        }
    };

    // Reading the resource manager substate from the substate store and getting the resource type
    let resource_type = {
        let node_id = RENodeId::ResourceManager(resource_manager_id);
        let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
        let substate_id = SubstateId(node_id, offset);
        substate_store
            .get_substate(&substate_id)
            .expect("Impossible case! Global address is valid but resource manager id is not valid")
            .substate
            .resource_manager()
            .resource_type
    };

    // Getting the non-fungible id type for this resource if it is a non-fungible resource
    match resource_type {
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
    ResourceIsNotNonFungible,
    FailedToGetLocalSubstateStorePath,
}

// ======
// Tests
// ======

#[cfg(test)]
mod tests {
    use radix_engine::types::{NonFungibleIdTypeId, ECDSA_SECP256K1_TOKEN};
    use serial_test::serial;

    use super::lookup_non_fungible_id_type;

    #[test]
    #[serial]
    pub fn non_fungible_id_type_ledger_lookup_matches_expected() {
        // Arrange
        let resource_address = ECDSA_SECP256K1_TOKEN;

        // Act
        let non_fungible_id_type = lookup_non_fungible_id_type(&resource_address).unwrap();

        // Assert
        assert_eq!(non_fungible_id_type, NonFungibleIdTypeId::Bytes)
    }
}
