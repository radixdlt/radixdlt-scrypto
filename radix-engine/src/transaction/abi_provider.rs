use scrypto::abi;
use scrypto::buffer::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use scrypto::utils::*;

use crate::engine::*;
use crate::ledger::*;
use crate::model::*;

/// An interface for exporting the ABI of a blueprint.
pub trait AbiProvider {
    /// Exports the ABI of a blueprint.
    fn export_abi<S: AsRef<str>>(
        &self,
        package_address: Address,
        blueprint_name: S,
    ) -> Result<abi::Blueprint, RuntimeError>;

    /// Exports the ABI of the blueprint, from which the given component is instantiated.
    fn export_abi_component(
        &self,
        component_address: Address,
    ) -> Result<abi::Blueprint, RuntimeError>;
}

/// Provides ABIs for blueprints either installed during bootstrap or added manually.
pub struct BasicAbiProvider {
    ledger: InMemoryLedger,
    trace: bool,
}

impl BasicAbiProvider {
    pub fn new(trace: bool) -> Self {
        Self {
            ledger: InMemoryLedger::with_bootstrap(),
            trace,
        }
    }

    pub fn with_package(&mut self, address: Address, code: Vec<u8>) -> &mut Self {
        self.ledger.put_package(address, Package::new(code));
        self
    }

    pub fn with_component(
        &mut self,
        component_address: Address,
        package_address: Address,
        blueprint_name: String,
        state: Vec<u8>,
    ) -> &mut Self {
        self.ledger.put_component(
            component_address,
            Component::new(package_address, blueprint_name, state),
        );
        self
    }
}

impl AbiProvider for BasicAbiProvider {
    fn export_abi<S: AsRef<str>>(
        &self,
        package_address: Address,
        blueprint_name: S,
    ) -> Result<abi::Blueprint, RuntimeError> {
        // Deterministic transaction context
        let mut ledger = self.ledger.clone();
        let current_epoch = 0;
        let transaction_hash = sha256([]);

        // Start a process and run abi generator
        let mut track = Track::new(&mut ledger, current_epoch, transaction_hash, Vec::new());
        let mut proc = track.start_process(self.trace);
        let output: (Vec<abi::Function>, Vec<abi::Method>) = proc
            .call_abi(package_address, blueprint_name.as_ref())
            .and_then(|rtn| scrypto_decode(&rtn.raw).map_err(RuntimeError::AbiValidationError))?;

        // Return ABI
        Ok(abi::Blueprint {
            package: package_address.to_string(),
            name: blueprint_name.as_ref().to_owned(),
            functions: output.0,
            methods: output.1,
        })
    }

    fn export_abi_component(
        &self,
        component_address: Address,
    ) -> Result<abi::Blueprint, RuntimeError> {
        let component = self
            .ledger
            .get_component(component_address)
            .ok_or(RuntimeError::ComponentNotFound(component_address))?;
        self.export_abi(
            component.package_address(),
            component.blueprint_name().to_owned(),
        )
    }
}
