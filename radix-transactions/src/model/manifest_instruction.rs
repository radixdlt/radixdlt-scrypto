use crate::internal_prelude::*;
use decompiler::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::*;
use radix_engine_interface::object_modules::role_assignment::*;
use radix_engine_interface::object_modules::royalty::*;

pub trait ManifestInstruction {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError>;
    fn effect(&self) -> ManifestInstructionEffect;
}

pub enum InvocationKind<'a> {
    Method {
        address: &'a DynamicGlobalAddress,
        module_id: ModuleId,
        method: &'a str,
    },
    Function {
        address: &'a DynamicPackageAddress,
        blueprint: &'a str,
        function: &'a str,
    },
    DirectMethod {
        address: &'a InternalAddress,
        method: &'a str,
    },
    YieldToParent,
    YieldToChild {
        child_index: ManifestIntent,
    },
    VerifyParent,
}

pub enum BucketSourceAmount<'a> {
    AllOnWorktop,
    AmountFromWorktop(Decimal),
    NonFungiblesFromWorktop(&'a [NonFungibleLocalId]),
}

pub enum ProofSourceAmount<'a> {
    AuthZonePopLastAddedProof,
    AuthZoneAllOf {
        resource_address: &'a ResourceAddress,
    },
    AuthZoneAmount {
        resource_address: &'a ResourceAddress,
        amount: Decimal,
    },
    AuthZoneNonFungibles {
        resource_address: &'a ResourceAddress,
        ids: &'a [NonFungibleLocalId],
    },
    BucketAllOf {
        bucket: ManifestBucket,
    },
    BucketAmount {
        bucket: ManifestBucket,
        amount: Decimal,
    },
    BucketNonFungibles {
        bucket: ManifestBucket,
        ids: &'a [NonFungibleLocalId],
    },
}

impl<'a> ProofSourceAmount<'a> {
    pub fn proof_kind(&self) -> ProofKind {
        match self {
            ProofSourceAmount::AuthZonePopLastAddedProof
            | ProofSourceAmount::AuthZoneAllOf { .. }
            | ProofSourceAmount::AuthZoneAmount { .. }
            | ProofSourceAmount::AuthZoneNonFungibles { .. } => ProofKind::AuthZoneProof,
            ProofSourceAmount::BucketAllOf { bucket, .. }
            | ProofSourceAmount::BucketAmount { bucket, .. }
            | ProofSourceAmount::BucketNonFungibles { bucket, .. } => {
                ProofKind::BucketProof(*bucket)
            }
        }
    }
}

pub enum BucketDestination {
    Worktop,
    Burned,
}

pub enum ProofDestination {
    AuthZone,
    Drop,
}

pub enum ResourceAmountAssertion<'a> {
    AnyAmountGreaterThanZero,
    AtLeastAmount(Decimal),
    AtLeastNonFungibles(&'a [NonFungibleLocalId]),
}

/// The new_X are only included if the effect context includes an allocator
pub enum ManifestInstructionEffect<'a> {
    CreateBucket {
        resource: &'a ResourceAddress,
        source_amount: BucketSourceAmount<'a>,
        new_bucket: Option<ManifestBucket>,
    },
    CreateProof {
        source_amount: ProofSourceAmount<'a>,
        new_proof: Option<ManifestProof>,
    },
    ConsumeBucket {
        bucket: ManifestBucket,
        destination: BucketDestination,
    },
    ConsumeProof {
        proof: ManifestProof,
        destination: ProofDestination,
    },
    CloneProof {
        cloned_proof: ManifestProof,
        new_proof: Option<ManifestProof>,
    },
    DropManyProofs {
        drop_all_named_proofs: bool,
        drop_all_authzone_signature_proofs: bool,
        drop_all_authzone_non_signature_proofs: bool,
    },
    Invocation {
        kind: InvocationKind<'a>,
        args: &'a ManifestValue,
    },
    CreateAddressAndReservation {
        package_address: &'a PackageAddress,
        blueprint_name: &'a str,
        new_address_reservation: Option<ManifestAddressReservation>,
        new_named_address: Option<ManifestAddress>,
    },
    ResourceAssertion {
        resource_address: &'a ResourceAddress,
        amount: ResourceAmountAssertion<'a>,
    },
}

use ManifestInstructionEffect as Effect;

//======================================================================
// Worktop
//======================================================================

/// Takes a bucket containing the all of a given resource from the worktop,
/// and binds the given bucket name to that bucket.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct TakeAllFromWorktop {
    pub resource_address: ResourceAddress,
}

impl ManifestInstruction for TakeAllFromWorktop {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("TAKE_ALL_FROM_WORKTOP")
            .add_argument(&self.resource_address)
            .add_argument(context.new_bucket());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CreateBucket {
            resource: &self.resource_address,
            source_amount: BucketSourceAmount::AllOnWorktop,
            new_bucket: None,
        }
    }
}

/// Takes a bucket containing the given amount of resource from the worktop,
/// and binds the given bucket name to that bucket.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct TakeFromWorktop {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

impl ManifestInstruction for TakeFromWorktop {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("TAKE_FROM_WORKTOP")
            .add_argument(&self.resource_address)
            .add_argument(&self.amount)
            .add_argument(context.new_bucket());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CreateBucket {
            resource: &self.resource_address,
            source_amount: BucketSourceAmount::AmountFromWorktop(self.amount),
            new_bucket: None,
        }
    }
}

/// Takes a bucket containing the given non-fungible ids of the resource from the worktop,
/// and binds the given bucket name to that bucket.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct TakeNonFungiblesFromWorktop {
    pub resource_address: ResourceAddress,
    pub ids: Vec<NonFungibleLocalId>,
}

impl ManifestInstruction for TakeNonFungiblesFromWorktop {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("TAKE_NON_FUNGIBLES_FROM_WORKTOP")
            .add_argument(&self.resource_address)
            .add_argument(&self.ids)
            .add_argument(context.new_bucket());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CreateBucket {
            resource: &self.resource_address,
            source_amount: BucketSourceAmount::NonFungiblesFromWorktop(&self.ids),
            new_bucket: None,
        }
    }
}

/// Returns a bucket to the worktop.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct ReturnToWorktop {
    pub bucket_id: ManifestBucket,
}

impl ManifestInstruction for ReturnToWorktop {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction =
            DecompiledInstruction::new("RETURN_TO_WORKTOP").add_argument(&self.bucket_id);
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::ConsumeBucket {
            bucket: self.bucket_id,
            destination: BucketDestination::Worktop,
        }
    }
}

/// Asserts that the worktop contains any positive amount of the specified resource.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct AssertWorktopContainsAny {
    pub resource_address: ResourceAddress,
}

impl ManifestInstruction for AssertWorktopContainsAny {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("ASSERT_WORKTOP_CONTAINS_ANY")
            .add_argument(&self.resource_address);
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::ResourceAssertion {
            resource_address: &self.resource_address,
            amount: ResourceAmountAssertion::AnyAmountGreaterThanZero,
        }
    }
}

/// Asserts that the worktop contains at least the given amount of the specified resource.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct AssertWorktopContains {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

impl ManifestInstruction for AssertWorktopContains {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("ASSERT_WORKTOP_CONTAINS")
            .add_argument(&self.resource_address)
            .add_argument(&self.amount);
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::ResourceAssertion {
            resource_address: &self.resource_address,
            amount: ResourceAmountAssertion::AtLeastAmount(self.amount),
        }
    }
}

/// Asserts that the worktop contains at least the given non-fungible ids of the specified resource.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct AssertWorktopContainsNonFungibles {
    pub resource_address: ResourceAddress,
    pub ids: Vec<NonFungibleLocalId>,
}

impl ManifestInstruction for AssertWorktopContainsNonFungibles {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("ASSERT_WORKTOP_CONTAINS_NON_FUNGIBLES")
            .add_argument(&self.resource_address)
            .add_argument(&self.ids);
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::ResourceAssertion {
            resource_address: &self.resource_address,
            amount: ResourceAmountAssertion::AtLeastNonFungibles(&self.ids),
        }
    }
}

//======================================================================
// Auth zone
//======================================================================

/// Takes the last proof from the auth zone.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct PopFromAuthZone;

impl ManifestInstruction for PopFromAuthZone {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction =
            DecompiledInstruction::new("POP_FROM_AUTH_ZONE").add_argument(context.new_proof());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CreateProof {
            source_amount: ProofSourceAmount::AuthZonePopLastAddedProof,
            new_proof: None,
        }
    }
}

/// Adds a proof to the auth zone.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct PushToAuthZone {
    pub proof_id: ManifestProof,
}

impl ManifestInstruction for PushToAuthZone {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction =
            DecompiledInstruction::new("PUSH_TO_AUTHZONE").add_argument(&self.proof_id);
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::ConsumeProof {
            proof: self.proof_id,
            destination: ProofDestination::AuthZone,
        }
    }
}

/// Creates a proof of the given amount from the proofs available in the auth zone.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CreateProofFromAuthZoneOfAmount {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

impl ManifestInstruction for CreateProofFromAuthZoneOfAmount {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT")
            .add_argument(&self.resource_address)
            .add_argument(&self.amount)
            .add_argument(context.new_proof());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CreateProof {
            source_amount: ProofSourceAmount::AuthZoneAmount {
                resource_address: &self.resource_address,
                amount: self.amount,
            },
            new_proof: None,
        }
    }
}

/// Creates a proof of the given non-fungible ids from the proofs available in the auth zone.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CreateProofFromAuthZoneOfNonFungibles {
    pub resource_address: ResourceAddress,
    pub ids: Vec<NonFungibleLocalId>,
}

impl ManifestInstruction for CreateProofFromAuthZoneOfNonFungibles {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction =
            DecompiledInstruction::new("CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES")
                .add_argument(&self.resource_address)
                .add_argument(&self.ids)
                .add_argument(context.new_proof());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CreateProof {
            source_amount: ProofSourceAmount::AuthZoneNonFungibles {
                resource_address: &self.resource_address,
                ids: &self.ids,
            },
            new_proof: None,
        }
    }
}

/// Creates a proof of all available amount of the given resource from the proofs available in the auth zone.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CreateProofFromAuthZoneOfAll {
    pub resource_address: ResourceAddress,
}

impl ManifestInstruction for CreateProofFromAuthZoneOfAll {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL")
            .add_argument(&self.resource_address)
            .add_argument(context.new_proof());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CreateProof {
            source_amount: ProofSourceAmount::AuthZoneAllOf {
                resource_address: &self.resource_address,
            },
            new_proof: None,
        }
    }
}

/// Drops all the proofs in the auth zone, potentially freeing up the assets locked in any containers backing the proofs.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct DropAuthZoneProofs;

impl ManifestInstruction for DropAuthZoneProofs {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("DROP_AUTH_ZONE_PROOFS");
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::DropManyProofs {
            drop_all_named_proofs: false,
            drop_all_authzone_signature_proofs: true,
            drop_all_authzone_non_signature_proofs: true,
        }
    }
}

/// Drops all the non-signature proofs in the auth zone, potentially freeing up the assets locked in any containers backing the proofs.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct DropAuthZoneRegularProofs;

impl ManifestInstruction for DropAuthZoneRegularProofs {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("DROP_AUTH_ZONE_REGULAR_PROOFS");
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::DropManyProofs {
            drop_all_named_proofs: false,
            drop_all_authzone_signature_proofs: false,
            drop_all_authzone_non_signature_proofs: true,
        }
    }
}

/// Drops all the signature proofs in the auth zone, preventing any further calls from making use of signature-based authentication.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct DropAuthZoneSignatureProofs;

impl ManifestInstruction for DropAuthZoneSignatureProofs {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("DROP_AUTH_ZONE_SIGNATURE_PROOFS");
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::DropManyProofs {
            drop_all_named_proofs: false,
            drop_all_authzone_signature_proofs: true,
            drop_all_authzone_non_signature_proofs: false,
        }
    }
}

//======================================================================
// Named bucket
//======================================================================

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CreateProofFromBucketOfAmount {
    pub bucket_id: ManifestBucket,
    pub amount: Decimal,
}

impl ManifestInstruction for CreateProofFromBucketOfAmount {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("CREATE_PROOF_FROM_BUCKET_OF_AMOUNT")
            .add_argument(&self.bucket_id)
            .add_argument(&self.amount)
            .add_argument(context.new_proof());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CreateProof {
            source_amount: ProofSourceAmount::BucketAmount {
                bucket: self.bucket_id,
                amount: self.amount,
            },
            new_proof: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CreateProofFromBucketOfNonFungibles {
    pub bucket_id: ManifestBucket,
    pub ids: Vec<NonFungibleLocalId>,
}

impl ManifestInstruction for CreateProofFromBucketOfNonFungibles {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES")
            .add_argument(&self.bucket_id)
            .add_argument(&self.ids)
            .add_argument(context.new_proof());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CreateProof {
            source_amount: ProofSourceAmount::BucketNonFungibles {
                bucket: self.bucket_id,
                ids: &self.ids,
            },
            new_proof: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CreateProofFromBucketOfAll {
    pub bucket_id: ManifestBucket,
}

impl ManifestInstruction for CreateProofFromBucketOfAll {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("CREATE_PROOF_FROM_BUCKET_OF_ALL")
            .add_argument(&self.bucket_id)
            .add_argument(context.new_proof());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CreateProof {
            source_amount: ProofSourceAmount::BucketAllOf {
                bucket: self.bucket_id,
            },
            new_proof: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct BurnResource {
    pub bucket_id: ManifestBucket,
}

impl ManifestInstruction for BurnResource {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("BURN_RESOURCE").add_argument(&self.bucket_id);
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::ConsumeBucket {
            bucket: self.bucket_id,
            destination: BucketDestination::Burned,
        }
    }
}

//======================================================================
// Named proof
//======================================================================

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CloneProof {
    pub proof_id: ManifestProof,
}

impl ManifestInstruction for CloneProof {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("CLONE_PROOF")
            .add_argument(&self.proof_id)
            .add_argument(context.new_proof());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CloneProof {
            cloned_proof: self.proof_id,
            new_proof: None,
        }
    }
}

/// Drops a proof.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct DropProof {
    pub proof_id: ManifestProof,
}

impl ManifestInstruction for DropProof {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("DROP_PROOF").add_argument(&self.proof_id);
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::ConsumeProof {
            proof: self.proof_id,
            destination: ProofDestination::Drop,
        }
    }
}

//======================================================================
// Invocation
//======================================================================

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CallFunction {
    pub package_address: DynamicPackageAddress,
    pub blueprint_name: String,
    pub function_name: String,
    pub args: ManifestValue,
}

impl CallFunction {
    fn decompile_header(&self) -> DecompiledInstruction {
        if let DynamicPackageAddress::Static(package_address) = &self.package_address {
            match (
                package_address,
                self.blueprint_name.as_str(),
                self.function_name.as_str(),
            ) {
                (&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_WASM_IDENT) => {
                    return DecompiledInstruction::new("PUBLISH_PACKAGE");
                }
                (&PACKAGE_PACKAGE, PACKAGE_BLUEPRINT, PACKAGE_PUBLISH_WASM_ADVANCED_IDENT) => {
                    return DecompiledInstruction::new("PUBLISH_PACKAGE_ADVANCED");
                }
                (&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_ADVANCED_IDENT) => {
                    return DecompiledInstruction::new("CREATE_ACCOUNT_ADVANCED");
                }
                (&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_IDENT) => {
                    return DecompiledInstruction::new("CREATE_ACCOUNT");
                }
                (&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT, IDENTITY_CREATE_ADVANCED_IDENT) => {
                    return DecompiledInstruction::new("CREATE_IDENTITY_ADVANCED");
                }
                (&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT, IDENTITY_CREATE_IDENT) => {
                    return DecompiledInstruction::new("CREATE_IDENTITY");
                }
                (
                    &ACCESS_CONTROLLER_PACKAGE,
                    ACCESS_CONTROLLER_BLUEPRINT,
                    ACCESS_CONTROLLER_CREATE_IDENT,
                ) => {
                    return DecompiledInstruction::new("CREATE_ACCESS_CONTROLLER");
                }
                (
                    &RESOURCE_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                ) => {
                    return DecompiledInstruction::new("CREATE_FUNGIBLE_RESOURCE");
                }
                (
                    &RESOURCE_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                ) => {
                    return DecompiledInstruction::new(
                        "CREATE_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY",
                    );
                }
                (
                    &RESOURCE_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                ) => {
                    return DecompiledInstruction::new("CREATE_NON_FUNGIBLE_RESOURCE");
                }
                (
                    &RESOURCE_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                ) => {
                    return DecompiledInstruction::new(
                        "CREATE_NON_FUNGIBLE_RESOURCE_WITH_INITIAL_SUPPLY",
                    );
                }
                _ => {}
            }
        }
        DecompiledInstruction::new("CALL_FUNCTION")
            .add_argument(&self.package_address)
            .add_argument(&self.blueprint_name)
            .add_argument(&self.function_name)
    }
}

impl ManifestInstruction for CallFunction {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let mut instruction = self.decompile_header();

        if let Value::Tuple { fields: arg_fields } = &self.args {
            for argument in arg_fields.iter() {
                instruction = instruction.add_value_argument(argument.clone());
            }
        } else {
            return Err(DecompileError::InvalidArguments);
        }
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::Invocation {
            kind: InvocationKind::Function {
                address: &self.package_address,
                blueprint: &self.blueprint_name,
                function: &self.function_name,
            },
            args: &self.args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CallMethod {
    pub address: DynamicGlobalAddress,
    pub method_name: String,
    pub args: ManifestValue,
}

impl CallMethod {
    fn decompile_header(&self) -> DecompiledInstruction {
        if let DynamicGlobalAddress::Static(global_address) = &self.address {
            match (global_address.as_node_id(), self.method_name.as_str()) {
                (address, PACKAGE_CLAIM_ROYALTIES_IDENT) if address.is_global_package() => {
                    return DecompiledInstruction::new("CLAIM_PACKAGE_ROYALTIES")
                        .add_argument(global_address);
                }
                (address, FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT)
                    if address.is_global_fungible_resource_manager() =>
                {
                    return DecompiledInstruction::new("MINT_FUNGIBLE")
                        .add_argument(global_address);
                }
                (address, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT)
                    if address.is_global_non_fungible_resource_manager() =>
                {
                    return DecompiledInstruction::new("MINT_NON_FUNGIBLE")
                        .add_argument(global_address);
                }
                (address, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT)
                    if address.is_global_non_fungible_resource_manager() =>
                {
                    return DecompiledInstruction::new("MINT_RUID_NON_FUNGIBLE")
                        .add_argument(global_address);
                }
                (address, CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT)
                    if address.is_global_consensus_manager() =>
                {
                    return DecompiledInstruction::new("CREATE_VALIDATOR");
                }
                _ => {}
            }
        }
        DecompiledInstruction::new("CALL_METHOD")
            .add_argument(&self.address)
            .add_argument(&self.method_name)
    }
}

impl ManifestInstruction for CallMethod {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let mut instruction = self.decompile_header();

        if let Value::Tuple { fields: arg_fields } = &self.args {
            for argument in arg_fields.iter() {
                instruction = instruction.add_value_argument(argument.clone());
            }
        } else {
            return Err(DecompileError::InvalidArguments);
        }
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::Invocation {
            kind: InvocationKind::Method {
                address: &self.address,
                module_id: ModuleId::Main,
                method: &self.method_name,
            },
            args: &self.args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CallRoyaltyMethod {
    pub address: DynamicGlobalAddress,
    pub method_name: String,
    pub args: ManifestValue,
}

impl CallRoyaltyMethod {
    fn decompile_header(&self) -> DecompiledInstruction {
        match self.method_name.as_str() {
            COMPONENT_ROYALTY_SET_ROYALTY_IDENT => {
                return DecompiledInstruction::new("SET_COMPONENT_ROYALTY")
                    .add_argument(&self.address);
            }
            COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT => {
                return DecompiledInstruction::new("LOCK_COMPONENT_ROYALTY")
                    .add_argument(&self.address);
            }
            COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT => {
                return DecompiledInstruction::new("CLAIM_COMPONENT_ROYALTIES")
                    .add_argument(&self.address);
            }
            _ => {}
        }
        DecompiledInstruction::new("CALL_ROYALTY_METHOD")
            .add_argument(&self.address)
            .add_argument(&self.method_name)
    }
}

impl ManifestInstruction for CallRoyaltyMethod {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let mut instruction = self.decompile_header();

        if let Value::Tuple { fields: arg_fields } = &self.args {
            for argument in arg_fields.iter() {
                instruction = instruction.add_value_argument(argument.clone());
            }
        } else {
            return Err(DecompileError::InvalidArguments);
        }
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::Invocation {
            kind: InvocationKind::Method {
                address: &self.address,
                module_id: ModuleId::Royalty,
                method: &self.method_name,
            },
            args: &self.args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CallMetadataMethod {
    pub address: DynamicGlobalAddress,
    pub method_name: String,
    pub args: ManifestValue,
}

impl CallMetadataMethod {
    fn decompile_header(&self) -> DecompiledInstruction {
        match self.method_name.as_str() {
            METADATA_SET_IDENT => {
                return DecompiledInstruction::new("SET_METADATA").add_argument(&self.address);
            }
            METADATA_REMOVE_IDENT => {
                return DecompiledInstruction::new("REMOVE_METADATA").add_argument(&self.address);
            }
            METADATA_LOCK_IDENT => {
                return DecompiledInstruction::new("LOCK_METADATA").add_argument(&self.address);
            }
            _ => {}
        }
        DecompiledInstruction::new("CALL_METADATA_METHOD")
            .add_argument(&self.address)
            .add_argument(&self.method_name)
    }
}

impl ManifestInstruction for CallMetadataMethod {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let mut instruction = self.decompile_header();

        if let Value::Tuple { fields: arg_fields } = &self.args {
            for argument in arg_fields.iter() {
                instruction = instruction.add_value_argument(argument.clone());
            }
        } else {
            return Err(DecompileError::InvalidArguments);
        }
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::Invocation {
            kind: InvocationKind::Method {
                address: &self.address,
                module_id: ModuleId::Metadata,
                method: &self.method_name,
            },
            args: &self.args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CallRoleAssignmentMethod {
    pub address: DynamicGlobalAddress,
    pub method_name: String,
    pub args: ManifestValue,
}

impl CallRoleAssignmentMethod {
    fn decompile_header(&self) -> DecompiledInstruction {
        match self.method_name.as_str() {
            ROLE_ASSIGNMENT_SET_OWNER_IDENT => {
                return DecompiledInstruction::new("SET_OWNER_ROLE").add_argument(&self.address);
            }
            ROLE_ASSIGNMENT_LOCK_OWNER_IDENT => {
                return DecompiledInstruction::new("LOCK_OWNER_ROLE").add_argument(&self.address);
            }
            ROLE_ASSIGNMENT_SET_IDENT => {
                return DecompiledInstruction::new("SET_ROLE").add_argument(&self.address);
            }
            _ => {}
        }
        DecompiledInstruction::new("CALL_ROLE_ASSIGNMENT_METHOD")
            .add_argument(&self.address)
            .add_argument(&self.method_name)
    }
}

impl ManifestInstruction for CallRoleAssignmentMethod {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let mut instruction = self.decompile_header();

        if let Value::Tuple { fields: arg_fields } = &self.args {
            for argument in arg_fields.iter() {
                instruction = instruction.add_value_argument(argument.clone());
            }
        } else {
            return Err(DecompileError::InvalidArguments);
        }
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::Invocation {
            kind: InvocationKind::Method {
                address: &self.address,
                module_id: ModuleId::RoleAssignment,
                method: &self.method_name,
            },
            args: &self.args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct CallDirectVaultMethod {
    pub address: InternalAddress,
    pub method_name: String,
    pub args: ManifestValue,
}

impl CallDirectVaultMethod {
    fn decompile_header(&self) -> DecompiledInstruction {
        match self.method_name.as_str() {
            VAULT_RECALL_IDENT => {
                return DecompiledInstruction::new("RECALL_FROM_VAULT").add_argument(&self.address);
            }
            VAULT_FREEZE_IDENT => {
                return DecompiledInstruction::new("FREEZE_VAULT").add_argument(&self.address);
            }
            VAULT_UNFREEZE_IDENT => {
                return DecompiledInstruction::new("UNFREEZE_VAULT").add_argument(&self.address);
            }
            NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT => {
                return DecompiledInstruction::new("RECALL_NON_FUNGIBLES_FROM_VAULT")
                    .add_argument(&self.address);
            }
            _ => {}
        }
        DecompiledInstruction::new("CALL_DIRECT_VAULT_METHOD")
            .add_argument(&self.address)
            .add_argument(&self.method_name)
    }
}

impl ManifestInstruction for CallDirectVaultMethod {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let mut instruction = self.decompile_header();

        if let Value::Tuple { fields: arg_fields } = &self.args {
            for argument in arg_fields.iter() {
                instruction = instruction.add_value_argument(argument.clone());
            }
        } else {
            return Err(DecompileError::InvalidArguments);
        }
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::Invocation {
            kind: InvocationKind::DirectMethod {
                address: &self.address,
                method: &self.method_name,
            },
            args: &self.args,
        }
    }
}

//======================================================================
// Complex
//======================================================================

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct DropNamedProofs;

impl ManifestInstruction for DropNamedProofs {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("DROP_NAMED_PROOFS");
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::DropManyProofs {
            drop_all_named_proofs: true,
            drop_all_authzone_signature_proofs: false,
            drop_all_authzone_non_signature_proofs: false,
        }
    }
}

/// Drops all proofs, both named proofs and auth zone proofs.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct DropAllProofs;

impl ManifestInstruction for DropAllProofs {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("DROP_ALL_PROOFS");
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::DropManyProofs {
            drop_all_named_proofs: true,
            drop_all_authzone_signature_proofs: true,
            drop_all_authzone_non_signature_proofs: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct AllocateGlobalAddress {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
}

impl ManifestInstruction for AllocateGlobalAddress {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("ALLOCATE_GLOBAL_ADDRESS")
            .add_argument(&self.package_address)
            .add_argument(&self.blueprint_name)
            .add_argument(context.new_address_reservation())
            .add_argument(context.new_address());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::CreateAddressAndReservation {
            package_address: &self.package_address,
            blueprint_name: &self.blueprint_name,
            new_address_reservation: None,
            new_named_address: None,
        }
    }
}

//======================================================================
// Interactions with other intents
//======================================================================

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct YieldToParent {
    pub args: ManifestValue,
}

impl ManifestInstruction for YieldToParent {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction =
            DecompiledInstruction::new("YIELD_TO_PARENT").add_value_argument(self.args.clone());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::Invocation {
            kind: InvocationKind::YieldToParent,
            args: &self.args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct YieldToChild {
    pub child_index: ManifestIntent,
    pub args: ManifestValue,
}

impl ManifestInstruction for YieldToChild {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("YIELD_TO_CHILD")
            .add_argument(self.child_index)
            .add_value_argument(self.args.clone());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::Invocation {
            kind: InvocationKind::YieldToChild {
                child_index: self.child_index,
            },
            args: &self.args,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct VerifyParent {
    pub access_rule: ManifestValue,
}

impl ManifestInstruction for VerifyParent {
    fn decompile(
        &self,
        _context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        let instruction = DecompiledInstruction::new("AUTHENTICATE_PARENT")
            .add_value_argument(self.access_rule.clone());
        Ok(instruction)
    }

    fn effect(&self) -> Effect {
        Effect::Invocation {
            kind: InvocationKind::VerifyParent,
            args: &self.access_rule,
        }
    }
}

//===============================================================
// INSTRUCTION DISCRIMINATORS:
//
// These are separately saved in the ledger app. To avoid too much
// churn there:
//
// - Try to keep these constant when adding/removing instructions:
//   > For a new instruction, allocate a new number from the end
//   > If removing an instruction, leave a gap
// - Feel free to move the enum around to make logical groupings
//   though
//===============================================================

//==============
// Worktop
//==============
pub const INSTRUCTION_TAKE_FROM_WORKTOP_DISCRIMINATOR: u8 = 0x00;
pub const INSTRUCTION_TAKE_NON_FUNGIBLES_FROM_WORKTOP_DISCRIMINATOR: u8 = 0x01;
pub const INSTRUCTION_TAKE_ALL_FROM_WORKTOP_DISCRIMINATOR: u8 = 0x02;
pub const INSTRUCTION_RETURN_TO_WORKTOP_DISCRIMINATOR: u8 = 0x03;
pub const INSTRUCTION_ASSERT_WORKTOP_CONTAINS_DISCRIMINATOR: u8 = 0x04;
pub const INSTRUCTION_ASSERT_WORKTOP_CONTAINS_NON_FUNGIBLES_DISCRIMINATOR: u8 = 0x05;
pub const INSTRUCTION_ASSERT_WORKTOP_CONTAINS_ANY_DISCRIMINATOR: u8 = 0x06;

//==============
// Auth zone
//==============
pub const INSTRUCTION_POP_FROM_AUTH_ZONE_DISCRIMINATOR: u8 = 0x10;
pub const INSTRUCTION_PUSH_TO_AUTH_ZONE_DISCRIMINATOR: u8 = 0x11;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT_DISCRIMINATOR: u8 = 0x14;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES_DISCRIMINATOR: u8 = 0x15;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL_DISCRIMINATOR: u8 = 0x16;
pub const INSTRUCTION_DROP_AUTH_ZONE_PROOFS_DISCRIMINATOR: u8 = 0x12;
pub const INSTRUCTION_DROP_AUTH_ZONE_REGULAR_PROOFS_DISCRIMINATOR: u8 = 0x13;
pub const INSTRUCTION_DROP_AUTH_ZONE_SIGNATURE_PROOFS_DISCRIMINATOR: u8 = 0x17;

//==============
// Named bucket
//==============
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_AMOUNT_DISCRIMINATOR: u8 = 0x21;
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES_DISCRIMINATOR: u8 = 0x22;
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_ALL_DISCRIMINATOR: u8 = 0x23;
pub const INSTRUCTION_BURN_RESOURCE_DISCRIMINATOR: u8 = 0x24;

//==============
// Named proof
//==============
pub const INSTRUCTION_CLONE_PROOF_DISCRIMINATOR: u8 = 0x30;
pub const INSTRUCTION_DROP_PROOF_DISCRIMINATOR: u8 = 0x31;

//==============
// Invocation
//==============
pub const INSTRUCTION_CALL_FUNCTION_DISCRIMINATOR: u8 = 0x40;
pub const INSTRUCTION_CALL_METHOD_DISCRIMINATOR: u8 = 0x41;
pub const INSTRUCTION_CALL_ROYALTY_METHOD_DISCRIMINATOR: u8 = 0x42;
pub const INSTRUCTION_CALL_METADATA_METHOD_DISCRIMINATOR: u8 = 0x43;
pub const INSTRUCTION_CALL_ROLE_ASSIGNMENT_METHOD_DISCRIMINATOR: u8 = 0x44;
pub const INSTRUCTION_CALL_DIRECT_VAULT_METHOD_DISCRIMINATOR: u8 = 0x45;

//==============
// Complex
//==============
pub const INSTRUCTION_DROP_NAMED_PROOFS_DISCRIMINATOR: u8 = 0x52;
pub const INSTRUCTION_DROP_ALL_PROOFS_DISCRIMINATOR: u8 = 0x50;
pub const INSTRUCTION_ALLOCATE_GLOBAL_ADDRESS_DISCRIMINATOR: u8 = 0x51;

//==============
// Interactions with other intents
//==============
pub const INSTRUCTION_YIELD_TO_PARENT_DISCRIMINATOR: u8 = 0x60;
pub const INSTRUCTION_YIELD_TO_CHILD_DISCRIMINATOR: u8 = 0x61;
pub const INSTRUCTION_VERIFY_PARENT_DISCRIMINATOR: u8 = 0x62;
