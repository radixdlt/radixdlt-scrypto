use crate::internal_prelude::*;
use radix_substate_store_interface::interface::{SubstateDatabase, SubstateDatabaseExtensions};

define_single_versioned! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
    pub TransactionValidationConfigurationSubstate(TransactionValidationConfigurationVersions) => TransactionValidationConfig = TransactionValidationConfigV1,
    outer_attributes: [
        #[derive(ScryptoSborAssertion)]
        #[sbor_assert(backwards_compatible(
            cuttlefish = "FILE:transaction_validation_configuration_substate_cuttlefish_schema.bin",
        ))]
    ]
}

impl TransactionValidationConfig {
    pub fn load(database: &impl SubstateDatabase) -> Self {
        database
            .get_substate::<TransactionValidationConfigurationSubstate>(
                TRANSACTION_TRACKER,
                BOOT_LOADER_PARTITION,
                BootLoaderField::TransactionValidationConfiguration,
            )
            .map(|s| s.fully_update_and_into_latest_version())
            .unwrap_or_else(|| Self::babylon())
    }

    pub(crate) fn allow_notary_to_duplicate_signer(&self, version: TransactionVersion) -> bool {
        match version {
            TransactionVersion::V1 => self.v1_transactions_allow_notary_to_duplicate_signer,
            TransactionVersion::V2 => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionVersion {
    V1,
    V2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub struct TransactionValidationConfigV1 {
    /// Signer signatures only, not including notary signature
    pub max_signer_signatures_per_intent: usize,
    pub max_references_per_intent: usize,
    pub min_tip_percentage: u16,
    pub max_tip_percentage: u16,
    pub max_epoch_range: u64,
    pub max_instructions: usize,
    pub message_validation: MessageValidationConfig,
    pub v1_transactions_allow_notary_to_duplicate_signer: bool,
    pub preparation_settings: PreparationSettingsV1,
    pub manifest_validation: ManifestValidationRuleset,
    // V2 settings
    pub v2_transactions_allowed: bool,
    pub min_tip_basis_points: u32,
    pub max_tip_basis_points: u32,
    /// A setting of N here allows a total depth of N + 1 if you
    /// include the root transaction intent.
    pub max_subintent_depth: usize,
    pub max_total_signature_validations: usize,
    pub max_total_references: usize,
}

impl TransactionValidationConfig {
    pub const fn latest() -> Self {
        Self::cuttlefish()
    }

    pub const fn babylon() -> Self {
        Self {
            max_signer_signatures_per_intent: 16,
            max_references_per_intent: usize::MAX,
            min_tip_percentage: 0,
            max_tip_percentage: u16::MAX,
            max_instructions: usize::MAX,
            // ~30 days given 5 minute epochs
            max_epoch_range: 12 * 24 * 30,
            v1_transactions_allow_notary_to_duplicate_signer: true,
            manifest_validation: ManifestValidationRuleset::BabylonBasicValidator,
            message_validation: MessageValidationConfig::babylon(),
            preparation_settings: PreparationSettings::babylon(),
            // V2-only settings
            v2_transactions_allowed: true,
            max_subintent_depth: 0,
            min_tip_basis_points: 0,
            max_tip_basis_points: 0,
            max_total_signature_validations: usize::MAX,
            max_total_references: usize::MAX,
        }
    }

    pub const fn cuttlefish() -> Self {
        Self {
            max_references_per_intent: 512,
            v2_transactions_allowed: true,
            max_subintent_depth: 3,
            min_tip_basis_points: 0,
            max_instructions: 1000,
            manifest_validation: ManifestValidationRuleset::Interpreter(
                InterpreterValidationRulesetSpecifier::Cuttlefish,
            ),
            // Tip of 100 times the cost of a transaction
            max_tip_basis_points: 100 * 10000,
            preparation_settings: PreparationSettings::cuttlefish(),
            max_total_signature_validations: 64,
            max_total_references: 512,
            ..Self::babylon()
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub enum ManifestValidationRuleset {
    BabylonBasicValidator,
    Interpreter(InterpreterValidationRulesetSpecifier),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub struct MessageValidationConfig {
    pub max_plaintext_message_length: usize,
    pub max_encrypted_message_length: usize,
    pub max_mime_type_length: usize,
    pub max_decryptors: usize,
}

impl MessageValidationConfig {
    pub const fn latest() -> Self {
        Self::babylon()
    }

    pub const fn babylon() -> Self {
        Self {
            max_plaintext_message_length: 2048,
            max_mime_type_length: 128,
            max_encrypted_message_length: 2048 + 12 + 16, // Account for IV and MAC - see AesGcmPayload
            max_decryptors: 20,
        }
    }
}

impl Default for MessageValidationConfig {
    fn default() -> Self {
        Self::latest()
    }
}
