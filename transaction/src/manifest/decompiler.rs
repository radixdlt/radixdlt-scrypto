use crate::data::*;
use crate::model::*;
use crate::validation::*;
use radix_engine_common::native_addresses::PACKAGE_PACKAGE;
use radix_engine_common::prelude::CONSENSUS_MANAGER;
use radix_engine_interface::address::AddressBech32Encoder;
use radix_engine_interface::api::node_modules::auth::{
    ACCESS_RULES_LOCK_OWNER_ROLE_IDENT, ACCESS_RULES_LOCK_ROLE_IDENT,
    ACCESS_RULES_SET_AND_LOCK_OWNER_ROLE_IDENT, ACCESS_RULES_SET_AND_LOCK_ROLE_IDENT,
    ACCESS_RULES_SET_OWNER_ROLE_IDENT, ACCESS_RULES_SET_ROLE_IDENT,
};
use radix_engine_interface::api::node_modules::metadata::METADATA_SET_IDENT;
use radix_engine_interface::api::node_modules::metadata::{
    METADATA_LOCK_IDENT, METADATA_REMOVE_IDENT,
};
use radix_engine_interface::api::node_modules::royalty::{
    COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT, COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT,
    COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
};
use radix_engine_interface::blueprints::access_controller::{
    ACCESS_CONTROLLER_BLUEPRINT, ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT,
};
use radix_engine_interface::blueprints::account::{
    ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_ADVANCED_IDENT, ACCOUNT_CREATE_IDENT,
};
use radix_engine_interface::blueprints::consensus_manager::CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT;
use radix_engine_interface::blueprints::identity::{
    IDENTITY_BLUEPRINT, IDENTITY_CREATE_ADVANCED_IDENT, IDENTITY_CREATE_IDENT,
};
use radix_engine_interface::blueprints::package::PACKAGE_BLUEPRINT;
use radix_engine_interface::blueprints::package::PACKAGE_CLAIM_ROYALTIES_IDENT;
use radix_engine_interface::blueprints::package::PACKAGE_PUBLISH_WASM_ADVANCED_IDENT;
use radix_engine_interface::blueprints::package::PACKAGE_PUBLISH_WASM_IDENT;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
    FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
    NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT,
    VAULT_FREEZE_IDENT, VAULT_RECALL_IDENT, VAULT_UNFREEZE_IDENT,
};
use radix_engine_interface::constants::{
    ACCESS_CONTROLLER_PACKAGE, ACCOUNT_PACKAGE, IDENTITY_PACKAGE, RESOURCE_PACKAGE,
};
use radix_engine_interface::data::manifest::model::*;
use radix_engine_interface::data::manifest::*;
use radix_engine_interface::network::NetworkDefinition;
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use sbor::*;

#[derive(Debug, Clone)]
pub enum DecompileError {
    InvalidArguments,
    EncodeError(EncodeError),
    DecodeError(DecodeError),
    FormattingError(fmt::Error),
    ValueConversionError(RustToManifestValueError),
}

impl From<EncodeError> for DecompileError {
    fn from(error: EncodeError) -> Self {
        Self::EncodeError(error)
    }
}

impl From<DecodeError> for DecompileError {
    fn from(error: DecodeError) -> Self {
        Self::DecodeError(error)
    }
}

impl From<fmt::Error> for DecompileError {
    fn from(error: fmt::Error) -> Self {
        Self::FormattingError(error)
    }
}

impl From<RustToManifestValueError> for DecompileError {
    fn from(error: RustToManifestValueError) -> Self {
        Self::ValueConversionError(error)
    }
}

#[derive(Default)]
pub struct DecompilationContext<'a> {
    pub address_bech32_encoder: Option<&'a AddressBech32Encoder>,
    pub id_allocator: ManifestIdAllocator,
    pub bucket_names: NonIterMap<ManifestBucket, String>,
    pub proof_names: NonIterMap<ManifestProof, String>,
    pub address_reservation_names: NonIterMap<ManifestAddressReservation, String>,
    pub address_names: NonIterMap<u32, String>,
}

impl<'a> DecompilationContext<'a> {
    pub fn new(address_bech32_encoder: &'a AddressBech32Encoder) -> Self {
        Self {
            address_bech32_encoder: Some(address_bech32_encoder),
            ..Default::default()
        }
    }

    pub fn new_with_optional_network(
        address_bech32_encoder: Option<&'a AddressBech32Encoder>,
    ) -> Self {
        Self {
            address_bech32_encoder,
            ..Default::default()
        }
    }

    pub fn for_value_display(&'a self) -> ManifestDecompilationDisplayContext<'a> {
        ManifestDecompilationDisplayContext::with_bech32_and_names(
            self.address_bech32_encoder,
            &self.bucket_names,
            &self.proof_names,
            &self.address_reservation_names,
            &self.address_names,
        )
        .with_multi_line(4, 4)
    }

    pub fn new_bucket(&mut self) -> ManifestBucket {
        let id = self.id_allocator.new_bucket_id();
        let name = format!("bucket{}", self.bucket_names.len() + 1);
        self.bucket_names.insert(id, name.clone());
        id
    }

    pub fn new_proof(&mut self) -> ManifestProof {
        let id = self.id_allocator.new_proof_id();
        let name = format!("proof{}", self.proof_names.len() + 1);
        self.proof_names.insert(id, name.clone());
        id
    }

    pub fn new_address_reservation(&mut self) -> ManifestAddressReservation {
        let id = self.id_allocator.new_address_reservation_id();
        let name = format!("reservation{}", self.address_reservation_names.len() + 1);
        self.address_reservation_names.insert(id, name.clone());
        id
    }

    pub fn new_address(&mut self) -> ManifestAddress {
        let id = self.id_allocator.new_address_id();
        let name = format!("address{}", self.address_names.len() + 1);
        self.address_names.insert(id, name.clone());
        ManifestAddress::Named(id)
    }

    /// Allocate addresses before transaction, for system transactions only.
    pub fn preallocate_addresses(&mut self, n: u32) {
        for _ in 0..n {
            self.new_address();
        }
    }
}

/// Contract: if the instructions are from a validated notarized transaction, no error
/// should be returned.
pub fn decompile(
    instructions: &[InstructionV1],
    network: &NetworkDefinition,
) -> Result<String, DecompileError> {
    let address_bech32_encoder = AddressBech32Encoder::new(network);
    let mut buf = String::new();
    let mut context = DecompilationContext::new(&address_bech32_encoder);
    for inst in instructions {
        decompile_instruction(&mut buf, inst, &mut context)?;
    }

    Ok(buf)
}

pub fn decompile_instruction<F: fmt::Write>(
    f: &mut F,
    instruction: &InstructionV1,
    context: &mut DecompilationContext,
) -> Result<(), DecompileError> {
    let (display_name, display_parameters) = match instruction {
        InstructionV1::TakeFromWorktop {
            resource_address,
            amount,
        } => {
            let bucket = context.new_bucket();
            (
                "TAKE_FROM_WORKTOP",
                to_manifest_value(&(resource_address, amount, bucket))?,
            )
        }
        InstructionV1::TakeNonFungiblesFromWorktop {
            ids,
            resource_address,
        } => {
            let bucket = context.new_bucket();
            (
                "TAKE_NON_FUNGIBLES_FROM_WORKTOP",
                to_manifest_value(&(resource_address, ids, bucket))?,
            )
        }
        InstructionV1::TakeAllFromWorktop { resource_address } => {
            let bucket = context.new_bucket();
            (
                "TAKE_ALL_FROM_WORKTOP",
                to_manifest_value(&(resource_address, bucket))?,
            )
        }
        InstructionV1::ReturnToWorktop { bucket_id } => {
            ("RETURN_TO_WORKTOP", to_manifest_value(&(bucket_id,))?)
        }
        InstructionV1::AssertWorktopContainsAny { resource_address } => (
            "ASSERT_WORKTOP_CONTAINS_ANY",
            to_manifest_value(&(resource_address))?,
        ),
        InstructionV1::AssertWorktopContains {
            amount,
            resource_address,
        } => (
            "ASSERT_WORKTOP_CONTAINS",
            to_manifest_value(&(resource_address, amount))?,
        ),
        InstructionV1::AssertWorktopContainsNonFungibles {
            resource_address,
            ids,
        } => (
            "ASSERT_WORKTOP_CONTAINS_NON_FUNGIBLES",
            to_manifest_value(&(resource_address, ids))?,
        ),
        InstructionV1::PopFromAuthZone => {
            let proof = context.new_proof();
            ("POP_FROM_AUTH_ZONE", to_manifest_value(&(proof,))?)
        }
        InstructionV1::PushToAuthZone { proof_id } => {
            ("PUSH_TO_AUTH_ZONE", to_manifest_value(&(proof_id,))?)
        }
        InstructionV1::ClearAuthZone => ("CLEAR_AUTH_ZONE", to_manifest_value_and_unwrap!(&())),
        InstructionV1::CreateProofFromAuthZone { resource_address } => {
            let proof = context.new_proof();
            (
                "CREATE_PROOF_FROM_AUTH_ZONE",
                to_manifest_value(&(resource_address, proof))?,
            )
        }
        InstructionV1::CreateProofFromAuthZoneOfAmount {
            resource_address,
            amount,
        } => {
            let proof = context.new_proof();

            (
                "CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT",
                to_manifest_value(&(resource_address, amount, proof))?,
            )
        }
        InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
            resource_address,
            ids,
        } => {
            let proof = context.new_proof();
            (
                "CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES",
                to_manifest_value(&(resource_address, ids, proof))?,
            )
        }
        InstructionV1::CreateProofFromAuthZoneOfAll { resource_address } => {
            let proof = context.new_proof();
            (
                "CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL",
                to_manifest_value(&(resource_address, proof))?,
            )
        }

        InstructionV1::ClearSignatureProofs => ("CLEAR_SIGNATURE_PROOFS", to_manifest_value(&())?),

        InstructionV1::CreateProofFromBucket { bucket_id } => {
            let proof = context.new_proof();
            (
                "CREATE_PROOF_FROM_BUCKET",
                to_manifest_value(&(bucket_id, proof))?,
            )
        }

        InstructionV1::CreateProofFromBucketOfAmount { bucket_id, amount } => {
            let proof = context.new_proof();
            (
                "CREATE_PROOF_FROM_BUCKET_OF_AMOUNT",
                to_manifest_value(&(bucket_id, amount, proof))?,
            )
        }
        InstructionV1::CreateProofFromBucketOfNonFungibles { bucket_id, ids } => {
            let proof = context.new_proof();
            (
                "CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES",
                to_manifest_value(&(bucket_id, ids, proof))?,
            )
        }
        InstructionV1::CreateProofFromBucketOfAll { bucket_id } => {
            let proof = context.new_proof();
            (
                "CREATE_PROOF_FROM_BUCKET_OF_ALL",
                to_manifest_value(&(bucket_id, proof))?,
            )
        }
        InstructionV1::BurnResource { bucket_id } => {
            ("BURN_RESOURCE", to_manifest_value(&(bucket_id,))?)
        }
        InstructionV1::CloneProof { proof_id } => {
            let proof_id2 = context.new_proof();
            ("CLONE_PROOF", to_manifest_value(&(proof_id, proof_id2))?)
        }
        InstructionV1::DropProof { proof_id } => ("DROP_PROOF", to_manifest_value(&(proof_id,))?),
        InstructionV1::CallFunction {
            package_address,
            blueprint_name,
            function_name,
            args,
        } => {
            let mut fields = Vec::new();
            let name = match (
                package_address,
                blueprint_name.as_str(),
                function_name.as_str(),
            ) {
                (package_address, PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_WASM_IDENT)
                    if package_address.is_static_global_package_of(&PACKAGE_PACKAGE) =>
                {
                    "PUBLISH_PACKAGE"
                }
                (package_address, PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_WASM_ADVANCED_IDENT)
                    if package_address.is_static_global_package_of(&PACKAGE_PACKAGE) =>
                {
                    "PUBLISH_PACKAGE_ADVANCED"
                }
                (package_address, ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_ADVANCED_IDENT)
                    if package_address.is_static_global_package_of(&ACCOUNT_PACKAGE) =>
                {
                    "CREATE_ACCOUNT_ADVANCED"
                }
                (package_address, ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_IDENT)
                    if package_address.is_static_global_package_of(&ACCOUNT_PACKAGE) =>
                {
                    "CREATE_ACCOUNT"
                }
                (package_address, IDENTITY_BLUEPRINT, IDENTITY_CREATE_ADVANCED_IDENT)
                    if package_address.is_static_global_package_of(&IDENTITY_PACKAGE) =>
                {
                    "CREATE_IDENTITY_ADVANCED"
                }
                (package_address, IDENTITY_BLUEPRINT, IDENTITY_CREATE_IDENT)
                    if package_address.is_static_global_package_of(&IDENTITY_PACKAGE) =>
                {
                    "CREATE_IDENTITY"
                }
                (
                    package_address,
                    ACCESS_CONTROLLER_BLUEPRINT,
                    ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT,
                ) if package_address.is_static_global_package_of(&ACCESS_CONTROLLER_PACKAGE) => {
                    "CREATE_ACCESS_CONTROLLER"
                }
                (
                    package_address,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                ) if package_address.is_static_global_package_of(&RESOURCE_PACKAGE) => {
                    "CREATE_FUNGIBLE_RESOURCE"
                }
                (
                    package_address,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                ) if package_address.is_static_global_package_of(&RESOURCE_PACKAGE) => {
                    "CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY"
                }
                (
                    package_address,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                ) if package_address.is_static_global_package_of(&RESOURCE_PACKAGE) => {
                    "CREATE_NON_FUNGIBLE_RESOURCE"
                }
                (
                    package_address,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                ) if package_address.is_static_global_package_of(&RESOURCE_PACKAGE) => {
                    "CREATE_NON_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY"
                }
                _ => {
                    fields.push(package_address.to_instruction_argument());
                    fields.push(to_manifest_value(blueprint_name)?);
                    fields.push(to_manifest_value(function_name)?);
                    "CALL_FUNCTION"
                }
            };

            if let Value::Tuple { fields: arg_fields } = args {
                fields.extend(arg_fields.clone());
            } else {
                return Err(DecompileError::InvalidArguments);
            }

            let parameters = Value::Tuple { fields };
            (name, parameters)
        }
        InstructionV1::CallMethod {
            address,
            method_name,
            args,
        } => {
            let mut fields = Vec::new();
            let name = match (address, method_name.as_str()) {
                // Nb - For Main method call, we also check the address type to avoid name clashing.

                /* Package */
                (address, PACKAGE_CLAIM_ROYALTIES_IDENT) if address.is_static_global_package() => {
                    fields.push(address.to_instruction_argument());
                    "CLAIM_PACKAGE_ROYALTIES"
                }

                /* Resource manager */
                (address, FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT)
                    if address.is_static_global_fungible_resource_manager() =>
                {
                    fields.push(address.to_instruction_argument());
                    "MINT_FUNGIBLE"
                }
                (address, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT)
                    if address.is_static_global_non_fungible_resource_manager() =>
                {
                    fields.push(address.to_instruction_argument());
                    "MINT_NON_FUNGIBLE"
                }
                (address, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT)
                    if address.is_static_global_non_fungible_resource_manager() =>
                {
                    fields.push(address.to_instruction_argument());
                    "MINT_RUID_NON_FUNGIBLE"
                }

                /* Validator */
                (address, CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT)
                    if address == &CONSENSUS_MANAGER.into() =>
                {
                    "CREATE_VALIDATOR"
                }

                /* Default */
                _ => {
                    fields.push(address.to_instruction_argument());
                    fields.push(to_manifest_value(method_name)?);
                    "CALL_METHOD"
                }
            };

            if let Value::Tuple { fields: arg_fields } = args {
                fields.extend(arg_fields.clone());
            } else {
                return Err(DecompileError::InvalidArguments);
            }

            let parameters = Value::Tuple { fields };
            (name, parameters)
        }
        InstructionV1::CallRoyaltyMethod {
            address,
            method_name,
            args,
        } => {
            let mut fields = Vec::new();
            let name = match (address, method_name.as_str()) {
                /* Component royalty */
                (address, COMPONENT_ROYALTY_SET_ROYALTY_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "SET_COMPONENT_ROYALTY"
                }
                (address, COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "LOCK_COMPONENT_ROYALTY"
                }
                (address, COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "CLAIM_COMPONENT_ROYALTIES"
                }

                /* Default */
                _ => {
                    fields.push(address.to_instruction_argument());
                    fields.push(to_manifest_value(method_name)?);
                    "CALL_ROYALTY_METHOD"
                }
            };

            if let Value::Tuple { fields: arg_fields } = args {
                fields.extend(arg_fields.clone());
            } else {
                return Err(DecompileError::InvalidArguments);
            }

            let parameters = Value::Tuple { fields };
            (name, parameters)
        }
        InstructionV1::CallMetadataMethod {
            address,
            method_name,
            args,
        } => {
            let mut fields = Vec::new();
            let name = match (address, method_name.as_str()) {
                /* Metadata */
                (address, METADATA_SET_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "SET_METADATA"
                }
                (address, METADATA_REMOVE_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "REMOVE_METADATA"
                }
                (address, METADATA_LOCK_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "LOCK_METADATA"
                }

                /* Default */
                _ => {
                    fields.push(address.to_instruction_argument());
                    fields.push(to_manifest_value(method_name)?);
                    "CALL_METADATA_METHOD"
                }
            };

            if let Value::Tuple { fields: arg_fields } = args {
                fields.extend(arg_fields.clone());
            } else {
                return Err(DecompileError::InvalidArguments);
            }

            let parameters = Value::Tuple { fields };
            (name, parameters)
        }
        InstructionV1::CallAccessRulesMethod {
            address,
            method_name,
            args,
        } => {
            let mut fields = Vec::new();
            let name = match (address, method_name.as_str()) {
                /* Access rules */
                (address, ACCESS_RULES_SET_OWNER_ROLE_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "SET_OWNER_ROLE"
                }
                (address, ACCESS_RULES_LOCK_OWNER_ROLE_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "LOCK_OWNER_ROLE"
                }
                (address, ACCESS_RULES_SET_AND_LOCK_OWNER_ROLE_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "SET_AND_LOCK_OWNER_ROLE"
                }
                (address, ACCESS_RULES_SET_ROLE_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "SET_ROLE"
                }
                (address, ACCESS_RULES_LOCK_ROLE_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "LOCK_ROLE"
                }
                (address, ACCESS_RULES_SET_AND_LOCK_ROLE_IDENT) => {
                    fields.push(address.to_instruction_argument());
                    "SET_AND_LOCK_ROLE"
                }

                /* Default */
                _ => {
                    fields.push(address.to_instruction_argument());
                    fields.push(to_manifest_value(method_name)?);
                    "CALL_ACCESS_RULES_METHOD"
                }
            };

            if let Value::Tuple { fields: arg_fields } = args {
                fields.extend(arg_fields.clone());
            } else {
                return Err(DecompileError::InvalidArguments);
            }

            let parameters = Value::Tuple { fields };
            (name, parameters)
        }
        InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name,
            args,
        } => {
            let mut fields = Vec::new();
            let name = match method_name.as_str() {
                VAULT_RECALL_IDENT => {
                    fields.push(to_manifest_value(vault_id)?);
                    "RECALL_FROM_VAULT"
                }
                VAULT_FREEZE_IDENT => {
                    fields.push(to_manifest_value(vault_id)?);
                    "FREEZE_VAULT"
                }
                VAULT_UNFREEZE_IDENT => {
                    fields.push(to_manifest_value(vault_id)?);
                    "UNFREEZE_VAULT"
                }
                /* Default */
                _ => {
                    fields.push(to_manifest_value(vault_id)?);
                    fields.push(to_manifest_value(method_name)?);
                    "CALL_DIRECT_VAULT_METHOD"
                }
            };

            if let Value::Tuple { fields: arg_fields } = args {
                fields.extend(arg_fields.clone());
            } else {
                return Err(DecompileError::InvalidArguments);
            }

            let parameters = Value::Tuple { fields };
            (name, parameters)
        }

        InstructionV1::DropAllProofs => ("DROP_ALL_PROOFS", to_manifest_value(&())?),
        InstructionV1::AllocateGlobalAddress {
            package_address,
            blueprint_name,
        } => {
            let address_reservation = context.new_address_reservation();
            let named_address = context.new_address();
            (
                "ALLOCATE_GLOBAL_ADDRESS",
                to_manifest_value(&(
                    package_address,
                    blueprint_name,
                    address_reservation,
                    named_address,
                ))?,
            )
        }
    };

    write!(f, "{}", display_name)?;
    if let Value::Tuple { fields } = display_parameters {
        let field_count = fields.len();
        for field in fields {
            write!(f, "\n")?;
            format_manifest_value(f, &field, &context.for_value_display(), true, 0)?;
        }
        if field_count > 0 {
            write!(f, "\n;\n")?;
        } else {
            write!(f, ";\n")?;
        }
    } else {
        panic!(
            "Parameters are not a tuple: name = {:?}, parameters = {:?}",
            display_name, display_parameters
        );
    }

    Ok(())
}
