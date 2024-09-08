use crate::internal_prelude::*;
use decompiler::*;

impl<T: SborEnumVariantFor<InstructionV2, ManifestCustomValueKind>> From<T> for InstructionV2 {
    fn from(value: T) -> Self {
        value.into_enum()
    }
}

impl ManifestInstruction for InstructionV2 {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        match self {
            Self::TakeAllFromWorktop(x) => x.decompile(context),
            Self::TakeFromWorktop(x) => x.decompile(context),
            Self::TakeNonFungiblesFromWorktop(x) => x.decompile(context),
            Self::ReturnToWorktop(x) => x.decompile(context),
            Self::AssertWorktopContainsAny(x) => x.decompile(context),
            Self::AssertWorktopContains(x) => x.decompile(context),
            Self::AssertWorktopContainsNonFungibles(x) => x.decompile(context),
            Self::PopFromAuthZone(x) => x.decompile(context),
            Self::PushToAuthZone(x) => x.decompile(context),
            Self::CreateProofFromAuthZoneOfAmount(x) => x.decompile(context),
            Self::CreateProofFromAuthZoneOfNonFungibles(x) => x.decompile(context),
            Self::CreateProofFromAuthZoneOfAll(x) => x.decompile(context),
            Self::DropAuthZoneProofs(x) => x.decompile(context),
            Self::DropAuthZoneRegularProofs(x) => x.decompile(context),
            Self::DropAuthZoneSignatureProofs(x) => x.decompile(context),
            Self::CreateProofFromBucketOfAmount(x) => x.decompile(context),
            Self::CreateProofFromBucketOfNonFungibles(x) => x.decompile(context),
            Self::CreateProofFromBucketOfAll(x) => x.decompile(context),
            Self::BurnResource(x) => x.decompile(context),
            Self::CloneProof(x) => x.decompile(context),
            Self::DropProof(x) => x.decompile(context),
            Self::CallFunction(x) => x.decompile(context),
            Self::CallMethod(x) => x.decompile(context),
            Self::CallRoyaltyMethod(x) => x.decompile(context),
            Self::CallMetadataMethod(x) => x.decompile(context),
            Self::CallRoleAssignmentMethod(x) => x.decompile(context),
            Self::CallDirectVaultMethod(x) => x.decompile(context),
            Self::DropNamedProofs(x) => x.decompile(context),
            Self::DropAllProofs(x) => x.decompile(context),
            Self::AllocateGlobalAddress(x) => x.decompile(context),
            Self::YieldToParent(x) => x.decompile(context),
            Self::YieldToChild(x) => x.decompile(context),
            Self::VerifyParent(x) => x.decompile(context),
        }
    }

    fn effect(&self) -> ManifestInstructionEffect {
        match self {
            Self::TakeAllFromWorktop(x) => x.effect(),
            Self::TakeFromWorktop(x) => x.effect(),
            Self::TakeNonFungiblesFromWorktop(x) => x.effect(),
            Self::ReturnToWorktop(x) => x.effect(),
            Self::AssertWorktopContainsAny(x) => x.effect(),
            Self::AssertWorktopContains(x) => x.effect(),
            Self::AssertWorktopContainsNonFungibles(x) => x.effect(),
            Self::PopFromAuthZone(x) => x.effect(),
            Self::PushToAuthZone(x) => x.effect(),
            Self::CreateProofFromAuthZoneOfAmount(x) => x.effect(),
            Self::CreateProofFromAuthZoneOfNonFungibles(x) => x.effect(),
            Self::CreateProofFromAuthZoneOfAll(x) => x.effect(),
            Self::DropAuthZoneProofs(x) => x.effect(),
            Self::DropAuthZoneRegularProofs(x) => x.effect(),
            Self::DropAuthZoneSignatureProofs(x) => x.effect(),
            Self::CreateProofFromBucketOfAmount(x) => x.effect(),
            Self::CreateProofFromBucketOfNonFungibles(x) => x.effect(),
            Self::CreateProofFromBucketOfAll(x) => x.effect(),
            Self::BurnResource(x) => x.effect(),
            Self::CloneProof(x) => x.effect(),
            Self::DropProof(x) => x.effect(),
            Self::CallFunction(x) => x.effect(),
            Self::CallMethod(x) => x.effect(),
            Self::CallRoyaltyMethod(x) => x.effect(),
            Self::CallMetadataMethod(x) => x.effect(),
            Self::CallRoleAssignmentMethod(x) => x.effect(),
            Self::CallDirectVaultMethod(x) => x.effect(),
            Self::DropNamedProofs(x) => x.effect(),
            Self::DropAllProofs(x) => x.effect(),
            Self::AllocateGlobalAddress(x) => x.effect(),
            Self::YieldToParent(x) => x.effect(),
            Self::YieldToChild(x) => x.effect(),
            Self::VerifyParent(x) => x.effect(),
        }
    }
}

impl From<InstructionV1> for InstructionV2 {
    fn from(value: InstructionV1) -> Self {
        match value {
            InstructionV1::TakeAllFromWorktop(x) => x.into(),
            InstructionV1::TakeFromWorktop(x) => x.into(),
            InstructionV1::TakeNonFungiblesFromWorktop(x) => x.into(),
            InstructionV1::ReturnToWorktop(x) => x.into(),
            InstructionV1::AssertWorktopContainsAny(x) => x.into(),
            InstructionV1::AssertWorktopContains(x) => x.into(),
            InstructionV1::AssertWorktopContainsNonFungibles(x) => x.into(),
            InstructionV1::PopFromAuthZone(x) => x.into(),
            InstructionV1::PushToAuthZone(x) => x.into(),
            InstructionV1::CreateProofFromAuthZoneOfAmount(x) => x.into(),
            InstructionV1::CreateProofFromAuthZoneOfNonFungibles(x) => x.into(),
            InstructionV1::CreateProofFromAuthZoneOfAll(x) => x.into(),
            InstructionV1::DropAuthZoneProofs(x) => x.into(),
            InstructionV1::DropAuthZoneRegularProofs(x) => x.into(),
            InstructionV1::DropAuthZoneSignatureProofs(x) => x.into(),
            InstructionV1::CreateProofFromBucketOfAmount(x) => x.into(),
            InstructionV1::CreateProofFromBucketOfNonFungibles(x) => x.into(),
            InstructionV1::CreateProofFromBucketOfAll(x) => x.into(),
            InstructionV1::BurnResource(x) => x.into(),
            InstructionV1::CloneProof(x) => x.into(),
            InstructionV1::DropProof(x) => x.into(),
            InstructionV1::CallFunction(x) => x.into(),
            InstructionV1::CallMethod(x) => x.into(),
            InstructionV1::CallRoyaltyMethod(x) => x.into(),
            InstructionV1::CallMetadataMethod(x) => x.into(),
            InstructionV1::CallRoleAssignmentMethod(x) => x.into(),
            InstructionV1::CallDirectVaultMethod(x) => x.into(),
            InstructionV1::DropNamedProofs(x) => x.into(),
            InstructionV1::DropAllProofs(x) => x.into(),
            InstructionV1::AllocateGlobalAddress(x) => x.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe, ScryptoSborAssertion)]
#[sbor(impl_variant_traits)]
#[sbor_assert(
    backwards_compatible(
        v1 = "FILE:../v1/instruction_v1_schema.txt",
        v2 = "FILE:instruction_v2_schema.txt",
    ),
    settings(
        comparison_between_versions = "EXPR: |s| s.allow_all_name_changes()",
        comparison_between_current_and_latest = "EXPR: |s| s",
    )
)]
pub enum InstructionV2 {
    //==============
    // Worktop
    //==============
    /// Takes resource from worktop.
    #[sbor(discriminator(INSTRUCTION_TAKE_ALL_FROM_WORKTOP_DISCRIMINATOR))]
    TakeAllFromWorktop(#[sbor(flatten)] TakeAllFromWorktop),

    /// Takes resource from worktop by the given amount.
    #[sbor(discriminator(INSTRUCTION_TAKE_FROM_WORKTOP_DISCRIMINATOR))]
    TakeFromWorktop(#[sbor(flatten)] TakeFromWorktop),

    /// Takes resource from worktop by the given non-fungible IDs.
    #[sbor(discriminator(INSTRUCTION_TAKE_NON_FUNGIBLES_FROM_WORKTOP_DISCRIMINATOR))]
    TakeNonFungiblesFromWorktop(#[sbor(flatten)] TakeNonFungiblesFromWorktop),

    /// Returns a bucket of resource to worktop.
    #[sbor(discriminator(INSTRUCTION_RETURN_TO_WORKTOP_DISCRIMINATOR))]
    ReturnToWorktop(#[sbor(flatten)] ReturnToWorktop),

    /// Asserts worktop contains any specified resource.
    #[sbor(discriminator(INSTRUCTION_ASSERT_WORKTOP_CONTAINS_ANY_DISCRIMINATOR))]
    AssertWorktopContainsAny(#[sbor(flatten)] AssertWorktopContainsAny),

    /// Asserts worktop contains resource by at least the given amount.
    #[sbor(discriminator(INSTRUCTION_ASSERT_WORKTOP_CONTAINS_DISCRIMINATOR))]
    AssertWorktopContains(#[sbor(flatten)] AssertWorktopContains),

    /// Asserts worktop contains resource by at least the given non-fungible IDs.
    #[sbor(discriminator(INSTRUCTION_ASSERT_WORKTOP_CONTAINS_NON_FUNGIBLES_DISCRIMINATOR))]
    AssertWorktopContainsNonFungibles(#[sbor(flatten)] AssertWorktopContainsNonFungibles),

    //==============
    // Auth zone
    //==============
    /// Takes the last proof from the auth zone.
    #[sbor(discriminator(INSTRUCTION_POP_FROM_AUTH_ZONE_DISCRIMINATOR))]
    PopFromAuthZone(#[sbor(flatten)] PopFromAuthZone),

    /// Adds a proof to the auth zone.
    #[sbor(discriminator(INSTRUCTION_PUSH_TO_AUTH_ZONE_DISCRIMINATOR))]
    PushToAuthZone(#[sbor(flatten)] PushToAuthZone),

    /// Creates a proof from the auth zone, by the given amount
    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT_DISCRIMINATOR))]
    CreateProofFromAuthZoneOfAmount(#[sbor(flatten)] CreateProofFromAuthZoneOfAmount),

    /// Creates a proof from the auth zone, by the given non-fungible IDs.
    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES_DISCRIMINATOR))]
    CreateProofFromAuthZoneOfNonFungibles(#[sbor(flatten)] CreateProofFromAuthZoneOfNonFungibles),

    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL_DISCRIMINATOR))]
    CreateProofFromAuthZoneOfAll(#[sbor(flatten)] CreateProofFromAuthZoneOfAll),

    #[sbor(discriminator(INSTRUCTION_DROP_AUTH_ZONE_PROOFS_DISCRIMINATOR))]
    DropAuthZoneProofs(#[sbor(flatten)] DropAuthZoneProofs),

    #[sbor(discriminator(INSTRUCTION_DROP_AUTH_ZONE_REGULAR_PROOFS_DISCRIMINATOR))]
    DropAuthZoneRegularProofs(#[sbor(flatten)] DropAuthZoneRegularProofs),

    #[sbor(discriminator(INSTRUCTION_DROP_AUTH_ZONE_SIGNATURE_PROOFS_DISCRIMINATOR))]
    DropAuthZoneSignatureProofs(#[sbor(flatten)] DropAuthZoneSignatureProofs),

    //==============
    // Named bucket
    //==============
    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_AMOUNT_DISCRIMINATOR))]
    CreateProofFromBucketOfAmount(#[sbor(flatten)] CreateProofFromBucketOfAmount),

    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES_DISCRIMINATOR))]
    CreateProofFromBucketOfNonFungibles(#[sbor(flatten)] CreateProofFromBucketOfNonFungibles),

    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_ALL_DISCRIMINATOR))]
    CreateProofFromBucketOfAll(#[sbor(flatten)] CreateProofFromBucketOfAll),

    #[sbor(discriminator(INSTRUCTION_BURN_RESOURCE_DISCRIMINATOR))]
    BurnResource(#[sbor(flatten)] BurnResource),

    //==============
    // Named proof
    //==============
    /// Clones a proof.
    #[sbor(discriminator(INSTRUCTION_CLONE_PROOF_DISCRIMINATOR))]
    CloneProof(#[sbor(flatten)] CloneProof),

    /// Drops a proof.
    #[sbor(discriminator(INSTRUCTION_DROP_PROOF_DISCRIMINATOR))]
    DropProof(#[sbor(flatten)] DropProof),

    //==============
    // Invocation
    //==============
    #[sbor(discriminator(INSTRUCTION_CALL_FUNCTION_DISCRIMINATOR))]
    CallFunction(#[sbor(flatten)] CallFunction),

    #[sbor(discriminator(INSTRUCTION_CALL_METHOD_DISCRIMINATOR))]
    CallMethod(#[sbor(flatten)] CallMethod),

    #[sbor(discriminator(INSTRUCTION_CALL_ROYALTY_METHOD_DISCRIMINATOR))]
    CallRoyaltyMethod(#[sbor(flatten)] CallRoyaltyMethod),

    #[sbor(discriminator(INSTRUCTION_CALL_METADATA_METHOD_DISCRIMINATOR))]
    CallMetadataMethod(#[sbor(flatten)] CallMetadataMethod),

    #[sbor(discriminator(INSTRUCTION_CALL_ROLE_ASSIGNMENT_METHOD_DISCRIMINATOR))]
    CallRoleAssignmentMethod(#[sbor(flatten)] CallRoleAssignmentMethod),

    #[sbor(discriminator(INSTRUCTION_CALL_DIRECT_VAULT_METHOD_DISCRIMINATOR))]
    CallDirectVaultMethod(#[sbor(flatten)] CallDirectVaultMethod),

    //==============
    // Complex
    //==============
    #[sbor(discriminator(INSTRUCTION_DROP_NAMED_PROOFS_DISCRIMINATOR))]
    DropNamedProofs(#[sbor(flatten)] DropNamedProofs),

    /// Drops all proofs, both named proofs and auth zone proofs.
    #[sbor(discriminator(INSTRUCTION_DROP_ALL_PROOFS_DISCRIMINATOR))]
    DropAllProofs(#[sbor(flatten)] DropAllProofs),

    #[sbor(discriminator(INSTRUCTION_ALLOCATE_GLOBAL_ADDRESS_DISCRIMINATOR))]
    AllocateGlobalAddress(#[sbor(flatten)] AllocateGlobalAddress),

    //==============
    // Interactions with other intents
    //==============
    #[sbor(discriminator(INSTRUCTION_YIELD_TO_PARENT_DISCRIMINATOR))]
    YieldToParent(#[sbor(flatten)] YieldToParent),

    #[sbor(discriminator(INSTRUCTION_YIELD_TO_CHILD_DISCRIMINATOR))]
    YieldToChild(#[sbor(flatten)] YieldToChild),

    #[sbor(discriminator(INSTRUCTION_VERIFY_PARENT_DISCRIMINATOR))]
    VerifyParent(#[sbor(flatten)] VerifyParent),
}