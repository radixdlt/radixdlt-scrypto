use radix_substate_store_interface::interface::SubstateDatabase;

use crate::internal_prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TransactionValidator {
    pub(super) config: TransactionValidationConfig,
    pub(super) required_network_id: Option<u8>,
}

impl TransactionValidator {
    /// This is the best constructor to use, as it reads the configuration dynamically
    /// Note that the validator needs recreating every time a protocol update runs,
    /// as the config can get updated then.
    pub fn new(database: &impl SubstateDatabase, network_definition: &NetworkDefinition) -> Self {
        Self::new_with_static_config(
            TransactionValidationConfig::load(database),
            network_definition.id,
        )
    }

    pub fn new_for_latest_simulator() -> Self {
        Self::new_with_static_config(
            TransactionValidationConfig::latest(),
            NetworkDefinition::simulator().id,
        )
    }

    pub fn new_with_latest_config(network_definition: &NetworkDefinition) -> Self {
        Self::new_with_static_config(TransactionValidationConfig::latest(), network_definition.id)
    }

    pub fn new_with_static_config(config: TransactionValidationConfig, network_id: u8) -> Self {
        Self {
            config,
            required_network_id: Some(network_id),
        }
    }

    pub fn new_with_latest_config_network_agnostic() -> Self {
        Self::new_with_static_config_network_agnostic(TransactionValidationConfig::latest())
    }

    pub fn new_with_static_config_network_agnostic(config: TransactionValidationConfig) -> Self {
        Self {
            config,
            required_network_id: None,
        }
    }

    /// Will typically be [`Some`], but [`None`] if the validator is network-independent.
    pub fn network_id(&self) -> Option<u8> {
        self.required_network_id
    }

    pub fn config(&self) -> &TransactionValidationConfig {
        &self.config
    }

    pub fn preparation_settings(&self) -> &PreparationSettings {
        &self.config.preparation_settings
    }
}

// Concrete methods on `TransactionValidator` are implemented across
// other modules, such as `transaction_validator_v1`, to avoid this
// file growing to an unmanagable size.
