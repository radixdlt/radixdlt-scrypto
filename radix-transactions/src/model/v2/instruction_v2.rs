use crate::internal_prelude::*;

impl<T: SborEnumVariantFor<InstructionV2, ManifestCustomValueKind>> From<T> for InstructionV2 {
    fn from(value: T) -> Self {
        value.into_enum()
    }
}

impl ManifestInstructionSet for InstructionV2 {
    fn map_ref<M: InstructionRefMapper>(&self, mapper: M) -> M::Output<'_> {
        match self {
            InstructionV2::TakeFromWorktop(x) => mapper.apply(x),
            InstructionV2::TakeNonFungiblesFromWorktop(x) => mapper.apply(x),
            InstructionV2::TakeAllFromWorktop(x) => mapper.apply(x),
            InstructionV2::ReturnToWorktop(x) => mapper.apply(x),
            InstructionV2::BurnResource(x) => mapper.apply(x),
            InstructionV2::AssertWorktopContainsAny(x) => mapper.apply(x),
            InstructionV2::AssertWorktopContains(x) => mapper.apply(x),
            InstructionV2::AssertWorktopContainsNonFungibles(x) => mapper.apply(x),
            InstructionV2::AssertWorktopResourcesOnly(x) => mapper.apply(x),
            InstructionV2::AssertWorktopResourcesInclude(x) => mapper.apply(x),
            InstructionV2::AssertNextCallReturnsOnly(x) => mapper.apply(x),
            InstructionV2::AssertNextCallReturnsInclude(x) => mapper.apply(x),
            InstructionV2::AssertBucketContents(x) => mapper.apply(x),
            InstructionV2::CreateProofFromBucketOfAmount(x) => mapper.apply(x),
            InstructionV2::CreateProofFromBucketOfNonFungibles(x) => mapper.apply(x),
            InstructionV2::CreateProofFromBucketOfAll(x) => mapper.apply(x),
            InstructionV2::CreateProofFromAuthZoneOfAmount(x) => mapper.apply(x),
            InstructionV2::CreateProofFromAuthZoneOfNonFungibles(x) => mapper.apply(x),
            InstructionV2::CreateProofFromAuthZoneOfAll(x) => mapper.apply(x),
            InstructionV2::CloneProof(x) => mapper.apply(x),
            InstructionV2::DropProof(x) => mapper.apply(x),
            InstructionV2::PushToAuthZone(x) => mapper.apply(x),
            InstructionV2::PopFromAuthZone(x) => mapper.apply(x),
            InstructionV2::DropAuthZoneProofs(x) => mapper.apply(x),
            InstructionV2::DropAuthZoneRegularProofs(x) => mapper.apply(x),
            InstructionV2::DropAuthZoneSignatureProofs(x) => mapper.apply(x),
            InstructionV2::DropNamedProofs(x) => mapper.apply(x),
            InstructionV2::DropAllProofs(x) => mapper.apply(x),
            InstructionV2::CallFunction(x) => mapper.apply(x),
            InstructionV2::CallMethod(x) => mapper.apply(x),
            InstructionV2::CallRoyaltyMethod(x) => mapper.apply(x),
            InstructionV2::CallMetadataMethod(x) => mapper.apply(x),
            InstructionV2::CallRoleAssignmentMethod(x) => mapper.apply(x),
            InstructionV2::CallDirectVaultMethod(x) => mapper.apply(x),
            InstructionV2::AllocateGlobalAddress(x) => mapper.apply(x),
            InstructionV2::YieldToParent(x) => mapper.apply(x),
            InstructionV2::YieldToChild(x) => mapper.apply(x),
            InstructionV2::VerifyParent(x) => mapper.apply(x),
        }
    }

    fn map_self<M: OwnedInstructionMapper>(self, mapper: M) -> M::Output {
        match self {
            InstructionV2::TakeFromWorktop(x) => mapper.apply(x),
            InstructionV2::TakeNonFungiblesFromWorktop(x) => mapper.apply(x),
            InstructionV2::TakeAllFromWorktop(x) => mapper.apply(x),
            InstructionV2::ReturnToWorktop(x) => mapper.apply(x),
            InstructionV2::BurnResource(x) => mapper.apply(x),
            InstructionV2::AssertWorktopContainsAny(x) => mapper.apply(x),
            InstructionV2::AssertWorktopContains(x) => mapper.apply(x),
            InstructionV2::AssertWorktopContainsNonFungibles(x) => mapper.apply(x),
            InstructionV2::AssertWorktopResourcesOnly(x) => mapper.apply(x),
            InstructionV2::AssertWorktopResourcesInclude(x) => mapper.apply(x),
            InstructionV2::AssertNextCallReturnsOnly(x) => mapper.apply(x),
            InstructionV2::AssertNextCallReturnsInclude(x) => mapper.apply(x),
            InstructionV2::AssertBucketContents(x) => mapper.apply(x),
            InstructionV2::CreateProofFromBucketOfAmount(x) => mapper.apply(x),
            InstructionV2::CreateProofFromBucketOfNonFungibles(x) => mapper.apply(x),
            InstructionV2::CreateProofFromBucketOfAll(x) => mapper.apply(x),
            InstructionV2::CreateProofFromAuthZoneOfAmount(x) => mapper.apply(x),
            InstructionV2::CreateProofFromAuthZoneOfNonFungibles(x) => mapper.apply(x),
            InstructionV2::CreateProofFromAuthZoneOfAll(x) => mapper.apply(x),
            InstructionV2::CloneProof(x) => mapper.apply(x),
            InstructionV2::DropProof(x) => mapper.apply(x),
            InstructionV2::PushToAuthZone(x) => mapper.apply(x),
            InstructionV2::PopFromAuthZone(x) => mapper.apply(x),
            InstructionV2::DropAuthZoneProofs(x) => mapper.apply(x),
            InstructionV2::DropAuthZoneRegularProofs(x) => mapper.apply(x),
            InstructionV2::DropAuthZoneSignatureProofs(x) => mapper.apply(x),
            InstructionV2::DropNamedProofs(x) => mapper.apply(x),
            InstructionV2::DropAllProofs(x) => mapper.apply(x),
            InstructionV2::CallFunction(x) => mapper.apply(x),
            InstructionV2::CallMethod(x) => mapper.apply(x),
            InstructionV2::CallRoyaltyMethod(x) => mapper.apply(x),
            InstructionV2::CallMetadataMethod(x) => mapper.apply(x),
            InstructionV2::CallRoleAssignmentMethod(x) => mapper.apply(x),
            InstructionV2::CallDirectVaultMethod(x) => mapper.apply(x),
            InstructionV2::AllocateGlobalAddress(x) => mapper.apply(x),
            InstructionV2::YieldToParent(x) => mapper.apply(x),
            InstructionV2::YieldToChild(x) => mapper.apply(x),
            InstructionV2::VerifyParent(x) => mapper.apply(x),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe, ScryptoSborAssertion)]
#[sbor(impl_variant_traits)]
#[sbor_assert(
    backwards_compatible(
        v1 = "FILE:../v1/instruction_v1_schema.txt",
        v2 = "FILE:instruction_v2_schema.bin",
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

    #[sbor(discriminator(AssertWorktopResourcesOnly::ID))]
    AssertWorktopResourcesOnly(#[sbor(flatten)] AssertWorktopResourcesOnly),

    #[sbor(discriminator(AssertWorktopResourcesInclude::ID))]
    AssertWorktopResourcesInclude(#[sbor(flatten)] AssertWorktopResourcesInclude),

    #[sbor(discriminator(AssertNextCallReturnsOnly::ID))]
    AssertNextCallReturnsOnly(#[sbor(flatten)] AssertNextCallReturnsOnly),

    #[sbor(discriminator(AssertNextCallReturnsInclude::ID))]
    AssertNextCallReturnsInclude(#[sbor(flatten)] AssertNextCallReturnsInclude),

    #[sbor(discriminator(AssertBucketContents::ID))]
    AssertBucketContents(#[sbor(flatten)] AssertBucketContents),

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
    // Interaction with other intents
    //==============
    #[sbor(discriminator(YieldToParent::ID))]
    YieldToParent(#[sbor(flatten)] YieldToParent),

    #[sbor(discriminator(YieldToChild::ID))]
    YieldToChild(#[sbor(flatten)] YieldToChild),

    #[sbor(discriminator(VerifyParent::ID))]
    VerifyParent(#[sbor(flatten)] VerifyParent),
}
