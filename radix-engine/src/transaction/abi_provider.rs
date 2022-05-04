use sbor::Type;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::crypto::hash;
use scrypto::engine::types::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;

use crate::engine::*;
use crate::errors::*;
use crate::ledger::*;
use crate::model::*;

/// An interface for exporting the ABI of a blueprint.
pub trait AbiProvider {
    /// Exports the ABI of a blueprint.
    fn export_abi(
        &self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> Result<abi::Blueprint, RuntimeError>;

    /// Exports the ABI of the blueprint, from which the given component is instantiated.
    fn export_abi_by_component(
        &self,
        component_address: ComponentAddress,
    ) -> Result<abi::Blueprint, RuntimeError>;
}

/// Provides ABIs for blueprints either installed during bootstrap or added manually.
pub struct BasicAbiProvider {
    substate_store: InMemorySubstateStore,
}

impl BasicAbiProvider {
    pub fn new() -> Self {
        Self {
            substate_store: InMemorySubstateStore::with_bootstrap(),
        }
    }

    pub fn with_package(
        &mut self,
        package_address: &PackageAddress,
        package: Package,
    ) -> &mut Self {
        let tx_hash = hash(self.substate_store.get_nonce().to_le_bytes());
        let mut id_gen = SubstateIdGenerator::new(tx_hash);

        self.substate_store
            .put_encoded_substate(package_address, &package, id_gen.next());
        self
    }
}

impl AbiProvider for BasicAbiProvider {
    fn export_abi(
        &self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> Result<abi::Blueprint, RuntimeError> {
        // Deterministic transaction context
        let mut ledger = self.substate_store.clone();
        let transaction_hash = hash([]);
        let mut track = Track::new(&mut ledger, transaction_hash, Vec::new());
        let package = track.get_package(&package_address).ok_or(RuntimeError::PackageNotFound(package_address))?;
        let output: (Type, Vec<abi::Function>, Vec<abi::Method>) = package
            .call_abi(blueprint_name)
            .and_then(|rtn| scrypto_decode(&rtn.raw).map_err(RuntimeError::AbiValidationError))?;

        // Return ABI
        Ok(abi::Blueprint {
            package_address: package_address.to_string(),
            blueprint_name: blueprint_name.to_owned(),
            functions: output.1,
            methods: output.2,
        })
    }

    fn export_abi_by_component(
        &self,
        component_address: ComponentAddress,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let component: Component = self
            .substate_store
            .get_decoded_substate(&component_address)
            .map(|(component, _)| component)
            .ok_or(RuntimeError::ComponentNotFound(component_address))?;
        self.export_abi(component.package_address(), component.blueprint_name())
    }
}
