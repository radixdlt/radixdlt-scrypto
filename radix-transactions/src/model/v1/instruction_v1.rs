use crate::internal_prelude::*;

impl<T: SborEnumVariantFor<InstructionV1, ManifestCustomValueKind>> From<T> for InstructionV1 {
    fn from(value: T) -> Self {
        value.into_enum()
    }
}

impl ManifestInstructionSet for InstructionV1 {
    fn decompile(
        &self,
        context: &mut decompiler::DecompilationContext,
    ) -> Result<decompiler::DecompiledInstruction, DecompileError> {
        match self {
            InstructionV1::TakeFromWorktop(x) => x.decompile(context),
            InstructionV1::TakeNonFungiblesFromWorktop(x) => x.decompile(context),
            InstructionV1::TakeAllFromWorktop(x) => x.decompile(context),
            InstructionV1::ReturnToWorktop(x) => x.decompile(context),
            InstructionV1::BurnResource(x) => x.decompile(context),
            InstructionV1::AssertWorktopContainsAny(x) => x.decompile(context),
            InstructionV1::AssertWorktopContains(x) => x.decompile(context),
            InstructionV1::AssertWorktopContainsNonFungibles(x) => x.decompile(context),
            InstructionV1::CreateProofFromBucketOfAmount(x) => x.decompile(context),
            InstructionV1::CreateProofFromBucketOfNonFungibles(x) => x.decompile(context),
            InstructionV1::CreateProofFromBucketOfAll(x) => x.decompile(context),
            InstructionV1::CreateProofFromAuthZoneOfAmount(x) => x.decompile(context),
            InstructionV1::CreateProofFromAuthZoneOfNonFungibles(x) => x.decompile(context),
            InstructionV1::CreateProofFromAuthZoneOfAll(x) => x.decompile(context),
            InstructionV1::CloneProof(x) => x.decompile(context),
            InstructionV1::DropProof(x) => x.decompile(context),
            InstructionV1::PushToAuthZone(x) => x.decompile(context),
            InstructionV1::PopFromAuthZone(x) => x.decompile(context),
            InstructionV1::DropAuthZoneProofs(x) => x.decompile(context),
            InstructionV1::DropAuthZoneRegularProofs(x) => x.decompile(context),
            InstructionV1::DropAuthZoneSignatureProofs(x) => x.decompile(context),
            InstructionV1::DropNamedProofs(x) => x.decompile(context),
            InstructionV1::DropAllProofs(x) => x.decompile(context),
            InstructionV1::CallFunction(x) => x.decompile(context),
            InstructionV1::CallMethod(x) => x.decompile(context),
            InstructionV1::CallRoyaltyMethod(x) => x.decompile(context),
            InstructionV1::CallMetadataMethod(x) => x.decompile(context),
            InstructionV1::CallRoleAssignmentMethod(x) => x.decompile(context),
            InstructionV1::CallDirectVaultMethod(x) => x.decompile(context),
            InstructionV1::AllocateGlobalAddress(x) => x.decompile(context),
        }
    }

    fn effect(&self) -> ManifestInstructionEffect {
        match self {
            InstructionV1::TakeFromWorktop(x) => x.effect(),
            InstructionV1::TakeNonFungiblesFromWorktop(x) => x.effect(),
            InstructionV1::TakeAllFromWorktop(x) => x.effect(),
            InstructionV1::ReturnToWorktop(x) => x.effect(),
            InstructionV1::BurnResource(x) => x.effect(),
            InstructionV1::AssertWorktopContainsAny(x) => x.effect(),
            InstructionV1::AssertWorktopContains(x) => x.effect(),
            InstructionV1::AssertWorktopContainsNonFungibles(x) => x.effect(),
            InstructionV1::CreateProofFromBucketOfAmount(x) => x.effect(),
            InstructionV1::CreateProofFromBucketOfNonFungibles(x) => x.effect(),
            InstructionV1::CreateProofFromBucketOfAll(x) => x.effect(),
            InstructionV1::CreateProofFromAuthZoneOfAmount(x) => x.effect(),
            InstructionV1::CreateProofFromAuthZoneOfNonFungibles(x) => x.effect(),
            InstructionV1::CreateProofFromAuthZoneOfAll(x) => x.effect(),
            InstructionV1::CloneProof(x) => x.effect(),
            InstructionV1::DropProof(x) => x.effect(),
            InstructionV1::PushToAuthZone(x) => x.effect(),
            InstructionV1::PopFromAuthZone(x) => x.effect(),
            InstructionV1::DropAuthZoneProofs(x) => x.effect(),
            InstructionV1::DropAuthZoneRegularProofs(x) => x.effect(),
            InstructionV1::DropAuthZoneSignatureProofs(x) => x.effect(),
            InstructionV1::DropNamedProofs(x) => x.effect(),
            InstructionV1::DropAllProofs(x) => x.effect(),
            InstructionV1::CallFunction(x) => x.effect(),
            InstructionV1::CallMethod(x) => x.effect(),
            InstructionV1::CallRoyaltyMethod(x) => x.effect(),
            InstructionV1::CallMetadataMethod(x) => x.effect(),
            InstructionV1::CallRoleAssignmentMethod(x) => x.effect(),
            InstructionV1::CallDirectVaultMethod(x) => x.effect(),
            InstructionV1::AllocateGlobalAddress(x) => x.effect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe, ScryptoSborAssertion)]
#[sbor(impl_variant_traits)]
#[sbor_assert(fixed("FILE:instruction_v1_schema.txt"))]
pub enum InstructionV1 {
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
}
