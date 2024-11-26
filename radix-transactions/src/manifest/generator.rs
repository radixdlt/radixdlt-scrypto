use core::iter::*;

use super::ast::Instruction;
use super::ast::InstructionWithSpan;
use super::ast::ValueKindWithSpan;
use super::blob_provider::*;
use crate::data::*;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::manifest::ast;
use crate::manifest::compiler::CompileErrorDiagnosticsStyle;
use crate::manifest::diagnostic_snippets::create_snippet;
use crate::manifest::token::Span;
use crate::model::*;
use crate::validation::*;
use radix_common::address::AddressBech32Decoder;
use radix_common::constants::*;
use radix_common::crypto::Hash;
use radix_common::data::manifest::model::*;
use radix_common::data::manifest::*;
use radix_common::data::scrypto::model::*;
use radix_common::math::{Decimal, PreciseDecimal};
use radix_common::prelude::CONSENSUS_MANAGER;
use radix_common::types::NodeId;
use radix_common::types::NonFungibleGlobalId;
use radix_common::types::PackageAddress;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::*;
use radix_engine_interface::object_modules::role_assignment::*;
use radix_engine_interface::object_modules::royalty::*;
use radix_engine_interface::types::*;
use sbor::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratorErrorKind {
    InvalidAstType {
        expected_value_kind: ast::ValueKind,
        actual: ast::ValueKind,
    },
    InvalidAstValue {
        expected_value_kinds: Vec<ast::ValueKind>,
        actual: ast::Value,
    },
    UnexpectedValueKind {
        expected_value_kind: ast::ValueKind,
        actual_value: ast::Value,
    },
    InvalidPackageAddress(String),
    InvalidResourceAddress(String),
    InvalidDecimal {
        actual: String,
        err: String,
    },
    InvalidPreciseDecimal {
        actual: String,
        err: String,
    },
    InvalidNonFungibleLocalId(String),
    InvalidNonFungibleGlobalId,
    InvalidExpression(String),
    InvalidBlobHash {
        actual: String,
        err: String,
    },
    BlobNotFound(String),
    InvalidBytesHex(String),
    NameResolverError(NameResolverError),
    IdValidationError {
        err: ManifestIdValidationError,
        name: Option<String>,
    },
    InvalidGlobalAddress(String),
    InvalidInternalAddress(String),
    InvalidSubTransactionId(String),
    InstructionNotSupportedInManifestVersion,
    ManifestBuildError(ManifestBuildError),
    HeaderInstructionMustComeFirst,
    IntentCannotBeUsedInValue,
    IntentCannotBeUsedAsValueKind,
    NamedIntentCannotBeUsedInValue,
    NamedIntentCannotBeUsedAsValueKind,
    ArgumentCouldNotBeReadAsExpectedType {
        type_name: String,
        error_message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorError {
    pub error_kind: GeneratorErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameResolverError {
    UndefinedBucket(String),
    UndefinedProof(String),
    UndefinedAddressReservation(String),
    UndefinedNamedAddress(String),
    UndefinedIntent(String),
    NamedAlreadyDefined(String),
}

#[derive(Default)]
pub struct NameResolver {
    named_buckets: IndexMap<String, ManifestBucket>,
    named_proofs: IndexMap<String, ManifestProof>,
    named_address_reservations: IndexMap<String, ManifestAddressReservation>,
    named_addresses: IndexMap<String, ManifestNamedAddress>,
    named_intents: IndexMap<String, ManifestNamedIntent>,
}

impl NameResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_bucket(
        &mut self,
        name: String,
        bucket_id: ManifestBucket,
    ) -> Result<(), NameResolverError> {
        if self.named_buckets.contains_key(&name) {
            Err(NameResolverError::NamedAlreadyDefined(name))
        } else {
            self.named_buckets.insert(name, bucket_id);
            Ok(())
        }
    }

    pub fn insert_proof(
        &mut self,
        name: String,
        proof_id: ManifestProof,
    ) -> Result<(), NameResolverError> {
        if self.named_proofs.contains_key(&name) {
            Err(NameResolverError::NamedAlreadyDefined(name))
        } else {
            self.named_proofs.insert(name, proof_id);
            Ok(())
        }
    }

    pub fn insert_address_reservation(
        &mut self,
        name: String,
        address_reservation_id: ManifestAddressReservation,
    ) -> Result<(), NameResolverError> {
        if self.named_address_reservations.contains_key(&name) {
            Err(NameResolverError::NamedAlreadyDefined(name))
        } else {
            self.named_address_reservations
                .insert(name, address_reservation_id);
            Ok(())
        }
    }

    pub fn insert_named_address(
        &mut self,
        name: String,
        address_id: ManifestNamedAddress,
    ) -> Result<(), NameResolverError> {
        if self.named_addresses.contains_key(&name) {
            Err(NameResolverError::NamedAlreadyDefined(name))
        } else {
            self.named_addresses.insert(name, address_id);
            Ok(())
        }
    }

    pub fn insert_intent(
        &mut self,
        name: String,
        intent_id: ManifestNamedIntent,
    ) -> Result<(), NameResolverError> {
        if self.named_intents.contains_key(&name) {
            Err(NameResolverError::NamedAlreadyDefined(name))
        } else {
            self.named_intents.insert(name, intent_id);
            Ok(())
        }
    }

    pub fn resolve_bucket(&mut self, name: &str) -> Result<ManifestBucket, NameResolverError> {
        match self.named_buckets.get(name).cloned() {
            Some(bucket_id) => Ok(bucket_id),
            None => Err(NameResolverError::UndefinedBucket(name.into())),
        }
    }

    pub fn resolve_proof(&mut self, name: &str) -> Result<ManifestProof, NameResolverError> {
        match self.named_proofs.get(name).cloned() {
            Some(proof_id) => Ok(proof_id),
            None => Err(NameResolverError::UndefinedProof(name.into())),
        }
    }

    pub fn resolve_address_reservation(
        &mut self,
        name: &str,
    ) -> Result<ManifestAddressReservation, NameResolverError> {
        match self.named_address_reservations.get(name).cloned() {
            Some(address_reservation_id) => Ok(address_reservation_id),
            None => Err(NameResolverError::UndefinedAddressReservation(name.into())),
        }
    }

    pub fn resolve_named_address(
        &mut self,
        name: &str,
    ) -> Result<ManifestNamedAddress, NameResolverError> {
        match self.named_addresses.get(name).cloned() {
            Some(address_id) => Ok(address_id),
            None => Err(NameResolverError::UndefinedNamedAddress(name.into())),
        }
    }

    pub fn resolve_named_intent(
        &mut self,
        name: &str,
    ) -> Result<ManifestNamedIntent, NameResolverError> {
        match self.named_intents.get(name).cloned() {
            Some(intent_id) => Ok(intent_id),
            None => Err(NameResolverError::UndefinedIntent(name.into())),
        }
    }

    pub fn resolve_bucket_name(&self, bucket: ManifestBucket) -> Option<String> {
        for (name, id) in self.named_buckets.iter() {
            if id.eq(&bucket) {
                return Some(name.to_string());
            }
        }
        return None;
    }

    pub fn resove_proof_name(&self, proof: ManifestProof) -> Option<String> {
        for (name, id) in self.named_proofs.iter() {
            if id.eq(&proof) {
                return Some(name.to_string());
            }
        }
        return None;
    }

    pub fn resolve_address_reservation_name(
        &self,
        reservation: ManifestAddressReservation,
    ) -> Option<String> {
        for (name, id) in self.named_address_reservations.iter() {
            if id.eq(&reservation) {
                return Some(name.to_string());
            }
        }
        return None;
    }

    pub fn resolve_named_address_name(&self, address: ManifestNamedAddress) -> Option<String> {
        for (name, id) in self.named_addresses.iter() {
            if id.eq(&address) {
                return Some(name.to_string());
            }
        }
        return None;
    }

    pub fn resolve_intent_name(&self, address: ManifestNamedIntent) -> Option<String> {
        for (name, id) in self.named_intents.iter() {
            if id.eq(&address) {
                return Some(name.to_string());
            }
        }
        return None;
    }

    pub fn into_known_names(self) -> KnownManifestObjectNames {
        KnownManifestObjectNames {
            bucket_names: self
                .named_buckets
                .into_iter()
                .map(|(name, value)| (value, name))
                .collect(),
            proof_names: self
                .named_proofs
                .into_iter()
                .map(|(name, value)| (value, name))
                .collect(),
            address_reservation_names: self
                .named_address_reservations
                .into_iter()
                .map(|(name, value)| (value, name))
                .collect(),
            address_names: self
                .named_addresses
                .into_iter()
                .map(|(name, value)| (value, name))
                .collect(),
            intent_names: self
                .named_intents
                .into_iter()
                .map(|(name, value)| (value, name))
                .collect(),
        }
    }
}

pub fn generate_manifest<B, M: BuildableManifest>(
    instructions: &[ast::InstructionWithSpan],
    address_bech32_decoder: &AddressBech32Decoder,
    transaction_bech32_decoder: &TransactionHashBech32Decoder,
    blobs: B,
) -> Result<M, GeneratorError>
where
    B: IsBlobProvider,
{
    let mut id_validator = BasicManifestValidator::new();
    let mut name_resolver = NameResolver::new();

    let mut manifest = M::default();

    let mut instructions_iter = instructions.iter().peekable();

    generate_pseudo_instructions(
        &mut manifest,
        &mut instructions_iter,
        &mut id_validator,
        &mut name_resolver,
        address_bech32_decoder,
        transaction_bech32_decoder,
    )?;

    for instruction in instructions_iter {
        let any_instruction = generate_instruction(
            instruction,
            &mut id_validator,
            &mut name_resolver,
            address_bech32_decoder,
            &blobs,
        )?;
        let valid_instruction = any_instruction.try_into().map_err(|_| GeneratorError {
            span: instruction.span,
            error_kind: GeneratorErrorKind::InstructionNotSupportedInManifestVersion,
        })?;
        manifest.add_instruction(valid_instruction);
    }
    for (hash, blob_content) in blobs.blobs() {
        manifest.add_blob(hash, blob_content);
    }
    manifest.set_names(name_resolver.into_known_names());

    Ok(manifest)
}

fn generate_pseudo_instructions(
    manifest: &mut impl BuildableManifest,
    instructions_iter: &mut Peekable<core::slice::Iter<ast::InstructionWithSpan>>,
    id_validator: &mut BasicManifestValidator,
    name_resolver: &mut NameResolver,
    address_bech32_decoder: &AddressBech32Decoder,
    transaction_bech32_decoder: &TransactionHashBech32Decoder,
) -> Result<(), GeneratorError> {
    // First handle the USE_PREALLOCATED_ADDRESS pseudo-instructions
    loop {
        let Some(InstructionWithSpan {
            instruction: Instruction::UsePreallocatedAddress { .. },
            ..
        }) = instructions_iter.peek()
        else {
            break;
        };
        let Some(InstructionWithSpan {
            instruction:
                Instruction::UsePreallocatedAddress {
                    package_address,
                    blueprint_name,
                    address_reservation,
                    preallocated_address,
                },
            span,
        }) = instructions_iter.next()
        else {
            unreachable!("Just peeked and verified");
        };
        declare_address_reservation(
            address_reservation,
            name_resolver,
            id_validator.new_address_reservation(),
        )?;
        manifest
            .add_preallocated_address(PreAllocatedAddress {
                blueprint_id: BlueprintId {
                    package_address: generate_package_address(
                        package_address,
                        address_bech32_decoder,
                    )?,
                    blueprint_name: generate_string(blueprint_name)?,
                },
                address: generate_global_address(preallocated_address, address_bech32_decoder)?,
            })
            .map_err(|err| GeneratorError {
                span: *span,
                error_kind: GeneratorErrorKind::ManifestBuildError(err),
            })?;
    }

    // Next, handle the USE_CHILD pseudo-instructions
    loop {
        let Some(InstructionWithSpan {
            instruction: Instruction::UseChild { .. },
            ..
        }) = instructions_iter.peek()
        else {
            break;
        };
        let Some(InstructionWithSpan {
            instruction:
                Instruction::UseChild {
                    named_intent,
                    subintent_hash,
                },
            span,
        }) = instructions_iter.next()
        else {
            unreachable!("Just peeked and verified");
        };
        declare_named_intent(named_intent, name_resolver, id_validator.new_intent())?;
        manifest
            .add_child_subintent(generate_subintent_hash(
                transaction_bech32_decoder,
                subintent_hash,
            )?)
            .map_err(|err| GeneratorError {
                span: *span,
                error_kind: GeneratorErrorKind::ManifestBuildError(err),
            })?;
    }

    Ok(())
}

macro_rules! get_span {
    ($outer:expr, $inner_vec:expr) => {
        if $inner_vec.is_empty() {
            $outer.span
        } else {
            let start = $inner_vec.get(0).unwrap().span.start;
            let end = $inner_vec.get($inner_vec.len() - 1).unwrap().span.end;
            Span { start, end }
        }
    };
}

fn generate_id_validation_error(
    resolver: &NameResolver,
    err: ManifestIdValidationError,
    span: Span,
) -> GeneratorError {
    let name = match err {
        ManifestIdValidationError::BucketLocked(bucket) => resolver.resolve_bucket_name(bucket),
        ManifestIdValidationError::BucketNotFound(bucket) => resolver.resolve_bucket_name(bucket),
        ManifestIdValidationError::ProofNotFound(proof) => resolver.resove_proof_name(proof),
        ManifestIdValidationError::AddressNotFound(address) => {
            resolver.resolve_named_address_name(address)
        }
        ManifestIdValidationError::AddressReservationNotFound(reservation) => {
            resolver.resolve_address_reservation_name(reservation)
        }
        ManifestIdValidationError::IntentNotFound(intent) => resolver.resolve_intent_name(intent),
    };

    GeneratorError {
        error_kind: GeneratorErrorKind::IdValidationError { err, name },
        span,
    }
}

pub fn generate_instruction<B>(
    instruction: &ast::InstructionWithSpan,
    id_validator: &mut BasicManifestValidator,
    resolver: &mut NameResolver,
    address_bech32_decoder: &AddressBech32Decoder,
    blobs: &B,
) -> Result<AnyInstruction, GeneratorError>
where
    B: IsBlobProvider,
{
    Ok(match &instruction.instruction {
        // ==============
        // Pseudo-instructions
        // ==============
        ast::Instruction::UsePreallocatedAddress { .. } | ast::Instruction::UseChild { .. } => {
            return Err(GeneratorError {
                span: instruction.span,
                error_kind: GeneratorErrorKind::HeaderInstructionMustComeFirst,
            })
        }
        // ==============
        // Standard instructions (in canonical order)
        // ==============

        // Bucket Lifecycle
        ast::Instruction::TakeFromWorktop {
            resource_address,
            amount,
            new_bucket,
        } => {
            let bucket_id = id_validator.new_bucket();
            declare_bucket(new_bucket, resolver, bucket_id)?;

            TakeFromWorktop {
                amount: generate_decimal(amount)?,
                resource_address: generate_resource_address(
                    resource_address,
                    address_bech32_decoder,
                )?,
            }
            .into()
        }
        ast::Instruction::TakeNonFungiblesFromWorktop {
            resource_address,
            ids,
            new_bucket,
        } => {
            let bucket_id = id_validator.new_bucket();
            declare_bucket(new_bucket, resolver, bucket_id)?;

            TakeNonFungiblesFromWorktop {
                ids: generate_non_fungible_local_ids(ids)?,
                resource_address: generate_resource_address(
                    resource_address,
                    address_bech32_decoder,
                )?,
            }
            .into()
        }
        ast::Instruction::TakeAllFromWorktop {
            resource_address,
            new_bucket,
        } => {
            let bucket_id = id_validator.new_bucket();
            declare_bucket(new_bucket, resolver, bucket_id)?;

            TakeAllFromWorktop {
                resource_address: generate_resource_address(
                    resource_address,
                    address_bech32_decoder,
                )?,
            }
            .into()
        }
        ast::Instruction::ReturnToWorktop { bucket } => {
            let (bucket_id, span) = generate_bucket(bucket, resolver)?;
            id_validator
                .drop_bucket(&bucket_id)
                .map_err(|err| generate_id_validation_error(resolver, err, span))?;
            ReturnToWorktop { bucket_id }.into()
        }
        ast::Instruction::BurnResource { bucket } => {
            let (bucket_id, span) = generate_bucket(bucket, resolver)?;
            id_validator
                .drop_bucket(&bucket_id)
                .map_err(|err| generate_id_validation_error(resolver, err, span))?;
            BurnResource { bucket_id }.into()
        }

        // Resource Assertions
        ast::Instruction::AssertWorktopContains {
            resource_address,
            amount,
        } => AssertWorktopContains {
            amount: generate_decimal(amount)?,
            resource_address: generate_resource_address(resource_address, address_bech32_decoder)?,
        }
        .into(),
        ast::Instruction::AssertWorktopContainsNonFungibles {
            resource_address,
            ids,
        } => AssertWorktopContainsNonFungibles {
            resource_address: generate_resource_address(resource_address, address_bech32_decoder)?,
            ids: generate_non_fungible_local_ids(ids)?,
        }
        .into(),
        ast::Instruction::AssertWorktopContainsAny { resource_address } => {
            AssertWorktopContainsAny {
                resource_address: generate_resource_address(
                    resource_address,
                    address_bech32_decoder,
                )?,
            }
            .into()
        }
        ast::Instruction::AssertWorktopIsEmpty {} => AssertWorktopResourcesOnly {
            constraints: Default::default(),
        }
        .into(),
        ast::Instruction::AssertWorktopResourcesOnly { constraints } => {
            AssertWorktopResourcesOnly {
                constraints: generate_typed_value(
                    constraints,
                    resolver,
                    address_bech32_decoder,
                    blobs,
                )?,
            }
            .into()
        }
        ast::Instruction::AssertWorktopResourcesInclude { constraints } => {
            AssertWorktopResourcesInclude {
                constraints: generate_typed_value(
                    constraints,
                    resolver,
                    address_bech32_decoder,
                    blobs,
                )?,
            }
            .into()
        }
        ast::Instruction::AssertNextCallReturnsOnly { constraints } => AssertNextCallReturnsOnly {
            constraints: generate_typed_value(
                constraints,
                resolver,
                address_bech32_decoder,
                blobs,
            )?,
        }
        .into(),
        ast::Instruction::AssertNextCallReturnsInclude { constraints } => {
            AssertNextCallReturnsInclude {
                constraints: generate_typed_value(
                    constraints,
                    resolver,
                    address_bech32_decoder,
                    blobs,
                )?,
            }
            .into()
        }
        ast::Instruction::AssertBucketContents { bucket, constraint } => {
            let (bucket_id, span) = generate_bucket(bucket, resolver)?;
            id_validator
                .check_bucket(&bucket_id)
                .map_err(|err| generate_id_validation_error(resolver, err, span))?;
            AssertBucketContents {
                bucket_id,
                constraint: generate_typed_value(
                    constraint,
                    resolver,
                    address_bech32_decoder,
                    blobs,
                )?,
            }
            .into()
        }

        // Proof Lifecycle
        ast::Instruction::CreateProofFromBucketOfAmount {
            bucket,
            amount,
            new_proof,
        } => {
            let (bucket_id, span) = generate_bucket(bucket, resolver)?;
            let amount = generate_decimal(amount)?;
            let proof_id = id_validator
                .new_proof(ProofKind::BucketProof(bucket_id.clone()))
                .map_err(|err| generate_id_validation_error(resolver, err, span))?;
            declare_proof(new_proof, resolver, proof_id)?;

            CreateProofFromBucketOfAmount { bucket_id, amount }.into()
        }
        ast::Instruction::CreateProofFromBucketOfNonFungibles {
            bucket,
            ids,
            new_proof,
        } => {
            let (bucket_id, span) = generate_bucket(bucket, resolver)?;
            let ids = generate_non_fungible_local_ids(ids)?;
            let proof_id = id_validator
                .new_proof(ProofKind::BucketProof(bucket_id.clone()))
                .map_err(|err| generate_id_validation_error(resolver, err, span))?;
            declare_proof(new_proof, resolver, proof_id)?;

            CreateProofFromBucketOfNonFungibles { bucket_id, ids }.into()
        }
        ast::Instruction::CreateProofFromBucketOfAll { bucket, new_proof } => {
            let (bucket_id, span) = generate_bucket(bucket, resolver)?;
            let proof_id = id_validator
                .new_proof(ProofKind::BucketProof(bucket_id.clone()))
                .map_err(|err| generate_id_validation_error(resolver, err, span))?;
            declare_proof(new_proof, resolver, proof_id)?;

            CreateProofFromBucketOfAll { bucket_id }.into()
        }
        ast::Instruction::CreateProofFromAuthZoneOfAmount {
            resource_address,
            amount,
            new_proof,
        } => {
            let resource_address =
                generate_resource_address(resource_address, address_bech32_decoder)?;
            let amount = generate_decimal(amount)?;
            let proof_id = id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(|err| generate_id_validation_error(resolver, err, instruction.span))?;
            declare_proof(new_proof, resolver, proof_id)?;

            CreateProofFromAuthZoneOfAmount {
                amount,
                resource_address,
            }
            .into()
        }
        ast::Instruction::CreateProofFromAuthZoneOfNonFungibles {
            resource_address,
            ids,
            new_proof,
        } => {
            let resource_address =
                generate_resource_address(resource_address, address_bech32_decoder)?;
            let ids = generate_non_fungible_local_ids(ids)?;
            let proof_id = id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(|err| generate_id_validation_error(resolver, err, instruction.span))?;
            declare_proof(new_proof, resolver, proof_id)?;

            CreateProofFromAuthZoneOfNonFungibles {
                ids,
                resource_address,
            }
            .into()
        }
        ast::Instruction::CreateProofFromAuthZoneOfAll {
            resource_address,
            new_proof,
        } => {
            let resource_address =
                generate_resource_address(resource_address, address_bech32_decoder)?;
            let proof_id = id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(|err| generate_id_validation_error(resolver, err, instruction.span))?;
            declare_proof(new_proof, resolver, proof_id)?;

            CreateProofFromAuthZoneOfAll { resource_address }.into()
        }
        ast::Instruction::CloneProof { proof, new_proof } => {
            let (proof_id, span) = generate_proof(proof, resolver)?;
            let proof_id2 = id_validator
                .clone_proof(&proof_id)
                .map_err(|err| generate_id_validation_error(resolver, err, span))?;
            declare_proof(new_proof, resolver, proof_id2)?;

            CloneProof { proof_id }.into()
        }
        ast::Instruction::DropProof { proof } => {
            let (proof_id, span) = generate_proof(proof, resolver)?;
            id_validator
                .drop_proof(&proof_id)
                .map_err(|err| generate_id_validation_error(resolver, err, span))?;
            DropProof { proof_id }.into()
        }
        ast::Instruction::PushToAuthZone { proof } => {
            let (proof_id, span) = generate_proof(proof, resolver)?;
            id_validator
                .drop_proof(&proof_id)
                .map_err(|err| generate_id_validation_error(resolver, err, span))?;
            PushToAuthZone { proof_id }.into()
        }
        ast::Instruction::PopFromAuthZone { new_proof } => {
            let proof_id = id_validator
                .new_proof(ProofKind::AuthZoneProof)
                .map_err(|err| generate_id_validation_error(resolver, err, instruction.span))?;
            declare_proof(new_proof, resolver, proof_id)?;

            PopFromAuthZone.into()
        }
        ast::Instruction::DropAuthZoneProofs => DropAuthZoneProofs.into(),
        ast::Instruction::DropAuthZoneRegularProofs => DropAuthZoneRegularProofs.into(),
        ast::Instruction::DropAuthZoneSignatureProofs => DropAuthZoneSignatureProofs.into(),
        ast::Instruction::DropNamedProofs => {
            id_validator
                .drop_all_named_proofs()
                .map_err(|err| generate_id_validation_error(resolver, err, instruction.span))?;
            DropNamedProofs.into()
        }

        ast::Instruction::DropAllProofs => {
            id_validator
                .drop_all_named_proofs()
                .map_err(|err| generate_id_validation_error(resolver, err, instruction.span))?;
            DropAllProofs.into()
        }

        // Invocation
        ast::Instruction::CallFunction {
            package_address,
            blueprint_name,
            function_name,
            args,
        } => {
            let package_address = generate_dynamic_package_address(
                &package_address,
                address_bech32_decoder,
                resolver,
            )?;
            let blueprint_name = generate_string(blueprint_name)?;
            let function_name = generate_string(function_name)?;
            let args_inner = generate_args(args, resolver, address_bech32_decoder, blobs)?;

            id_validator.process_call_data(&args_inner).map_err(|err| {
                generate_id_validation_error(resolver, err, get_span!(instruction, args))
            })?;

            CallFunction {
                package_address,
                blueprint_name,
                function_name,
                args: args_inner,
            }
            .into()
        }
        ast::Instruction::CallMethod {
            address,
            method_name,
            args,
        } => {
            let address =
                generate_dynamic_global_address(address, address_bech32_decoder, resolver)?;
            let method_name = generate_string(method_name)?;
            let args_inner = generate_args(args, resolver, address_bech32_decoder, blobs)?;
            id_validator.process_call_data(&args_inner).map_err(|err| {
                generate_id_validation_error(resolver, err, get_span!(instruction, args))
            })?;
            CallMethod {
                address,
                method_name,
                args: args_inner,
            }
            .into()
        }
        ast::Instruction::CallRoyaltyMethod {
            address,
            method_name,
            args,
        } => {
            let address =
                generate_dynamic_global_address(address, address_bech32_decoder, resolver)?;
            let method_name = generate_string(method_name)?;
            let args_inner = generate_args(args, resolver, address_bech32_decoder, blobs)?;
            id_validator.process_call_data(&args_inner).map_err(|err| {
                generate_id_validation_error(resolver, err, get_span!(instruction, args))
            })?;
            CallRoyaltyMethod {
                address,
                method_name,
                args: args_inner,
            }
            .into()
        }
        ast::Instruction::CallMetadataMethod {
            address,
            method_name,
            args,
        } => {
            let address =
                generate_dynamic_global_address(address, address_bech32_decoder, resolver)?;
            let method_name = generate_string(method_name)?;
            let args_inner = generate_args(args, resolver, address_bech32_decoder, blobs)?;
            id_validator.process_call_data(&args_inner).map_err(|err| {
                generate_id_validation_error(resolver, err, get_span!(instruction, args))
            })?;
            CallMetadataMethod {
                address,
                method_name,
                args: args_inner,
            }
            .into()
        }
        ast::Instruction::CallRoleAssignmentMethod {
            address,
            method_name,
            args,
        } => {
            let address =
                generate_dynamic_global_address(address, address_bech32_decoder, resolver)?;
            let method_name = generate_string(method_name)?;
            let args_inner = generate_args(args, resolver, address_bech32_decoder, blobs)?;
            id_validator.process_call_data(&args_inner).map_err(|err| {
                generate_id_validation_error(resolver, err, get_span!(instruction, args))
            })?;
            CallRoleAssignmentMethod {
                address,
                method_name,
                args: args_inner,
            }
            .into()
        }
        ast::Instruction::CallDirectVaultMethod {
            address,
            method_name,
            args,
        } => {
            let address = generate_internal_address(address, address_bech32_decoder)?;
            let method_name = generate_string(method_name)?;
            let args_inner = generate_args(args, resolver, address_bech32_decoder, blobs)?;
            id_validator.process_call_data(&args_inner).map_err(|err| {
                generate_id_validation_error(resolver, err, get_span!(instruction, args))
            })?;
            CallDirectVaultMethod {
                address,
                method_name,
                args: args_inner,
            }
            .into()
        }

        // Address Allocation
        ast::Instruction::AllocateGlobalAddress {
            package_address,
            blueprint_name,
            address_reservation,
            named_address,
        } => {
            let address_reservation_id = id_validator.new_address_reservation();
            declare_address_reservation(address_reservation, resolver, address_reservation_id)?;

            let address_id = id_validator.new_named_address();
            declare_named_address(named_address, resolver, address_id)?;

            AllocateGlobalAddress {
                package_address: generate_package_address(package_address, address_bech32_decoder)?,
                blueprint_name: generate_string(blueprint_name)?,
            }
            .into()
        }

        // Interaction with other intents
        ast::Instruction::YieldToParent { args } => YieldToParent {
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::YieldToChild { child, args } => YieldToChild {
            child_index: generate_named_intent(child, resolver)?.into(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::VerifyParent { access_rule } => VerifyParent {
            access_rule: generate_typed_value(
                access_rule,
                resolver,
                address_bech32_decoder,
                blobs,
            )?,
        }
        .into(),

        // ==============
        // Call direct vault method aliases
        // ==============
        ast::Instruction::RecallFromVault { vault_id, args } => CallDirectVaultMethod {
            address: generate_internal_address(vault_id, address_bech32_decoder)?,
            method_name: VAULT_RECALL_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::FreezeVault { vault_id, args } => CallDirectVaultMethod {
            address: generate_internal_address(vault_id, address_bech32_decoder)?,
            method_name: VAULT_FREEZE_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::UnfreezeVault { vault_id, args } => CallDirectVaultMethod {
            address: generate_internal_address(vault_id, address_bech32_decoder)?,
            method_name: VAULT_UNFREEZE_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::RecallNonFungiblesFromVault { vault_id, args } => CallDirectVaultMethod {
            address: generate_internal_address(vault_id, address_bech32_decoder)?,
            method_name: NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),

        // ==============
        // Call function aliases
        // ==============
        ast::Instruction::PublishPackage { args } => CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::PublishPackageAdvanced { args } => CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::CreateFungibleResource { args } => CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::CreateFungibleResourceWithInitialSupply { args } => CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::CreateNonFungibleResource { args } => CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::CreateNonFungibleResourceWithInitialSupply { args } => CallFunction {
            package_address: RESOURCE_PACKAGE.into(),
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                .to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::CreateAccessController { args } => CallFunction {
            package_address: ACCESS_CONTROLLER_PACKAGE.into(),
            blueprint_name: ACCESS_CONTROLLER_BLUEPRINT.to_string(),
            function_name: ACCESS_CONTROLLER_CREATE_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::CreateIdentity { args } => CallFunction {
            package_address: IDENTITY_PACKAGE.into(),
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::CreateIdentityAdvanced { args } => CallFunction {
            package_address: IDENTITY_PACKAGE.into(),
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::CreateAccount { args } => CallFunction {
            package_address: ACCOUNT_PACKAGE.into(),
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::CreateAccountAdvanced { args } => CallFunction {
            package_address: ACCOUNT_PACKAGE.into(),
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),

        // ==============
        // Call non-main-method aliases
        // ==============
        ast::Instruction::SetMetadata { address, args } => CallMetadataMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: METADATA_SET_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::RemoveMetadata { address, args } => CallMetadataMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: METADATA_REMOVE_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::LockMetadata { address, args } => CallMetadataMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: METADATA_LOCK_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::SetComponentRoyalty { address, args } => CallRoyaltyMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: COMPONENT_ROYALTY_SET_ROYALTY_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::LockComponentRoyalty { address, args } => CallRoyaltyMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::ClaimComponentRoyalties { address, args } => CallRoyaltyMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::SetOwnerRole { address, args } => CallRoleAssignmentMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: ROLE_ASSIGNMENT_SET_OWNER_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::LockOwnerRole { address, args } => CallRoleAssignmentMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: ROLE_ASSIGNMENT_LOCK_OWNER_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::SetRole { address, args } => CallRoleAssignmentMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: ROLE_ASSIGNMENT_SET_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),

        // ==============
        // Call main-method aliases
        // ==============
        ast::Instruction::MintFungible { address, args } => CallMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::MintNonFungible { address, args } => CallMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::MintRuidNonFungible { address, args } => CallMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::ClaimPackageRoyalties { address, args } => CallMethod {
            address: generate_dynamic_global_address(address, address_bech32_decoder, resolver)?,
            method_name: PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
        ast::Instruction::CreateValidator { args } => CallMethod {
            address: CONSENSUS_MANAGER.into(),
            method_name: CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            args: generate_args(args, resolver, address_bech32_decoder, blobs)?,
        }
        .into(),
    })
}

#[macro_export]
macro_rules! invalid_type {
    ( $span:expr, $v:expr, $($exp:expr),+ ) => {
        Err(GeneratorError {
            error_kind: GeneratorErrorKind::InvalidAstValue {
                expected_value_kinds: vec!($($exp),+),
                actual: $v.clone(),
            },
            span: $span,
        })
    };
}

fn generate_args<B>(
    values: &Vec<ast::ValueWithSpan>,
    resolver: &mut NameResolver,
    address_bech32_decoder: &AddressBech32Decoder,
    blobs: &B,
) -> Result<ManifestValue, GeneratorError>
where
    B: IsBlobProvider,
{
    let mut fields = Vec::new();
    for v in values {
        fields.push(generate_value(
            &v,
            None,
            resolver,
            address_bech32_decoder,
            blobs,
        )?)
    }

    Ok(ManifestValue::Tuple { fields })
}

fn generate_string(value: &ast::ValueWithSpan) -> Result<String, GeneratorError> {
    match &value.value {
        ast::Value::String(s) => Ok(s.into()),
        v => invalid_type!(value.span, v, ast::ValueKind::String),
    }
}

fn generate_decimal(value: &ast::ValueWithSpan) -> Result<Decimal, GeneratorError> {
    match &value.value {
        ast::Value::Decimal(inner) => match &inner.value {
            ast::Value::String(s) => Decimal::from_str(&s).map_err(|err| GeneratorError {
                error_kind: GeneratorErrorKind::InvalidDecimal {
                    actual: s.to_string(),
                    err: format!("{:?}", err),
                },
                span: inner.span,
            }),
            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::Decimal),
    }
}

fn generate_precise_decimal(value: &ast::ValueWithSpan) -> Result<PreciseDecimal, GeneratorError> {
    match &value.value {
        ast::Value::PreciseDecimal(inner) => match &inner.value {
            ast::Value::String(s) => PreciseDecimal::from_str(s).map_err(|err| GeneratorError {
                error_kind: GeneratorErrorKind::InvalidPreciseDecimal {
                    actual: s.to_string(),
                    err: format!("{:?}", err),
                },
                span: inner.span,
            }),

            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::Decimal),
    }
}

fn generate_global_address(
    value: &ast::ValueWithSpan,
    address_bech32_decoder: &AddressBech32Decoder,
) -> Result<GlobalAddress, GeneratorError> {
    match &value.value {
        ast::Value::Address(inner) => match &inner.value {
            ast::Value::String(s) => {
                // TODO: Consider more precise message by interpreting AddressBech32DecodeError
                // (applies everywhere where validate_and_decode() is used)
                if let Ok((_, full_data)) = address_bech32_decoder.validate_and_decode(s) {
                    if let Ok(address) = GlobalAddress::try_from(full_data.as_ref()) {
                        return Ok(address);
                    }
                }
                return Err(GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidGlobalAddress(s.into()),
                    span: inner.span,
                });
            }
            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::Address),
    }
}

fn generate_package_address(
    value: &ast::ValueWithSpan,
    address_bech32_decoder: &AddressBech32Decoder,
) -> Result<PackageAddress, GeneratorError> {
    match &value.value {
        ast::Value::Address(inner) => match &inner.value {
            ast::Value::String(s) => {
                // TODO: Consider more precise message by interpreting AddressBech32DecodeError
                // (applies everywhere where validate_and_decode() is used)
                if let Ok((_, full_data)) = address_bech32_decoder.validate_and_decode(s) {
                    if let Ok(address) = PackageAddress::try_from(full_data.as_ref()) {
                        return Ok(address);
                    }
                }
                return Err(GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidPackageAddress(s.into()),
                    span: inner.span,
                });
            }
            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::Address),
    }
}

fn generate_resource_address(
    value: &ast::ValueWithSpan,
    address_bech32_decoder: &AddressBech32Decoder,
) -> Result<ResourceAddress, GeneratorError> {
    match &value.value {
        ast::Value::Address(inner) => match &inner.value {
            ast::Value::String(s) => {
                if let Ok((_, full_data)) = address_bech32_decoder.validate_and_decode(s) {
                    if let Ok(address) = ResourceAddress::try_from(full_data.as_ref()) {
                        return Ok(address);
                    }
                }
                return Err(GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidResourceAddress(s.into()),
                    span: inner.span,
                });
            }
            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::Address),
    }
}

fn generate_dynamic_global_address(
    value: &ast::ValueWithSpan,
    address_bech32_decoder: &AddressBech32Decoder,
    resolver: &mut NameResolver,
) -> Result<ManifestGlobalAddress, GeneratorError> {
    match &value.value {
        ast::Value::Address(inner) => match &inner.value {
            ast::Value::String(s) => {
                if let Ok((_, full_data)) = address_bech32_decoder.validate_and_decode(s) {
                    if let Ok(address) = GlobalAddress::try_from(full_data.as_ref()) {
                        return Ok(ManifestGlobalAddress::Static(address));
                    }
                }
                return Err(GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidGlobalAddress(s.into()),
                    span: inner.span,
                });
            }
            v => return invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        ast::Value::NamedAddress(inner) => {
            match &inner.value {
                ast::Value::U32(n) => Ok(ManifestGlobalAddress::Named(ManifestNamedAddress(*n))),
                ast::Value::String(s) => resolver
                    .resolve_named_address(&s)
                    .map(Into::into)
                    .map_err(|err| GeneratorError {
                        error_kind: GeneratorErrorKind::NameResolverError(err),
                        span: inner.span,
                    }),
                v => invalid_type!(value.span, v, ast::ValueKind::U32, ast::ValueKind::String),
            }
        }
        v => invalid_type!(
            value.span,
            v,
            ast::ValueKind::Address,
            ast::ValueKind::NamedAddress
        ),
    }
}

fn generate_internal_address(
    value: &ast::ValueWithSpan,
    address_bech32_decoder: &AddressBech32Decoder,
) -> Result<InternalAddress, GeneratorError> {
    match &value.value {
        ast::Value::Address(inner) => match &inner.value {
            ast::Value::String(s) => {
                if let Ok((_, full_data)) = address_bech32_decoder.validate_and_decode(s) {
                    if let Ok(address) = InternalAddress::try_from(full_data.as_ref()) {
                        return Ok(address);
                    }
                }
                return Err(GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidInternalAddress(s.into()),
                    span: inner.span,
                });
            }
            v => return invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::Address),
    }
}

fn generate_dynamic_package_address(
    value: &ast::ValueWithSpan,
    address_bech32_decoder: &AddressBech32Decoder,
    resolver: &mut NameResolver,
) -> Result<ManifestPackageAddress, GeneratorError> {
    match &value.value {
        ast::Value::Address(inner) => match &inner.value {
            ast::Value::String(s) => {
                if let Ok((_, full_data)) = address_bech32_decoder.validate_and_decode(s) {
                    if let Ok(address) = PackageAddress::try_from(full_data.as_ref()) {
                        return Ok(ManifestPackageAddress::Static(address));
                    }
                }
                return Err(GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidPackageAddress(s.into()),
                    span: inner.span,
                });
            }
            v => return invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        ast::Value::NamedAddress(inner) => {
            match &inner.value {
                ast::Value::U32(n) => Ok(ManifestPackageAddress::Named(ManifestNamedAddress(*n))),
                ast::Value::String(s) => resolver
                    .resolve_named_address(&s)
                    .map(Into::into)
                    .map_err(|err| GeneratorError {
                        error_kind: GeneratorErrorKind::NameResolverError(err),
                        span: inner.span,
                    }),
                v => invalid_type!(value.span, v, ast::ValueKind::U32, ast::ValueKind::String),
            }
        }
        v => invalid_type!(
            value.span,
            v,
            ast::ValueKind::Address,
            ast::ValueKind::NamedAddress
        ),
    }
}

fn declare_bucket(
    value: &ast::ValueWithSpan,
    resolver: &mut NameResolver,
    bucket_id: ManifestBucket,
) -> Result<(), GeneratorError> {
    match &value.value {
        ast::Value::Bucket(inner) => match &inner.value {
            ast::Value::String(name) => resolver
                .insert_bucket(name.to_string(), bucket_id)
                .map_err(|err| GeneratorError {
                    error_kind: GeneratorErrorKind::NameResolverError(err),
                    span: inner.span,
                }),
            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::Bucket),
    }
}

fn generate_bucket(
    value: &ast::ValueWithSpan,
    resolver: &mut NameResolver,
) -> Result<(ManifestBucket, Span), GeneratorError> {
    match &value.value {
        ast::Value::Bucket(inner) => {
            let bucket = match &inner.value {
                ast::Value::U32(n) => Ok(ManifestBucket(*n)),
                ast::Value::String(s) => {
                    resolver.resolve_bucket(&s).map_err(|err| GeneratorError {
                        error_kind: GeneratorErrorKind::NameResolverError(err),
                        span: inner.span,
                    })
                }
                v => invalid_type!(inner.span, v, ast::ValueKind::U32, ast::ValueKind::String),
            }?;
            Ok((bucket, inner.span))
        }
        v => invalid_type!(value.span, v, ast::ValueKind::Bucket),
    }
}

fn declare_proof(
    value: &ast::ValueWithSpan,
    resolver: &mut NameResolver,
    proof_id: ManifestProof,
) -> Result<(), GeneratorError> {
    match &value.value {
        ast::Value::Proof(inner) => {
            match &inner.value {
                ast::Value::String(name) => resolver
                    .insert_proof(name.to_string(), proof_id)
                    .map_err(|err| GeneratorError {
                        error_kind: GeneratorErrorKind::NameResolverError(err),
                        span: inner.span,
                    }),
                v => invalid_type!(inner.span, v, ast::ValueKind::String),
            }
        }
        v => invalid_type!(value.span, v, ast::ValueKind::Proof),
    }
}

fn declare_address_reservation(
    value: &ast::ValueWithSpan,
    resolver: &mut NameResolver,
    address_reservation_id: ManifestAddressReservation,
) -> Result<(), GeneratorError> {
    match &value.value {
        ast::Value::AddressReservation(inner) => match &inner.value {
            ast::Value::String(name) => resolver
                .insert_address_reservation(name.to_string(), address_reservation_id)
                .map_err(|err| GeneratorError {
                    error_kind: GeneratorErrorKind::NameResolverError(err),
                    span: inner.span,
                }),
            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::AddressReservation),
    }
}

fn declare_named_address(
    value: &ast::ValueWithSpan,
    resolver: &mut NameResolver,
    address_id: ManifestNamedAddress,
) -> Result<(), GeneratorError> {
    match &value.value {
        ast::Value::NamedAddress(inner) => match &inner.value {
            ast::Value::String(name) => resolver
                .insert_named_address(name.to_string(), address_id)
                .map_err(|err| GeneratorError {
                    error_kind: GeneratorErrorKind::NameResolverError(err),
                    span: inner.span,
                }),
            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::NamedAddress),
    }
}

fn declare_named_intent(
    value: &ast::ValueWithSpan,
    resolver: &mut NameResolver,
    intent_id: ManifestNamedIntent,
) -> Result<(), GeneratorError> {
    match &value.value {
        ast::Value::NamedIntent(inner) => match &inner.value {
            ast::Value::String(name) => resolver
                .insert_intent(name.to_string(), intent_id)
                .map_err(|err| GeneratorError {
                    error_kind: GeneratorErrorKind::NameResolverError(err),
                    span: inner.span,
                }),
            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::NamedIntent),
    }
}

fn generate_named_intent(
    value: &ast::ValueWithSpan,
    resolver: &mut NameResolver,
) -> Result<ManifestNamedIntent, GeneratorError> {
    match &value.value {
        ast::Value::NamedIntent(inner) => {
            let out = match &inner.value {
                // Don't support U32 for new types like this
                ast::Value::String(s) => {
                    resolver
                        .resolve_named_intent(&s)
                        .map_err(|err| GeneratorError {
                            error_kind: GeneratorErrorKind::NameResolverError(err),
                            span: inner.span,
                        })
                }
                v => invalid_type!(inner.span, v, ast::ValueKind::String),
            }?;
            Ok(out)
        }
        v => invalid_type!(value.span, v, ast::ValueKind::NamedIntent),
    }
}

fn generate_proof(
    value: &ast::ValueWithSpan,
    resolver: &mut NameResolver,
) -> Result<(ManifestProof, Span), GeneratorError> {
    match &value.value {
        ast::Value::Proof(inner) => {
            let proof = match &inner.value {
                ast::Value::U32(n) => Ok(ManifestProof(*n)),
                ast::Value::String(s) => resolver.resolve_proof(&s).map_err(|err| GeneratorError {
                    error_kind: GeneratorErrorKind::NameResolverError(err),
                    span: inner.span,
                }),
                v => invalid_type!(inner.span, v, ast::ValueKind::U32, ast::ValueKind::String),
            }?;
            Ok((proof, inner.span))
        }
        v => invalid_type!(value.span, v, ast::ValueKind::Proof),
    }
}

fn generate_address_reservation(
    value: &ast::ValueWithSpan,
    resolver: &mut NameResolver,
) -> Result<ManifestAddressReservation, GeneratorError> {
    match &value.value {
        ast::Value::AddressReservation(inner) => match &inner.value {
            ast::Value::U32(n) => Ok(ManifestAddressReservation(*n)),
            ast::Value::String(s) => {
                resolver
                    .resolve_address_reservation(&s)
                    .map_err(|err| GeneratorError {
                        error_kind: GeneratorErrorKind::NameResolverError(err),
                        span: inner.span,
                    })
            }
            v => invalid_type!(inner.span, v, ast::ValueKind::U32, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::AddressReservation),
    }
}

fn generate_static_address(
    value: &ast::ValueWithSpan,
    address_bech32_decoder: &AddressBech32Decoder,
) -> Result<ManifestAddress, GeneratorError> {
    match &value.value {
        ast::Value::Address(inner) => match &inner.value {
            ast::Value::String(s) => {
                // Check bech32 && entity type
                if let Ok((_, full_data)) = address_bech32_decoder.validate_and_decode(s) {
                    // Check length
                    if full_data.len() == NodeId::LENGTH {
                        return Ok(ManifestAddress::Static(NodeId(
                            full_data.try_into().unwrap(),
                        )));
                    }
                }
                return Err(GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidGlobalAddress(s.into()),
                    span: inner.span,
                });
            }
            v => return invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::Address),
    }
}

fn generate_named_address(
    value: &ast::ValueWithSpan,
    resolver: &mut NameResolver,
) -> Result<ManifestAddress, GeneratorError> {
    match &value.value {
        ast::Value::NamedAddress(inner) => match &inner.value {
            ast::Value::U32(n) => Ok(ManifestAddress::Named(ManifestNamedAddress(*n))),
            ast::Value::String(s) => resolver
                .resolve_named_address(&s)
                .map(|x| ManifestAddress::Named(x))
                .map_err(|err| GeneratorError {
                    error_kind: GeneratorErrorKind::NameResolverError(err),
                    span: inner.span,
                }),
            v => invalid_type!(inner.span, v, ast::ValueKind::U32, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::NamedAddress),
    }
}

fn generate_non_fungible_local_id(
    value: &ast::ValueWithSpan,
) -> Result<NonFungibleLocalId, GeneratorError> {
    match &value.value {
        ast::Value::NonFungibleLocalId(inner) => match &inner.value {
            ast::Value::String(s) => NonFungibleLocalId::from_str(s)
                // TODO: Consider more precise message by interpreting ParseNonFungibleLocalIdError
                .map_err(|_| GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidNonFungibleLocalId(s.into()),
                    span: inner.span,
                }),
            v => invalid_type!(inner.span, v, ast::ValueKind::String)?,
        },
        v => invalid_type!(value.span, v, ast::ValueKind::NonFungibleLocalId),
    }
}

fn generate_expression(value: &ast::ValueWithSpan) -> Result<ManifestExpression, GeneratorError> {
    match &value.value {
        ast::Value::Expression(inner) => match &inner.value {
            ast::Value::String(s) => match s.as_str() {
                "ENTIRE_WORKTOP" => Ok(ManifestExpression::EntireWorktop),
                "ENTIRE_AUTH_ZONE" => Ok(ManifestExpression::EntireAuthZone),
                _ => Err(GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidExpression(s.into()),
                    span: inner.span,
                }),
            },
            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::Expression),
    }
}

fn translate_parse_hash_error(err: ParseHashError) -> String {
    match err {
        ParseHashError::InvalidHex(_) => "invalid hex value".to_string(),
        ParseHashError::InvalidLength { actual, expected } => {
            format!("invalid hash length {}, expected {}", actual, expected)
        }
    }
}

fn generate_blob<B>(
    value: &ast::ValueWithSpan,
    blobs: &B,
) -> Result<ManifestBlobRef, GeneratorError>
where
    B: IsBlobProvider,
{
    match &value.value {
        ast::Value::Blob(inner) => match &inner.value {
            ast::Value::String(s) => {
                let hash = Hash::from_str(s).map_err(|err| GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidBlobHash {
                        actual: s.to_string(),
                        err: translate_parse_hash_error(err),
                    },
                    span: inner.span,
                })?;
                blobs.get_blob(&hash).ok_or(GeneratorError {
                    error_kind: GeneratorErrorKind::BlobNotFound(s.clone()),
                    span: inner.span,
                })?;
                Ok(ManifestBlobRef(hash.0))
            }
            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::Blob),
    }
}

fn generate_non_fungible_local_ids(
    value: &ast::ValueWithSpan,
) -> Result<Vec<NonFungibleLocalId>, GeneratorError> {
    match &value.value {
        ast::Value::Array(kind, values) => {
            if kind.value_kind != ast::ValueKind::NonFungibleLocalId {
                return Err(GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidAstType {
                        expected_value_kind: ast::ValueKind::NonFungibleLocalId,
                        actual: kind.value_kind.clone(),
                    },
                    span: kind.span,
                });
            }

            values
                .iter()
                .map(|v| generate_non_fungible_local_id(v))
                .collect()
        }
        v => invalid_type!(value.span, v, ast::ValueKind::Array),
    }
}

fn generate_byte_vec_from_hex(value: &ast::ValueWithSpan) -> Result<Vec<u8>, GeneratorError> {
    let bytes = match &value.value {
        ast::Value::String(s) => hex::decode(s).map_err(|_| GeneratorError {
            error_kind: GeneratorErrorKind::InvalidBytesHex(s.to_string()),
            span: value.span,
        })?,
        v => invalid_type!(value.span, v, ast::ValueKind::String)?,
    };
    Ok(bytes)
}

fn generate_subintent_hash(
    decoder: &TransactionHashBech32Decoder,
    value: &ast::ValueWithSpan,
) -> Result<SubintentHash, GeneratorError> {
    match &value.value {
        ast::Value::Intent(inner) => match &inner.value {
            ast::Value::String(s) => decoder.validate_and_decode(s).map_err(|_| GeneratorError {
                error_kind: GeneratorErrorKind::InvalidSubTransactionId(s.into()),
                span: inner.span,
            }),
            v => invalid_type!(inner.span, v, ast::ValueKind::String),
        },
        v => invalid_type!(value.span, v, ast::ValueKind::Intent),
    }
}

pub fn generate_typed_value<T: ManifestDecode + ScryptoDescribe, B>(
    value_with_span: &ast::ValueWithSpan,
    resolver: &mut NameResolver,
    address_bech32_decoder: &AddressBech32Decoder,
    blobs: &B,
) -> Result<T, GeneratorError>
where
    B: IsBlobProvider,
{
    let value = generate_value(
        value_with_span,
        None,
        resolver,
        address_bech32_decoder,
        blobs,
    )?;
    let encoded = manifest_encode(&value).map_err(|encode_error| GeneratorError {
        span: value_with_span.span,
        error_kind: GeneratorErrorKind::ArgumentCouldNotBeReadAsExpectedType {
            type_name: core::any::type_name::<T>().to_string(),
            error_message: format!("{encode_error:?}"),
        },
    })?;
    let decoded =
        manifest_decode_with_nice_error(&encoded).map_err(|error_message| GeneratorError {
            span: value_with_span.span,
            error_kind: GeneratorErrorKind::ArgumentCouldNotBeReadAsExpectedType {
                type_name: core::any::type_name::<T>().to_string(),
                error_message,
            },
        })?;
    Ok(decoded)
}

pub fn generate_value<B>(
    value_with_span: &ast::ValueWithSpan,
    expected_value_kind: Option<&ast::ValueKindWithSpan>,
    resolver: &mut NameResolver,
    address_bech32_decoder: &AddressBech32Decoder,
    blobs: &B,
) -> Result<ManifestValue, GeneratorError>
where
    B: IsBlobProvider,
{
    if let Some(value_kind) = expected_value_kind {
        // We check sbor value kinds to permit structures which are SBOR-compatible,
        // even if they don't look immediately valid in SBOR land
        // e.g. an Array<Tuple>(NonFungibleGlobalId("..."))
        // e.g. an Array<Vec>(Bytes("..."))
        if value_kind.sbor_value_kind() != value_with_span.value_kind().sbor_value_kind() {
            return Err(GeneratorError {
                span: value_with_span.span,
                error_kind: GeneratorErrorKind::UnexpectedValueKind {
                    expected_value_kind: value_kind.value_kind,
                    actual_value: value_with_span.value.clone(),
                },
            });
        }
    }

    match &value_with_span.value {
        // ==============
        // Basic types
        // ==============
        ast::Value::Bool(value) => Ok(Value::Bool { value: *value }),
        ast::Value::I8(value) => Ok(Value::I8 { value: *value }),
        ast::Value::I16(value) => Ok(Value::I16 { value: *value }),
        ast::Value::I32(value) => Ok(Value::I32 { value: *value }),
        ast::Value::I64(value) => Ok(Value::I64 { value: *value }),
        ast::Value::I128(value) => Ok(Value::I128 { value: *value }),
        ast::Value::U8(value) => Ok(Value::U8 { value: *value }),
        ast::Value::U16(value) => Ok(Value::U16 { value: *value }),
        ast::Value::U32(value) => Ok(Value::U32 { value: *value }),
        ast::Value::U64(value) => Ok(Value::U64 { value: *value }),
        ast::Value::U128(value) => Ok(Value::U128 { value: *value }),
        ast::Value::String(value) => Ok(Value::String {
            value: value.clone(),
        }),
        ast::Value::Tuple(fields) => Ok(Value::Tuple {
            fields: generate_singletons(&fields, None, resolver, address_bech32_decoder, blobs)?,
        }),
        ast::Value::Enum(discriminator, fields) => Ok(Value::Enum {
            discriminator: discriminator.clone(),
            fields: generate_singletons(&fields, None, resolver, address_bech32_decoder, blobs)?,
        }),
        ast::Value::Array(element_type, elements) => {
            let element_value_kind = element_type.sbor_value_kind()?;
            Ok(Value::Array {
                element_value_kind,
                elements: generate_singletons(
                    &elements,
                    Some(element_type),
                    resolver,
                    address_bech32_decoder,
                    blobs,
                )?,
            })
        }
        ast::Value::Map(key_type, value_type, entries) => {
            let key_value_kind = key_type.sbor_value_kind()?;
            let value_value_kind = value_type.sbor_value_kind()?;
            Ok(Value::Map {
                key_value_kind,
                value_value_kind,
                entries: generate_kv_entries(
                    &entries,
                    &key_type,
                    &value_type,
                    resolver,
                    address_bech32_decoder,
                    blobs,
                )?,
            })
        }
        // ==============
        // Aliases
        // ==============
        ast::Value::Some(value) => Ok(Value::Enum {
            discriminator: OPTION_VARIANT_SOME,
            fields: vec![generate_value(
                &value,
                None,
                resolver,
                address_bech32_decoder,
                blobs,
            )?],
        }),
        ast::Value::None => Ok(Value::Enum {
            discriminator: OPTION_VARIANT_NONE,
            fields: vec![],
        }),
        ast::Value::Ok(value) => Ok(Value::Enum {
            discriminator: RESULT_VARIANT_OK,
            fields: vec![generate_value(
                &value,
                None,
                resolver,
                address_bech32_decoder,
                blobs,
            )?],
        }),
        ast::Value::Err(value) => Ok(Value::Enum {
            discriminator: RESULT_VARIANT_ERR,
            fields: vec![generate_value(
                &value,
                None,
                resolver,
                address_bech32_decoder,
                blobs,
            )?],
        }),
        ast::Value::Bytes(value) => {
            let bytes = generate_byte_vec_from_hex(&value)?;
            Ok(Value::Array {
                element_value_kind: ValueKind::U8,
                elements: bytes.iter().map(|i| Value::U8 { value: *i }).collect(),
            })
        }
        ast::Value::NonFungibleGlobalId(value) => {
            let global_id = match &value.value {
                ast::Value::String(s) => NonFungibleGlobalId::try_from_canonical_string(
                    address_bech32_decoder,
                    s.as_str(),
                )
                .map_err(|_| GeneratorError {
                    error_kind: GeneratorErrorKind::InvalidNonFungibleGlobalId,
                    span: value.span,
                }),
                v => invalid_type!(value.span, v, ast::ValueKind::String)?,
            }?;
            Ok(Value::Tuple {
                fields: vec![
                    Value::Custom {
                        value: ManifestCustomValue::Address(ManifestAddress::Static(
                            global_id.resource_address().into(),
                        )),
                    },
                    Value::Custom {
                        value: ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(
                            global_id.local_id().clone(),
                        )),
                    },
                ],
            })
        }
        // ==============
        // Custom Types
        // ==============
        ast::Value::Address(_) => generate_static_address(value_with_span, address_bech32_decoder)
            .map(|v| Value::Custom {
                value: ManifestCustomValue::Address(v),
            }),
        ast::Value::NamedAddress(_) => {
            generate_named_address(value_with_span, resolver).map(|v| Value::Custom {
                value: ManifestCustomValue::Address(v),
            })
        }
        ast::Value::Bucket(_) => {
            generate_bucket(value_with_span, resolver).map(|(v, _span)| Value::Custom {
                value: ManifestCustomValue::Bucket(v),
            })
        }
        ast::Value::Proof(_) => {
            generate_proof(value_with_span, resolver).map(|(v, _span)| Value::Custom {
                value: ManifestCustomValue::Proof(v),
            })
        }
        ast::Value::Expression(_) => generate_expression(value_with_span).map(|v| Value::Custom {
            value: ManifestCustomValue::Expression(v),
        }),
        ast::Value::Blob(_) => generate_blob(value_with_span, blobs).map(|v| Value::Custom {
            value: ManifestCustomValue::Blob(v),
        }),
        ast::Value::Decimal(_) => generate_decimal(value_with_span).map(|v| Value::Custom {
            value: ManifestCustomValue::Decimal(from_decimal(v)),
        }),
        ast::Value::PreciseDecimal(_) => {
            generate_precise_decimal(value_with_span).map(|v| Value::Custom {
                value: ManifestCustomValue::PreciseDecimal(from_precise_decimal(v)),
            })
        }
        ast::Value::NonFungibleLocalId(_) => {
            generate_non_fungible_local_id(value_with_span).map(|v| Value::Custom {
                value: ManifestCustomValue::NonFungibleLocalId(from_non_fungible_local_id(v)),
            })
        }
        ast::Value::AddressReservation(_) => {
            generate_address_reservation(value_with_span, resolver).map(|v| Value::Custom {
                value: ManifestCustomValue::AddressReservation(v),
            })
        }
        ast::Value::NamedIntent(_) => {
            return Err(GeneratorError {
                error_kind: GeneratorErrorKind::NamedIntentCannotBeUsedInValue,
                span: value_with_span.span,
            });
        }
        ast::Value::Intent(_) => {
            return Err(GeneratorError {
                error_kind: GeneratorErrorKind::IntentCannotBeUsedInValue,
                span: value_with_span.span,
            });
        }
    }
}

fn generate_singletons<B>(
    elements: &Vec<ast::ValueWithSpan>,
    expected_value_kind: Option<&ast::ValueKindWithSpan>,
    resolver: &mut NameResolver,
    address_bech32_decoder: &AddressBech32Decoder,
    blobs: &B,
) -> Result<Vec<ManifestValue>, GeneratorError>
where
    B: IsBlobProvider,
{
    let mut result = vec![];
    for element in elements {
        result.push(generate_value(
            element,
            expected_value_kind,
            resolver,
            address_bech32_decoder,
            blobs,
        )?);
    }
    Ok(result)
}

fn generate_kv_entries<B>(
    entries: &[(ast::ValueWithSpan, ast::ValueWithSpan)],
    key_value_kind: &ValueKindWithSpan,
    value_value_kind: &ValueKindWithSpan,
    resolver: &mut NameResolver,
    address_bech32_decoder: &AddressBech32Decoder,
    blobs: &B,
) -> Result<Vec<(ManifestValue, ManifestValue)>, GeneratorError>
where
    B: IsBlobProvider,
{
    let mut result = vec![];
    for entry in entries {
        let key = generate_value(
            &entry.0,
            Some(key_value_kind),
            resolver,
            address_bech32_decoder,
            blobs,
        )?;
        let value = generate_value(
            &entry.1,
            Some(value_value_kind),
            resolver,
            address_bech32_decoder,
            blobs,
        )?;
        result.push((key, value));
    }
    Ok(result)
}

pub fn generator_error_diagnostics(
    s: &str,
    err: GeneratorError,
    style: CompileErrorDiagnosticsStyle,
) -> String {
    // The title should be a little longer, and include context about what triggered
    // the error. The label is inline next to arrows pointing to the span which is
    // invalid, so can be shorter.
    // These will appear roughly like below:
    //
    //   error: <TITLE>
    //     ...
    //   12 |       Bytes(1u32),
    //      |       ^^^^^ <LABEL>
    let (title, label) = match err.error_kind {
        GeneratorErrorKind::InvalidAstType {
            expected_value_kind,
            actual,
        } => {
            let title = format!("expected {:?}, found {:?}", expected_value_kind, actual,);
            let label = format!("expected {:?}", expected_value_kind);
            (title, label)
        }
        GeneratorErrorKind::InvalidAstValue {
            expected_value_kinds,
            actual,
        } => {
            let expected_value_kinds = expected_value_kinds
                .iter()
                .map(|vk| vk.to_string())
                .collect::<Vec<_>>()
                .join(" or ");
            let actual_value_kind = actual.value_kind();
            let title = format!("expected {expected_value_kinds}, found {actual_value_kind}",);
            let label = format!("expected {expected_value_kinds}");
            (title, label)
        }
        GeneratorErrorKind::UnexpectedValueKind {
            expected_value_kind,
            actual_value: actual,
        } => {
            let title = format!(
                "expected {}, found {}",
                expected_value_kind,
                actual.value_kind(),
            );
            let label = format!("expected {}", expected_value_kind);
            (title, label)
        }
        GeneratorErrorKind::InvalidPackageAddress(string) => {
            let title = format!("invalid package address '{}'", string);
            (title, "invalid package address".to_string())
        }
        GeneratorErrorKind::InvalidResourceAddress(string) => {
            let title = format!("invalid resource address '{}'", string);
            (title, "invalid resource address".to_string())
        }
        GeneratorErrorKind::InvalidDecimal { actual, err } => {
            let title = format!("invalid decimal '{}' - {}", actual, err);
            (title, "invalid decimal".to_string())
        }
        GeneratorErrorKind::InvalidPreciseDecimal { actual, err } => {
            let title = format!("invalid precise decimal '{}' - {}", actual, err);
            (title, "invalid precise decimal".to_string())
        }
        GeneratorErrorKind::InvalidNonFungibleLocalId(string) => {
            let title = format!("invalid non-fungible local id '{}'", string);
            (title, "invalid non-fungible local id".to_string())
        }
        GeneratorErrorKind::InvalidNonFungibleGlobalId => {
            let title = format!("invalid non-fungible global id");
            (title, "invalid non-fungible global id".to_string())
        }
        GeneratorErrorKind::InvalidExpression(string) => {
            let title = format!("invalid expression '{}'", string);
            (title, "invalid expression".to_string())
        }
        GeneratorErrorKind::InvalidBlobHash { actual, err } => {
            let title = format!("invalid blob hash '{}' - {}", actual, err);
            (title, "invalid blob hash".to_string())
        }
        GeneratorErrorKind::BlobNotFound(string) => {
            let title = format!("blob with hash '{}' not found", string);
            (title, "blob not found".to_string())
        }
        GeneratorErrorKind::InvalidBytesHex(string) => {
            let title = format!("invalid hex value '{}'", string);
            (title, "invalid hex value".to_string())
        }
        GeneratorErrorKind::NameResolverError(error) => match error {
            NameResolverError::UndefinedBucket(string) => {
                let title = format!("undefined bucket '{}'", string);
                (title, "undefined bucket".to_string())
            }
            NameResolverError::UndefinedProof(string) => {
                let title = format!("undefined proof '{}'", string);
                (title, "undefined proof".to_string())
            }
            NameResolverError::UndefinedAddressReservation(string) => {
                let title = format!("undefined address reservation '{}'", string);
                (title, "undefined address reservation".to_string())
            }
            NameResolverError::UndefinedNamedAddress(string) => {
                let title = format!("undefined named address '{}'", string);
                (title, "undefined named address".to_string())
            }
            NameResolverError::UndefinedIntent(string) => {
                let title = format!("undefined intent '{}'", string);
                (title, "undefined intent".to_string())
            }
            NameResolverError::NamedAlreadyDefined(string) => {
                let title = format!("name already defined '{}'", string);
                (title, "name already defined".to_string())
            }
        },
        GeneratorErrorKind::IdValidationError { err, name } => {
            match err {
                ManifestIdValidationError::BucketNotFound(bucket_id) => {
                    let title = if let Some(name) = name {
                        format!("bucket '{}' not found", name)
                    } else {
                        format!("bucket id '{:?}' not found", bucket_id)
                    };
                    (title, "bucket not found".to_string())
                }
                ManifestIdValidationError::ProofNotFound(proof_id) => {
                    let title = if let Some(name) = name {
                        format!("proof '{}' not found", name)
                    } else {
                        format!("proof id '{:?}' not found", proof_id)
                    };
                    (title, "proof not found".to_string())
                }
                ManifestIdValidationError::BucketLocked(bucket_id) => {
                    let title = if let Some(name) = name {
                        format!("cannot consume bucket '{}' because it's believed to be currently locked", name)
                    } else {
                        format!("cannot consume bucket id '{:?}' because it's believed to be currently locked", bucket_id)
                    };
                    (title, "bucket locked".to_string())
                }
                ManifestIdValidationError::AddressReservationNotFound(reservation) => {
                    let title = if let Some(name) = name {
                        format!("address reservation '{}' not found", name)
                    } else {
                        format!("address reservation id '{:?}' not found", reservation)
                    };
                    (title, "address reservation not found".to_string())
                }
                ManifestIdValidationError::AddressNotFound(address) => {
                    let title = if let Some(name) = name {
                        format!("address '{}' not found", name)
                    } else {
                        format!("address id '{:?}' not found", address)
                    };
                    (title, "address not found".to_string())
                }
                ManifestIdValidationError::IntentNotFound(intent) => {
                    let title = if let Some(name) = name {
                        format!("intent '{}' not found", name)
                    } else {
                        format!("intent id '{:?}' not found", intent)
                    };
                    (title, "intent not found".to_string())
                }
            }
        }
        GeneratorErrorKind::InvalidGlobalAddress(string) => {
            let title = format!("invalid global address '{}'", string);
            (title, "invalid global address".to_string())
        }
        GeneratorErrorKind::InvalidInternalAddress(string) => {
            let title = format!("invalid internal address '{}'", string);
            (title, "invalid internal address".to_string())
        }
        GeneratorErrorKind::InvalidSubTransactionId(string) => {
            let title = format!("invalid sub transaction id '{}'", string);
            (title, "invalid sub transaction id".to_string())
        }
        GeneratorErrorKind::InstructionNotSupportedInManifestVersion => {
            let title = format!("unsupported instruction for this manifest version");
            (title, "unsupported instruction".to_string())
        }
        GeneratorErrorKind::ManifestBuildError(ManifestBuildError::DuplicateChildSubintentHash) => {
            let title = format!("child subintents cannot have the same hash");
            (title, "duplicate hash".to_string())
        }
        GeneratorErrorKind::ManifestBuildError(
            ManifestBuildError::PreallocatedAddressesUnsupportedByManifestType,
        ) => {
            let title = format!("preallocated addresses are not supported in this manifest type");
            (title, "unsupported instruction".to_string())
        }
        GeneratorErrorKind::ManifestBuildError(
            ManifestBuildError::ChildSubintentsUnsupportedByManifestType,
        ) => {
            let title = format!("child subintents are not supported in this manifest type");
            (title, "unsupported instruction".to_string())
        }
        GeneratorErrorKind::HeaderInstructionMustComeFirst => {
            let title = format!(
                "a psuedo-instruction such as USE_CHILD must come before all other instructions"
            );
            (title, "must be at the start of the manifest".to_string())
        }
        GeneratorErrorKind::IntentCannotBeUsedInValue => {
            let title = format!("an Intent(...) cannot currently be used inside a value");
            (title, "cannot be used inside a value".to_string())
        }
        GeneratorErrorKind::NamedIntentCannotBeUsedInValue => {
            let title = format!("a NamedIntent(...) cannot currently be used inside a value");
            (title, "cannot be used inside a value".to_string())
        }
        GeneratorErrorKind::IntentCannotBeUsedAsValueKind => {
            let title = format!("an Intent cannot be used as a value kind");
            (title, "cannot be used as a value kind".to_string())
        }
        GeneratorErrorKind::NamedIntentCannotBeUsedAsValueKind => {
            let title = format!("a NamedIntent cannot be used as a value kind");
            (title, "cannot be used as a value kind".to_string())
        }
        GeneratorErrorKind::ArgumentCouldNotBeReadAsExpectedType {
            type_name,
            error_message,
        } => {
            let title = format!(
                "an argument's structure does not fit with the {type_name} type. {error_message}"
            );
            let description = format!("cannot be decoded as a {type_name}");
            (title, description)
        }
    };

    create_snippet(s, &err.span, &title, &label, style)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::lexer::tokenize;
    use crate::manifest::parser::{Parser, ParserError, ParserErrorKind, PARSER_MAX_DEPTH};
    use crate::manifest::token::{Position, Span};
    use crate::{position, span};
    use radix_common::address::AddressBech32Decoder;
    use radix_common::constants::CONSENSUS_MANAGER;
    use radix_common::crypto::Secp256k1PrivateKey;
    use radix_common::manifest_args;
    use radix_common::network::NetworkDefinition;
    use radix_common::traits::NonFungibleData;
    use radix_common::types::{ComponentAddress, PackageAddress};
    use radix_engine_interface::blueprints::consensus_manager::ConsensusManagerCreateValidatorManifestInput;
    use radix_engine_interface::blueprints::resource::{
        NonFungibleDataSchema, NonFungibleResourceManagerMintManifestInput,
        NonFungibleResourceManagerMintRuidManifestInput,
    };
    use radix_engine_interface::object_modules::metadata::MetadataValue;
    use radix_engine_interface::object_modules::ModuleConfig;
    use radix_engine_interface::types::PackageRoyaltyConfig;
    use radix_rust::prelude::IndexMap;
    use scrypto::radix_blueprint_schema_init::BlueprintStateSchemaInit;

    #[macro_export]
    macro_rules! generate_value_ok {
        ( $s:expr,   $expected:expr ) => {{
            let value = Parser::new(tokenize($s).unwrap(), PARSER_MAX_DEPTH)
                .unwrap()
                .parse_value()
                .unwrap();
            let mut resolver = NameResolver::new();
            assert_eq!(
                generate_value(
                    &value,
                    None,
                    &mut resolver,
                    &AddressBech32Decoder::new(&NetworkDefinition::simulator()),
                    &BlobProvider::default()
                ),
                Ok($expected)
            );
        }};
    }

    #[macro_export]
    macro_rules! generate_instruction_ok {
        ( $s:expr, $expected:expr $(,)? ) => {{
            // If you use the following output for test cases, make sure you've checked the diff
            // println!("{}", crate::manifest::decompile(&[$expected.clone()], &NetworkDefinition::simulator()).unwrap());
            let instruction = Parser::new(tokenize($s).unwrap(), PARSER_MAX_DEPTH)
                .unwrap()
                .parse_instruction()
                .unwrap();
            let mut id_validator = BasicManifestValidator::new();
            let mut resolver = NameResolver::new();
            assert_eq!(
                generate_instruction(
                    &instruction,
                    &mut id_validator,
                    &mut resolver,
                    &AddressBech32Decoder::new(&NetworkDefinition::simulator()),
                    &MockBlobProvider::default()
                ),
                Ok($expected.into())
            );
        }}
    }

    #[macro_export]
    macro_rules! generate_value_error {
        ( $s:expr, $expected:expr ) => {{
            let value = Parser::new(tokenize($s).unwrap(), PARSER_MAX_DEPTH)
                .unwrap()
                .parse_value()
                .unwrap();
            match generate_value(
                &value,
                None,
                &mut NameResolver::new(),
                &AddressBech32Decoder::new(&NetworkDefinition::simulator()),
                &BlobProvider::default(),
            ) {
                Ok(_) => {
                    panic!("Expected {:?} but no error is thrown", $expected);
                }
                Err(e) => {
                    assert_eq!(e, $expected);
                }
            }
        }};
    }

    #[test]
    fn test_value() {
        generate_value_ok!(r#"Tuple()"#, Value::Tuple { fields: vec![] });
        generate_value_ok!(r#"true"#, Value::Bool { value: true });
        generate_value_ok!(r#"false"#, Value::Bool { value: false });
        generate_value_ok!(r#"1i8"#, Value::I8 { value: 1 });
        generate_value_ok!(r#"1i128"#, Value::I128 { value: 1 });
        generate_value_ok!(r#"1u8"#, Value::U8 { value: 1 });
        generate_value_ok!(r#"1u128"#, Value::U128 { value: 1 });
        generate_value_ok!(
            r#"Tuple(Bucket(1u32), Proof(2u32), "bar")"#,
            Value::Tuple {
                fields: vec![
                    Value::Custom {
                        value: ManifestCustomValue::Bucket(ManifestBucket(1))
                    },
                    Value::Custom {
                        value: ManifestCustomValue::Proof(ManifestProof(2))
                    },
                    Value::String {
                        value: "bar".into()
                    }
                ]
            }
        );
        generate_value_ok!(
            r#"Tuple(Decimal("1"))"#,
            Value::Tuple {
                fields: vec![Value::Custom {
                    value: ManifestCustomValue::Decimal(from_decimal(
                        Decimal::from_str("1").unwrap()
                    ))
                },]
            }
        );
        generate_value_ok!(r#"Tuple()"#, Value::Tuple { fields: vec![] });
        generate_value_ok!(
            r#"Enum<0u8>( "abc")"#,
            Value::Enum {
                discriminator: 0,
                fields: vec![Value::String {
                    value: "abc".to_owned()
                }]
            }
        );
        generate_value_ok!(
            r#"Enum<1u8>()"#,
            Value::Enum {
                discriminator: 1,
                fields: vec![]
            }
        );
        generate_value_ok!(
            r#"Enum<AccessRule::AllowAll>()"#,
            Value::Enum {
                discriminator: 0,
                fields: vec![]
            }
        );
        generate_value_ok!(
            r#"Expression("ENTIRE_WORKTOP")"#,
            Value::Custom {
                value: ManifestCustomValue::Expression(ManifestExpression::EntireWorktop)
            }
        );
        generate_value_ok!(
            r#"Expression("ENTIRE_AUTH_ZONE")"#,
            Value::Custom {
                value: ManifestCustomValue::Expression(ManifestExpression::EntireAuthZone)
            }
        );
    }

    #[test]
    fn test_failures() {
        generate_value_error!(
            r#"Address(100u32)"#,
            GeneratorError {
                error_kind: GeneratorErrorKind::InvalidAstValue {
                    expected_value_kinds: vec![ast::ValueKind::String],
                    actual: ast::Value::U32(100),
                },
                span: span!(start = (8, 0, 8), end = (14, 0, 14)),
            }
        );
        generate_value_error!(
            r#"Address("invalid_package_address")"#,
            GeneratorError {
                error_kind: GeneratorErrorKind::InvalidGlobalAddress(
                    "invalid_package_address".into(),
                ),
                span: span!(start = (8, 0, 8), end = (33, 0, 33))
            }
        );
        generate_value_error!(
            r#"Decimal("invalid_decimal")"#,
            GeneratorError {
                error_kind: GeneratorErrorKind::InvalidDecimal {
                    actual: "invalid_decimal".to_string(),
                    err: "InvalidDigit".to_string(),
                },
                span: span!(start = (8, 0, 8), end = (25, 0, 25))
            }
        );
        generate_value_error!(
            r#"Decimal("i")"#,
            GeneratorError {
                error_kind: GeneratorErrorKind::InvalidDecimal {
                    actual: "i".to_string(),
                    err: "InvalidDigit".to_string(),
                },
                span: span!(start = (8, 0, 8), end = (11, 0, 11))
            }
        );

        // Test unicode and spans
        generate_value_error!(
            r#"Decimal("🤓")"#,
            GeneratorError {
                error_kind: GeneratorErrorKind::InvalidDecimal {
                    actual: "🤓".to_string(),
                    err: "InvalidDigit".to_string(),
                },
                span: span!(start = (8, 0, 8), end = (11, 0, 11))
            }
        );

        generate_value_error!(
            r#"Decimal("\uD83D\uDC69")"#,
            GeneratorError {
                error_kind: GeneratorErrorKind::InvalidDecimal {
                    actual: "\u{1f469}".to_string(), // this is a value of '👩'
                    err: "InvalidDigit".to_string(),
                },
                span: span!(start = (8, 0, 8), end = (22, 0, 22))
            }
        );
        generate_value_error!(
            r#"Decimal("👩")"#,
            GeneratorError {
                error_kind: GeneratorErrorKind::InvalidDecimal {
                    actual: "👩".to_string(),
                    err: "InvalidDigit".to_string(),
                },
                span: span!(start = (8, 0, 8), end = (11, 0, 11))
            }
        );
    }

    #[test]
    fn test_instructions() {
        let address_bech32_decoder = AddressBech32Decoder::new(&NetworkDefinition::simulator());
        let package_address = PackageAddress::try_from_bech32(
            &address_bech32_decoder,
            "package_sim1p4r4955skdjq9swg8s5jguvcjvyj7tsxct87a9z6sw76cdfd2jg3zk".into(),
        )
        .unwrap();
        let component = ComponentAddress::try_from_bech32(
            &address_bech32_decoder,
            "component_sim1cqvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cvemygpmu",
        )
        .unwrap();
        let resource_address = ResourceAddress::try_from_bech32(
            &address_bech32_decoder,
            "resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez",
        )
        .unwrap();

        generate_instruction_ok!(
            r#"TAKE_FROM_WORKTOP  Address("resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez")  Decimal("1")  Bucket("xrd_bucket");"#,
            TakeFromWorktop {
                amount: Decimal::from(1),
                resource_address,
            },
        );
        generate_instruction_ok!(
            r#"TAKE_ALL_FROM_WORKTOP  Address("resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez")  Bucket("xrd_bucket");"#,
            TakeAllFromWorktop { resource_address },
        );
        generate_instruction_ok!(
            r#"ASSERT_WORKTOP_CONTAINS  Address("resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez")  Decimal("1");"#,
            AssertWorktopContains {
                amount: Decimal::from(1),
                resource_address,
            },
        );
        generate_instruction_ok!(
            r#"CALL_FUNCTION  Address("package_sim1p4r4955skdjq9swg8s5jguvcjvyj7tsxct87a9z6sw76cdfd2jg3zk")  "Airdrop"  "new"  500u32  PreciseDecimal("120");"#,
            CallFunction {
                package_address: package_address.into(),
                blueprint_name: "Airdrop".into(),
                function_name: "new".to_string(),
                args: manifest_args!(500u32, pdec!("120")).into()
            },
        );
        generate_instruction_ok!(
            r#"CALL_METHOD  Address("component_sim1cqvgx33089ukm2pl97pv4max0x40ruvfy4lt60yvya744cvemygpmu")  "refill";"#,
            CallMethod {
                address: component.into(),
                method_name: "refill".to_string(),
                args: manifest_args!().into()
            },
        );
        generate_instruction_ok!(
            r#"MINT_FUNGIBLE Address("resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez") Decimal("100");"#,
            CallMethod {
                address: resource_address.into(),
                method_name: "mint".to_string(),
                args: manifest_args!(dec!("100")).into()
            },
        );
    }

    #[test]
    fn test_publish_instruction() {
        generate_instruction_ok!(
            r#"PUBLISH_PACKAGE_ADVANCED Blob("a710f0959d8e139b3c1ca74ac4fcb9a95ada2c82e7f563304c5487e0117095c0") Map<String, Tuple>() Map<String, Enum>() Map<String, Enum>() Map<String, Enum>();"#,
            CallFunction {
                package_address: PACKAGE_PACKAGE.into(),
                blueprint_name: PACKAGE_BLUEPRINT.to_string(),
                function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
                args: manifest_args!(
                    ManifestBlobRef(
                        hex::decode(
                            "a710f0959d8e139b3c1ca74ac4fcb9a95ada2c82e7f563304c5487e0117095c0"
                        )
                        .unwrap()
                        .try_into()
                        .unwrap()
                    ),
                    IndexMap::<String, BlueprintStateSchemaInit, _>::new(),
                    IndexMap::<String, PackageRoyaltyConfig, _>::new(),
                    IndexMap::<String, MetadataValue, _>::new(),
                    RoleAssignmentInit::new()
                )
                .into(),
            },
        );
    }

    #[test]
    fn test_create_non_fungible_instruction() {
        generate_instruction_ok!(
            r#"CREATE_NON_FUNGIBLE_RESOURCE
                Enum<0u8>()
                Enum<NonFungibleIdType::Integer>()
                false
                Enum<0u8>(
                    Enum<0u8>(
                        Tuple(
                            Array<Enum>(),
                            Array<Tuple>(),
                            Array<Enum>()
                        )
                    ),
                    Enum<0u8>(66u8),
                    Array<String>()
                )
                Tuple(
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>()
                )
                Tuple(
                    Map<String, Tuple>(
                        "name" => Tuple(
                            Enum<Option::Some>(Enum<Metadata::String>("Token")),
                            true
                        ),
                    ),
                    Map<String, Enum>()
                )
                Enum<0u8>();"#,
            CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerCreateManifestInput {
                        owner_role: OwnerRole::None,
                        id_type: NonFungibleIdType::Integer,
                        track_total_supply: false,
                        non_fungible_schema:
                            NonFungibleDataSchema::new_local_without_self_package_replacement::<()>(
                            ),
                        metadata: metadata! {
                            init {
                                "name" => "Token".to_string(), locked;
                            }
                        },
                        resource_roles: NonFungibleResourceRoles::default(),
                        address_reservation: None,
                    }
                ),
            },
        );
    }

    #[derive(ScryptoSbor)]
    struct MyNonFungibleData {
        name: String,
        description: String,
        stored_number: Decimal,
    }

    // Because we can't import the derive trait
    impl NonFungibleData for MyNonFungibleData {
        const MUTABLE_FIELDS: &'static [&'static str] = &["description", "stored_number"];
    }

    #[test]
    fn test_generate_non_fungible_instruction_with_specific_data() {
        // This test is mostly to assist with generating manifest instructions for the testing harness
        let manifest = ManifestBuilder::new_v1()
            .call_function(
                RESOURCE_PACKAGE,
                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                NonFungibleResourceManagerCreateManifestInput {
                    owner_role: OwnerRole::None,
                    track_total_supply: false,
                    id_type: NonFungibleIdType::Integer,
                    non_fungible_schema:
                        NonFungibleDataSchema::new_local_without_self_package_replacement::<
                            MyNonFungibleData,
                        >(),
                    resource_roles: NonFungibleResourceRoles::default(),
                    metadata: metadata!(),
                    address_reservation: None,
                },
            )
            .build();
        println!(
            "{}",
            crate::manifest::decompile(&manifest, &NetworkDefinition::simulator()).unwrap()
        );
    }

    #[test]
    fn test_create_non_fungible_with_initial_supply_instruction() {
        generate_instruction_ok!(
            r##"CREATE_NON_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY
                Enum<0u8>()
                Enum<NonFungibleIdType::Integer>()
                false
                Enum<0u8>(
                    Enum<0u8>(
                        Tuple(
                            Array<Enum>(),
                            Array<Tuple>(),
                            Array<Enum>()
                        )
                    ),
                    Enum<0u8>(66u8),
                    Array<String>()
                )
                Map<NonFungibleLocalId, Tuple>(
                    NonFungibleLocalId("#1#") => Tuple(
                        Tuple(
                            "Hello World",
                            Decimal("12")
                        )
                    )
                )
                Tuple(
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>()
                )
                Tuple(
                    Map<String, Tuple>(
                        "name" => Tuple(Enum<Option::Some>(Enum<Metadata::String>("Token")), true)
                    ),
                    Map<String, Enum>()
                )
                Enum<0u8>()
            ;"##,
            CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        owner_role: OwnerRole::None,
                        track_total_supply: false,
                        id_type: NonFungibleIdType::Integer,
                        non_fungible_schema:
                            NonFungibleDataSchema::new_local_without_self_package_replacement::<()>(
                            ),
                        resource_roles: NonFungibleResourceRoles::default(),
                        metadata: metadata! {
                            init {
                                "name" => "Token".to_string(), locked;
                            }
                        },
                        entries: indexmap!(
                            NonFungibleLocalId::integer(1) =>
                            (to_manifest_value_and_unwrap!(&(
                                String::from("Hello World"),
                                dec!("12")
                            )),),
                        ),
                        address_reservation: None,
                    }
                ),
            },
        );
    }

    #[test]
    fn test_create_fungible_instruction() {
        generate_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE
                Enum<0u8>()
                false
                18u8
                Tuple(
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>()
                )
                Tuple(
                    Map<String, Tuple>(
                        "name" => Tuple(Enum<Option::Some>(Enum<Metadata::String>("Token")), false)
                    ),
                    Map<String, Enum>()
                )
                Enum<0u8>()
            ;"#,
            CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&FungibleResourceManagerCreateManifestInput {
                    owner_role: OwnerRole::None,
                    track_total_supply: false,
                    divisibility: 18,
                    resource_roles: FungibleResourceRoles::default(),
                    metadata: metadata! {
                        init {
                            "name" => "Token".to_owned(), updatable;
                        }
                    },
                    address_reservation: None,
                }),
            },
        );
    }

    #[test]
    fn test_create_fungible_with_initial_supply_instruction() {
        generate_instruction_ok!(
            r#"CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY
                Enum<0u8>()
                false
                18u8
                Decimal("500")
                Tuple(
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>(),
                    Enum<0u8>()
                )
                Tuple(
                    Map<String, Tuple>(
                        "name" => Tuple(Enum<Option::Some>(Enum<Metadata::String>("Token")), false)
                    ),
                    Map<String, Enum>()
                )
                Enum<0u8>()
            ;"#,
            CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &FungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        owner_role: OwnerRole::None,
                        track_total_supply: false,
                        divisibility: 18,
                        initial_supply: "500".parse().unwrap(),
                        resource_roles: FungibleResourceRoles::default(),
                        metadata: metadata! {
                            init {
                                "name" => "Token".to_owned(), updatable;
                            }
                        },
                        address_reservation: None,
                    }
                )
            },
        );
    }

    #[test]
    fn test_mint_non_fungible_instruction() {
        let address_bech32_decoder = AddressBech32Decoder::new(&NetworkDefinition::simulator());
        let resource_address = ResourceAddress::try_from_bech32(
            &address_bech32_decoder,
            "resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez",
        )
        .unwrap();

        generate_instruction_ok!(
            r##"
            MINT_NON_FUNGIBLE
                Address("resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez")
                Map<NonFungibleLocalId, Tuple>(NonFungibleLocalId("#1#") => Tuple(Tuple("Hello World", Decimal("12"))));
            "##,
            CallMethod {
                address: resource_address.into(),
                method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&NonFungibleResourceManagerMintManifestInput {
                    entries: indexmap!(
                        NonFungibleLocalId::integer(1) =>
                        (to_manifest_value_and_unwrap!(&(
                            String::from("Hello World"),
                            dec!("12")
                        )),)
                    )
                })
            },
        );
    }

    #[test]
    fn test_mint_ruid_non_fungible_instruction() {
        let address_bech32_decoder = AddressBech32Decoder::new(&NetworkDefinition::simulator());
        let resource_address = ResourceAddress::try_from_bech32(
            &address_bech32_decoder,
            "resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez",
        )
        .unwrap();

        generate_instruction_ok!(
            r#"
            MINT_RUID_NON_FUNGIBLE
                Address("resource_sim1thvwu8dh6lk4y9mntemkvj25wllq8adq42skzufp4m8wxxuemugnez")
                Array<Tuple>(
                    Tuple(Tuple("Hello World", Decimal("12")))
                );
            "#,
            CallMethod {
                address: resource_address.into(),
                method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerMintRuidManifestInput {
                        entries: Vec::from([(to_manifest_value_and_unwrap!(&(
                            String::from("Hello World"),
                            dec!("12")
                        )),),])
                    }
                ),
            },
        );
    }

    #[test]
    fn test_create_validator_instruction() {
        let tokens = tokenize(
            r#"
            CREATE_VALIDATOR Bytes("02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5") Decimal("1") Bucket("xrd_bucket");
            "#
        ).unwrap();
        let instruction = Parser::new(tokens, PARSER_MAX_DEPTH)
            .unwrap()
            .parse_instruction()
            .unwrap();
        let mut id_validator = BasicManifestValidator::new();
        let mut resolver = NameResolver::new();
        resolver
            .named_buckets
            .insert("xrd_bucket".to_string(), ManifestBucket(0u32));
        assert_eq!(
            generate_instruction(
                &instruction,
                &mut id_validator,
                &mut resolver,
                &AddressBech32Decoder::new(&NetworkDefinition::simulator()),
                &MockBlobProvider::default()
            ),
            Ok(CallMethod {
                address: CONSENSUS_MANAGER.into(),
                method_name: CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(
                    &ConsensusManagerCreateValidatorManifestInput {
                        key: Secp256k1PrivateKey::from_u64(2u64).unwrap().public_key(),
                        fee_factor: Decimal::ONE,
                        xrd_payment: ManifestBucket(0u32)
                    }
                ),
            }
            .into())
        );
    }

    macro_rules! generate_manifest_input_with_given_depth {
        ( $depth:expr ) => {{
            let depth: usize = $depth;
            // check depth
            let mut manifest = r#"CALL_FUNCTION Address("package_sim1p4r4955skdjq9swg8s5jguvcjvyj7tsxct87a9z6sw76cdfd2jg3zk") "blueprint" "func" "#.to_string();
            for _ in 0..depth - 1 {
                manifest.push_str("Tuple(");
            }
            manifest.push_str("0u8");
            for _ in 0..depth - 1 {
                manifest.push_str(")");
            }
            manifest.push_str(";");
            manifest
        }};
    }

    macro_rules! generate_compiled_manifest_with_given_depth {
        ( $depth:expr ) => {{
            let manifest = generate_manifest_input_with_given_depth!($depth);
            let address_bech32_decoder = AddressBech32Decoder::new(&NetworkDefinition::simulator());
            let transaction_bech32_decoder =
                TransactionHashBech32Decoder::new(&NetworkDefinition::simulator());

            let tokens = tokenize(&manifest)
                .map_err(CompileError::LexerError)
                .unwrap();

            let instructions = parser::Parser::new(tokens, $depth)
                .unwrap()
                .parse_manifest()
                .unwrap();
            let blobs = BlobProvider::new();

            generate_manifest::<_, TransactionManifestV1>(
                &instructions,
                &address_bech32_decoder,
                &transaction_bech32_decoder,
                blobs,
            )
            .unwrap()
        }};
    }

    #[test]
    fn test_no_stack_overflow_for_very_deep_manifest() {
        use crate::manifest::*;

        let manifest = generate_manifest_input_with_given_depth!(1000);

        let result = compile_manifest_v1(
            &manifest,
            &NetworkDefinition::simulator(),
            BlobProvider::default(),
        );
        let expected = CompileError::ParserError(ParserError {
            error_kind: ParserErrorKind::MaxDepthExceeded {
                actual: 21,
                max: 20,
            },
            span: span!(start = (231, 0, 231), end = (236, 0, 236)),
        });

        match result {
            Ok(_) => {
                panic!("Expected {:?} but no error is thrown", expected);
            }
            Err(e) => {
                assert_eq!(e, expected);
            }
        }
    }

    #[test]
    fn test_if_max_depth_is_possibly_maximal() {
        use crate::manifest::*;
        // This test checks if PARSER_MAX_DEPTH is correctly adjusted in relation with
        // MANIFEST_SBOR_V1_MAX_DEPTH

        // When using manifest input with maximum depth we expect to
        // successfully encode manifest back from compiled one
        let compiled = generate_compiled_manifest_with_given_depth!(PARSER_MAX_DEPTH);

        let _result = manifest_encode(&compiled).unwrap();

        // When using manifest input maximum depth is exceeded by one we expect
        // encoding error when encoding the compiled one.
        let compiled = generate_compiled_manifest_with_given_depth!(PARSER_MAX_DEPTH + 1);

        let expected = EncodeError::MaxDepthExceeded(MANIFEST_SBOR_V1_MAX_DEPTH);

        let result = manifest_encode(&compiled);
        assert_eq!(result, Err(expected));
    }
}
