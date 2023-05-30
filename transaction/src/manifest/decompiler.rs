use crate::data::*;
use crate::errors::*;
use crate::model::*;
use crate::validation::*;
use radix_engine_common::native_addresses::PACKAGE_PACKAGE;
use radix_engine_common::prelude::CONSENSUS_MANAGER;
use radix_engine_interface::address::Bech32Encoder;
use radix_engine_interface::api::node_modules::auth::ACCESS_RULES_UPDATE_ROLE_IDENT;
use radix_engine_interface::api::node_modules::metadata::METADATA_REMOVE_IDENT;
use radix_engine_interface::api::node_modules::metadata::METADATA_SET_IDENT;
use radix_engine_interface::api::node_modules::royalty::{
    COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
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
use radix_engine_interface::blueprints::package::PACKAGE_PUBLISH_WASM_ADVANCED_IDENT;
use radix_engine_interface::blueprints::package::PACKAGE_PUBLISH_WASM_IDENT;
use radix_engine_interface::blueprints::package::{
    PACKAGE_CLAIM_ROYALTY_IDENT, PACKAGE_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT, FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
    FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
    NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT,
    VAULT_RECALL_IDENT,
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
    IdAllocationError(ManifestIdAllocationError),
    FormattingError(fmt::Error),
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

pub struct DecompilationContext<'a> {
    pub bech32_encoder: Option<&'a Bech32Encoder>,
    pub id_allocator: ManifestIdAllocator,
    pub bucket_names: NonIterMap<ManifestBucket, String>,
    pub proof_names: NonIterMap<ManifestProof, String>,
}

impl<'a> DecompilationContext<'a> {
    pub fn new(bech32_encoder: &'a Bech32Encoder) -> Self {
        Self {
            bech32_encoder: Some(bech32_encoder),
            id_allocator: ManifestIdAllocator::new(),
            bucket_names: NonIterMap::<ManifestBucket, String>::new(),
            proof_names: NonIterMap::<ManifestProof, String>::new(),
        }
    }

    pub fn new_with_optional_network(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
        Self {
            bech32_encoder,
            id_allocator: ManifestIdAllocator::new(),
            bucket_names: NonIterMap::<ManifestBucket, String>::new(),
            proof_names: NonIterMap::<ManifestProof, String>::new(),
        }
    }

    pub fn for_value_display(&'a self) -> ManifestDecompilationDisplayContext<'a> {
        ManifestDecompilationDisplayContext::with_bech32_and_names(
            self.bech32_encoder,
            &self.bucket_names,
            &self.proof_names,
        )
        .with_multi_line(4, 4)
    }

    pub fn new_bucket(&mut self) -> Result<ManifestBucket, DecompileError> {
        let bucket = self
            .id_allocator
            .new_bucket_id()
            .map_err(DecompileError::IdAllocationError)?;
        let name = format!("bucket{}", self.bucket_names.len() + 1);
        self.bucket_names.insert(bucket, name.clone());
        Ok(bucket)
    }

    pub fn new_proof(&mut self) -> Result<ManifestProof, DecompileError> {
        let proof = self
            .id_allocator
            .new_proof_id()
            .map_err(DecompileError::IdAllocationError)?;
        let name = format!("proof{}", self.proof_names.len() + 1);
        self.proof_names.insert(proof, name.clone());
        Ok(proof)
    }
}

/// Contract: if the instructions are from a validated notarized transaction, no error
/// should be returned.
pub fn decompile(
    instructions: &[InstructionV1],
    network: &NetworkDefinition,
) -> Result<String, DecompileError> {
    let bech32_encoder = Bech32Encoder::new(network);
    let mut buf = String::new();
    let mut context = DecompilationContext::new(&bech32_encoder);
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
            let bucket = context.new_bucket()?;
            (
                "TAKE_FROM_WORKTOP",
                to_manifest_value(&(resource_address, amount, bucket)),
            )
        }
        InstructionV1::TakeNonFungiblesFromWorktop {
            ids,
            resource_address,
        } => {
            let bucket = context.new_bucket()?;
            (
                "TAKE_NON_FUNGIBLES_FROM_WORKTOP",
                to_manifest_value(&(resource_address, ids, bucket)),
            )
        }
        InstructionV1::TakeAllFromWorktop { resource_address } => {
            let bucket = context.new_bucket()?;
            (
                "TAKE_ALL_FROM_WORKTOP",
                to_manifest_value(&(resource_address, bucket)),
            )
        }
        InstructionV1::ReturnToWorktop { bucket_id } => {
            ("RETURN_TO_WORKTOP", to_manifest_value(&(bucket_id,)))
        }
        InstructionV1::AssertWorktopContains {
            amount,
            resource_address,
        } => (
            "ASSERT_WORKTOP_CONTAINS",
            to_manifest_value(&(resource_address, amount)),
        ),
        InstructionV1::AssertWorktopContainsNonFungibles {
            resource_address,
            ids,
        } => (
            "ASSERT_WORKTOP_CONTAINS_NON_FUNGIBLES",
            to_manifest_value(&(resource_address, ids)),
        ),
        InstructionV1::PopFromAuthZone => {
            let proof = context.new_proof()?;
            ("POP_FROM_AUTH_ZONE", to_manifest_value(&(proof,)))
        }
        InstructionV1::PushToAuthZone { proof_id } => {
            ("PUSH_TO_AUTH_ZONE", to_manifest_value(&(proof_id,)))
        }
        InstructionV1::ClearAuthZone => ("CLEAR_AUTH_ZONE", to_manifest_value(&())),
        InstructionV1::CreateProofFromAuthZone { resource_address } => {
            let proof = context.new_proof()?;
            (
                "CREATE_PROOF_FROM_AUTH_ZONE",
                to_manifest_value(&(resource_address, proof)),
            )
        }
        InstructionV1::CreateProofFromAuthZoneOfAmount {
            resource_address,
            amount,
        } => {
            let proof = context.new_proof()?;

            (
                "CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT",
                to_manifest_value(&(resource_address, amount, proof)),
            )
        }
        InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
            resource_address,
            ids,
        } => {
            let proof = context.new_proof()?;
            (
                "CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES",
                to_manifest_value(&(resource_address, ids, proof)),
            )
        }
        InstructionV1::CreateProofFromAuthZoneOfAll { resource_address } => {
            let proof = context.new_proof()?;
            (
                "CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL",
                to_manifest_value(&(resource_address, proof)),
            )
        }

        InstructionV1::ClearSignatureProofs => ("CLEAR_SIGNATURE_PROOFS", to_manifest_value(&())),

        InstructionV1::CreateProofFromBucket { bucket_id } => {
            let proof = context.new_proof()?;
            (
                "CREATE_PROOF_FROM_BUCKET",
                to_manifest_value(&(bucket_id, proof)),
            )
        }

        InstructionV1::CreateProofFromBucketOfAmount { bucket_id, amount } => {
            let proof = context.new_proof()?;
            (
                "CREATE_PROOF_FROM_BUCKET_OF_AMOUNT",
                to_manifest_value(&(bucket_id, amount, proof)),
            )
        }
        InstructionV1::CreateProofFromBucketOfNonFungibles { bucket_id, ids } => {
            let proof = context.new_proof()?;
            (
                "CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES",
                to_manifest_value(&(bucket_id, ids, proof)),
            )
        }
        InstructionV1::CreateProofFromBucketOfAll { bucket_id } => {
            let proof = context.new_proof()?;
            (
                "CREATE_PROOF_FROM_BUCKET_OF_ALL",
                to_manifest_value(&(bucket_id, proof)),
            )
        }
        InstructionV1::BurnResource { bucket_id } => {
            ("BURN_RESOURCE", to_manifest_value(&(bucket_id,)))
        }
        InstructionV1::CloneProof { proof_id } => {
            let proof_id2 = context.new_proof()?;
            ("CLONE_PROOF", to_manifest_value(&(proof_id, proof_id2)))
        }
        InstructionV1::DropProof { proof_id } => ("DROP_PROOF", to_manifest_value(&(proof_id,))),
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
                (&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_WASM_IDENT) => {
                    "PUBLISH_PACKAGE"
                }
                (&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_WASM_ADVANCED_IDENT) => {
                    "PUBLISH_PACKAGE_ADVANCED"
                }
                (&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_ADVANCED_IDENT) => {
                    "CREATE_ACCOUNT_ADVANCED"
                }
                (&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_IDENT) => "CREATE_ACCOUNT",
                (&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT, IDENTITY_CREATE_ADVANCED_IDENT) => {
                    "CREATE_IDENTITY_ADVANCED"
                }
                (&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT, IDENTITY_CREATE_IDENT) => "CREATE_IDENTITY",
                (
                    &ACCESS_CONTROLLER_PACKAGE,
                    ACCESS_CONTROLLER_BLUEPRINT,
                    ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT,
                ) => "CREATE_ACCESS_CONTROLLER",
                (
                    &RESOURCE_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                ) => "CREATE_FUNGIBLE_RESOURCE",
                (
                    &RESOURCE_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                ) => "CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY",
                (
                    &RESOURCE_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                ) => "CREATE_NON_FUNGIBLE_RESOURCE",
                (
                    &RESOURCE_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                ) => "CREATE_NON_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY",
                _ => {
                    fields.push(to_manifest_value(package_address));
                    fields.push(to_manifest_value(blueprint_name));
                    fields.push(to_manifest_value(function_name));
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
                (address, PACKAGE_SET_ROYALTY_CONFIG_IDENT)
                    if address.as_node_id().is_global_package() =>
                {
                    fields.push(to_manifest_value(address));
                    "SET_PACKAGE_ROYALTY_CONFIG"
                }
                (address, PACKAGE_CLAIM_ROYALTY_IDENT)
                    if address.as_node_id().is_global_package() =>
                {
                    fields.push(to_manifest_value(address));
                    "CLAIM_PACKAGE_ROYALTY"
                }

                /* Resource manager */
                (address, FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT)
                    if address.as_node_id().is_global_fungible_resource_manager() =>
                {
                    fields.push(to_manifest_value(address));
                    "MINT_FUNGIBLE"
                }
                (address, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT)
                    if address
                        .as_node_id()
                        .is_global_non_fungible_resource_manager() =>
                {
                    fields.push(to_manifest_value(address));
                    "MINT_NON_FUNGIBLE"
                }
                (address, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT)
                    if address
                        .as_node_id()
                        .is_global_non_fungible_resource_manager() =>
                {
                    fields.push(to_manifest_value(address));
                    "MINT_UUID_NON_FUNGIBLE"
                }

                /* Validator */
                (address, CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT)
                    if address == &CONSENSUS_MANAGER.into() =>
                {
                    "CREATE_VALIDATOR"
                }

                /* Default */
                _ => {
                    fields.push(to_manifest_value(address));
                    fields.push(to_manifest_value(method_name));
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
                (address, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT) => {
                    fields.push(to_manifest_value(address));
                    "SET_COMPONENT_ROYALTY_CONFIG"
                }
                (address, COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT) => {
                    fields.push(to_manifest_value(address));
                    "CLAIM_COMPONENT_ROYALTY"
                }

                /* Default */
                _ => {
                    fields.push(to_manifest_value(address));
                    fields.push(to_manifest_value(method_name));
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
                    fields.push(to_manifest_value(address));
                    "SET_METADATA"
                }
                (address, METADATA_REMOVE_IDENT) => {
                    fields.push(to_manifest_value(address));
                    "REMOVE_METADATA"
                }

                /* Default */
                _ => {
                    fields.push(to_manifest_value(address));
                    fields.push(to_manifest_value(method_name));
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
                (address, ACCESS_RULES_UPDATE_ROLE_IDENT) => {
                    fields.push(to_manifest_value(address));
                    "UPDATE_ROLE"
                }

                /* Default */
                _ => {
                    fields.push(to_manifest_value(address));
                    fields.push(to_manifest_value(method_name));
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
            vault_id,
            method_name,
            args,
        } => {
            let mut fields = Vec::new();
            let name = match method_name.as_str() {
                VAULT_RECALL_IDENT => {
                    fields.push(to_manifest_value(vault_id));
                    "RECALL_RESOURCE"
                }
                /* Default */
                _ => {
                    fields.push(to_manifest_value(vault_id));
                    fields.push(to_manifest_value(method_name));
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

        InstructionV1::DropAllProofs => ("DROP_ALL_PROOFS", to_manifest_value(&())),
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
