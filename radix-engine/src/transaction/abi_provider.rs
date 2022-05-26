use sbor::rust::borrow::ToOwned;
use sbor::rust::string::ToString;
use scrypto::abi;
use scrypto::engine::types::*;

use crate::engine::*;
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
pub struct BasicAbiProvider<'l, L: ReadableSubstateStore> {
    substate_store: &'l L,
}

impl<'l, L: ReadableSubstateStore> BasicAbiProvider<'l, L> {
    pub fn new(substate_store: &'l L) -> Self {
        Self { substate_store }
    }
}

impl<'l, L: ReadableSubstateStore> AbiProvider for BasicAbiProvider<'l, L> {
    fn export_abi(
        &self,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let package: ValidatedPackage = self
            .substate_store
            .get_decoded_substate(&package_address)
            .map(|(package, _)| package)
            .ok_or(RuntimeError::PackageNotFound(package_address))?;

        let abi = package
            .blueprint_abi(blueprint_name)
            .ok_or(RuntimeError::BlueprintNotFound(
                package_address,
                blueprint_name.to_owned(),
            ))?;

        // Return ABI
        Ok(abi::Blueprint {
            package_address: package_address.to_string(),
            blueprint_name: blueprint_name.to_owned(),
            functions: abi.1.clone(),
            methods: abi.2.clone(),
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
