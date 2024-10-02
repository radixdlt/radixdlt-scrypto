use crate::internal_prelude::*;
use decompiler::*;

impl<T: SborEnumVariantFor<InstructionV2, ManifestCustomValueKind>> From<T> for InstructionV2 {
    fn from(value: T) -> Self {
        value.into_enum()
    }
}

impl ManifestInstructionSet for InstructionV2 {
    fn decompile(
        &self,
        context: &mut DecompilationContext,
    ) -> Result<DecompiledInstruction, DecompileError> {
        match self {
            InstructionV2::TakeFromWorktop(x) => x.decompile(context),
            InstructionV2::TakeNonFungiblesFromWorktop(x) => x.decompile(context),
            InstructionV2::TakeAllFromWorktop(x) => x.decompile(context),
            InstructionV2::ReturnToWorktop(x) => x.decompile(context),
            InstructionV2::BurnResource(x) => x.decompile(context),
            InstructionV2::AssertWorktopContainsAny(x) => x.decompile(context),
            InstructionV2::AssertWorktopContains(x) => x.decompile(context),
            InstructionV2::AssertWorktopContainsNonFungibles(x) => x.decompile(context),
            InstructionV2::AssertWorktopIsEmpty(x) => x.decompile(context),
            InstructionV2::CreateProofFromBucketOfAmount(x) => x.decompile(context),
            InstructionV2::CreateProofFromBucketOfNonFungibles(x) => x.decompile(context),
            InstructionV2::CreateProofFromBucketOfAll(x) => x.decompile(context),
            InstructionV2::CreateProofFromAuthZoneOfAmount(x) => x.decompile(context),
            InstructionV2::CreateProofFromAuthZoneOfNonFungibles(x) => x.decompile(context),
            InstructionV2::CreateProofFromAuthZoneOfAll(x) => x.decompile(context),
            InstructionV2::CloneProof(x) => x.decompile(context),
            InstructionV2::DropProof(x) => x.decompile(context),
            InstructionV2::PushToAuthZone(x) => x.decompile(context),
            InstructionV2::PopFromAuthZone(x) => x.decompile(context),
            InstructionV2::DropAuthZoneProofs(x) => x.decompile(context),
            InstructionV2::DropAuthZoneRegularProofs(x) => x.decompile(context),
            InstructionV2::DropAuthZoneSignatureProofs(x) => x.decompile(context),
            InstructionV2::DropNamedProofs(x) => x.decompile(context),
            InstructionV2::DropAllProofs(x) => x.decompile(context),
            InstructionV2::CallFunction(x) => x.decompile(context),
            InstructionV2::CallMethod(x) => x.decompile(context),
            InstructionV2::CallRoyaltyMethod(x) => x.decompile(context),
            InstructionV2::CallMetadataMethod(x) => x.decompile(context),
            InstructionV2::CallRoleAssignmentMethod(x) => x.decompile(context),
            InstructionV2::CallDirectVaultMethod(x) => x.decompile(context),
            InstructionV2::AllocateGlobalAddress(x) => x.decompile(context),
            InstructionV2::YieldToParent(x) => x.decompile(context),
            InstructionV2::YieldToChild(x) => x.decompile(context),
            InstructionV2::VerifyParent(x) => x.decompile(context),
        }
    }

    fn effect(&self) -> ManifestInstructionEffect {
        match self {
            InstructionV2::TakeFromWorktop(x) => x.effect(),
            InstructionV2::TakeNonFungiblesFromWorktop(x) => x.effect(),
            InstructionV2::TakeAllFromWorktop(x) => x.effect(),
            InstructionV2::ReturnToWorktop(x) => x.effect(),
            InstructionV2::BurnResource(x) => x.effect(),
            InstructionV2::AssertWorktopContainsAny(x) => x.effect(),
            InstructionV2::AssertWorktopContains(x) => x.effect(),
            InstructionV2::AssertWorktopContainsNonFungibles(x) => x.effect(),
            InstructionV2::AssertWorktopIsEmpty(x) => x.effect(),
            InstructionV2::CreateProofFromBucketOfAmount(x) => x.effect(),
            InstructionV2::CreateProofFromBucketOfNonFungibles(x) => x.effect(),
            InstructionV2::CreateProofFromBucketOfAll(x) => x.effect(),
            InstructionV2::CreateProofFromAuthZoneOfAmount(x) => x.effect(),
            InstructionV2::CreateProofFromAuthZoneOfNonFungibles(x) => x.effect(),
            InstructionV2::CreateProofFromAuthZoneOfAll(x) => x.effect(),
            InstructionV2::CloneProof(x) => x.effect(),
            InstructionV2::DropProof(x) => x.effect(),
            InstructionV2::PushToAuthZone(x) => x.effect(),
            InstructionV2::PopFromAuthZone(x) => x.effect(),
            InstructionV2::DropAuthZoneProofs(x) => x.effect(),
            InstructionV2::DropAuthZoneRegularProofs(x) => x.effect(),
            InstructionV2::DropAuthZoneSignatureProofs(x) => x.effect(),
            InstructionV2::DropNamedProofs(x) => x.effect(),
            InstructionV2::DropAllProofs(x) => x.effect(),
            InstructionV2::CallFunction(x) => x.effect(),
            InstructionV2::CallMethod(x) => x.effect(),
            InstructionV2::CallRoyaltyMethod(x) => x.effect(),
            InstructionV2::CallMetadataMethod(x) => x.effect(),
            InstructionV2::CallRoleAssignmentMethod(x) => x.effect(),
            InstructionV2::CallDirectVaultMethod(x) => x.effect(),
            InstructionV2::AllocateGlobalAddress(x) => x.effect(),
            InstructionV2::YieldToParent(x) => x.effect(),
            InstructionV2::YieldToChild(x) => x.effect(),
            InstructionV2::VerifyParent(x) => x.effect(),
        }
    }
}

impl From<InstructionV1> for InstructionV2 {
    fn from(value: InstructionV1) -> Self {
        match value {
            InstructionV1::TakeFromWorktop(x) => x.into(),
            InstructionV1::TakeNonFungiblesFromWorktop(x) => x.into(),
            InstructionV1::TakeAllFromWorktop(x) => x.into(),
            InstructionV1::ReturnToWorktop(x) => x.into(),
            InstructionV1::BurnResource(x) => x.into(),
            InstructionV1::AssertWorktopContainsAny(x) => x.into(),
            InstructionV1::AssertWorktopContains(x) => x.into(),
            InstructionV1::AssertWorktopContainsNonFungibles(x) => x.into(),
            InstructionV1::CreateProofFromBucketOfAmount(x) => x.into(),
            InstructionV1::CreateProofFromBucketOfNonFungibles(x) => x.into(),
            InstructionV1::CreateProofFromBucketOfAll(x) => x.into(),
            InstructionV1::CreateProofFromAuthZoneOfAmount(x) => x.into(),
            InstructionV1::CreateProofFromAuthZoneOfNonFungibles(x) => x.into(),
            InstructionV1::CreateProofFromAuthZoneOfAll(x) => x.into(),
            InstructionV1::CloneProof(x) => x.into(),
            InstructionV1::DropProof(x) => x.into(),
            InstructionV1::PushToAuthZone(x) => x.into(),
            InstructionV1::PopFromAuthZone(x) => x.into(),
            InstructionV1::DropAuthZoneProofs(x) => x.into(),
            InstructionV1::DropAuthZoneRegularProofs(x) => x.into(),
            InstructionV1::DropAuthZoneSignatureProofs(x) => x.into(),
            InstructionV1::DropNamedProofs(x) => x.into(),
            InstructionV1::DropAllProofs(x) => x.into(),
            InstructionV1::CallFunction(x) => x.into(),
            InstructionV1::CallMethod(x) => x.into(),
            InstructionV1::CallRoyaltyMethod(x) => x.into(),
            InstructionV1::CallMetadataMethod(x) => x.into(),
            InstructionV1::CallRoleAssignmentMethod(x) => x.into(),
            InstructionV1::CallDirectVaultMethod(x) => x.into(),
            InstructionV1::AllocateGlobalAddress(x) => x.into(),
        }
    }
}

impl TryFrom<InstructionV2> for InstructionV1 {
    type Error = ();

    fn try_from(value: InstructionV2) -> Result<Self, Self::Error> {
        let mapped = match value {
            InstructionV2::TakeFromWorktop(x) => x.into(),
            InstructionV2::TakeNonFungiblesFromWorktop(x) => x.into(),
            InstructionV2::TakeAllFromWorktop(x) => x.into(),
            InstructionV2::ReturnToWorktop(x) => x.into(),
            InstructionV2::BurnResource(x) => x.into(),
            InstructionV2::AssertWorktopContainsAny(x) => x.into(),
            InstructionV2::AssertWorktopContains(x) => x.into(),
            InstructionV2::AssertWorktopContainsNonFungibles(x) => x.into(),
            InstructionV2::AssertWorktopIsEmpty(_) => return Err(()),
            InstructionV2::CreateProofFromBucketOfAmount(x) => x.into(),
            InstructionV2::CreateProofFromBucketOfNonFungibles(x) => x.into(),
            InstructionV2::CreateProofFromBucketOfAll(x) => x.into(),
            InstructionV2::CreateProofFromAuthZoneOfAmount(x) => x.into(),
            InstructionV2::CreateProofFromAuthZoneOfNonFungibles(x) => x.into(),
            InstructionV2::CreateProofFromAuthZoneOfAll(x) => x.into(),
            InstructionV2::CloneProof(x) => x.into(),
            InstructionV2::DropProof(x) => x.into(),
            InstructionV2::PushToAuthZone(x) => x.into(),
            InstructionV2::PopFromAuthZone(x) => x.into(),
            InstructionV2::DropAuthZoneProofs(x) => x.into(),
            InstructionV2::DropAuthZoneRegularProofs(x) => x.into(),
            InstructionV2::DropAuthZoneSignatureProofs(x) => x.into(),
            InstructionV2::DropNamedProofs(x) => x.into(),
            InstructionV2::DropAllProofs(x) => x.into(),
            InstructionV2::CallFunction(x) => x.into(),
            InstructionV2::CallMethod(x) => x.into(),
            InstructionV2::CallRoyaltyMethod(x) => x.into(),
            InstructionV2::CallMetadataMethod(x) => x.into(),
            InstructionV2::CallRoleAssignmentMethod(x) => x.into(),
            InstructionV2::CallDirectVaultMethod(x) => x.into(),
            InstructionV2::AllocateGlobalAddress(x) => x.into(),
            InstructionV2::YieldToParent(_) => return Err(()),
            InstructionV2::YieldToChild(_) => return Err(()),
            InstructionV2::VerifyParent(_) => return Err(()),
        };
        Ok(mapped)
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
    //===============
    // Bucket Lifecycle
    //===============
    #[sbor(discriminator(TakeFromWorktop::ID))]
    TakeFromWorktop(#[sbor(flatten)] TakeFromWorktop),

    #[sbor(discriminator(TakeNonFungiblesFromWorktop::ID))]
    TakeNonFungiblesFromWorktop(#[sbor(flatten)] TakeNonFungiblesFromWorktop),

    #[sbor(discriminator(TakeAllFromWorktop::ID))]
    TakeAllFromWorktop(#[sbor(flatten)] TakeAllFromWorktop),

    #[sbor(discriminator(ReturnToWorktop::ID))]
    ReturnToWorktop(#[sbor(flatten)] ReturnToWorktop),

    #[sbor(discriminator(BurnResource::ID))]
    BurnResource(#[sbor(flatten)] BurnResource),

    //==============
    // Resource Assertions
    //==============
    #[sbor(discriminator(AssertWorktopContainsAny::ID))]
    AssertWorktopContainsAny(#[sbor(flatten)] AssertWorktopContainsAny),

    #[sbor(discriminator(AssertWorktopContains::ID))]
    AssertWorktopContains(#[sbor(flatten)] AssertWorktopContains),

    #[sbor(discriminator(AssertWorktopContainsNonFungibles::ID))]
    AssertWorktopContainsNonFungibles(#[sbor(flatten)] AssertWorktopContainsNonFungibles),

    #[sbor(discriminator(AssertWorktopIsEmpty::ID))]
    AssertWorktopIsEmpty(#[sbor(flatten)] AssertWorktopIsEmpty),

    //==============
    // Proof Lifecycle
    //==============
    #[sbor(discriminator(CreateProofFromBucketOfAmount::ID))]
    CreateProofFromBucketOfAmount(#[sbor(flatten)] CreateProofFromBucketOfAmount),

    #[sbor(discriminator(CreateProofFromBucketOfNonFungibles::ID))]
    CreateProofFromBucketOfNonFungibles(#[sbor(flatten)] CreateProofFromBucketOfNonFungibles),

    #[sbor(discriminator(CreateProofFromBucketOfAll::ID))]
    CreateProofFromBucketOfAll(#[sbor(flatten)] CreateProofFromBucketOfAll),

    #[sbor(discriminator(CreateProofFromAuthZoneOfAmount::ID))]
    CreateProofFromAuthZoneOfAmount(#[sbor(flatten)] CreateProofFromAuthZoneOfAmount),

    #[sbor(discriminator(CreateProofFromAuthZoneOfNonFungibles::ID))]
    CreateProofFromAuthZoneOfNonFungibles(#[sbor(flatten)] CreateProofFromAuthZoneOfNonFungibles),

    #[sbor(discriminator(CreateProofFromAuthZoneOfAll::ID))]
    CreateProofFromAuthZoneOfAll(#[sbor(flatten)] CreateProofFromAuthZoneOfAll),

    #[sbor(discriminator(CloneProof::ID))]
    CloneProof(#[sbor(flatten)] CloneProof),

    #[sbor(discriminator(DropProof::ID))]
    DropProof(#[sbor(flatten)] DropProof),

    #[sbor(discriminator(PushToAuthZone::ID))]
    PushToAuthZone(#[sbor(flatten)] PushToAuthZone),

    #[sbor(discriminator(PopFromAuthZone::ID))]
    PopFromAuthZone(#[sbor(flatten)] PopFromAuthZone),

    #[sbor(discriminator(DropAuthZoneProofs::ID))]
    DropAuthZoneProofs(#[sbor(flatten)] DropAuthZoneProofs),

    #[sbor(discriminator(DropAuthZoneRegularProofs::ID))]
    DropAuthZoneRegularProofs(#[sbor(flatten)] DropAuthZoneRegularProofs),

    #[sbor(discriminator(DropAuthZoneSignatureProofs::ID))]
    DropAuthZoneSignatureProofs(#[sbor(flatten)] DropAuthZoneSignatureProofs),

    #[sbor(discriminator(DropNamedProofs::ID))]
    DropNamedProofs(#[sbor(flatten)] DropNamedProofs),

    #[sbor(discriminator(DropAllProofs::ID))]
    DropAllProofs(#[sbor(flatten)] DropAllProofs),

    //==============
    // Invocation
    //==============
    #[sbor(discriminator(CallFunction::ID))]
    CallFunction(#[sbor(flatten)] CallFunction),

    #[sbor(discriminator(CallMethod::ID))]
    CallMethod(#[sbor(flatten)] CallMethod),

    #[sbor(discriminator(CallRoyaltyMethod::ID))]
    CallRoyaltyMethod(#[sbor(flatten)] CallRoyaltyMethod),

    #[sbor(discriminator(CallMetadataMethod::ID))]
    CallMetadataMethod(#[sbor(flatten)] CallMetadataMethod),

    #[sbor(discriminator(CallRoleAssignmentMethod::ID))]
    CallRoleAssignmentMethod(#[sbor(flatten)] CallRoleAssignmentMethod),

    #[sbor(discriminator(CallDirectVaultMethod::ID))]
    CallDirectVaultMethod(#[sbor(flatten)] CallDirectVaultMethod),

    //==============
    // Address Allocation
    //==============
    #[sbor(discriminator(AllocateGlobalAddress::ID))]
    AllocateGlobalAddress(#[sbor(flatten)] AllocateGlobalAddress),

    //==============
    // Interactions with other intents
    //==============
    #[sbor(discriminator(YieldToParent::ID))]
    YieldToParent(#[sbor(flatten)] YieldToParent),

    #[sbor(discriminator(YieldToChild::ID))]
    YieldToChild(#[sbor(flatten)] YieldToChild),

    #[sbor(discriminator(VerifyParent::ID))]
    VerifyParent(#[sbor(flatten)] VerifyParent),
}
