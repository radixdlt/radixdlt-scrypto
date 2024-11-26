use crate::internal_prelude::*;

impl<T: SborEnumVariantFor<InstructionV1, ManifestCustomValueKind>> From<T> for InstructionV1 {
    fn from(instruction: T) -> Self {
        instruction.into_enum()
    }
}

impl From<InstructionV1> for AnyInstruction {
    fn from(any_v1_instruction: InstructionV1) -> Self {
        any_v1_instruction.into_any()
    }
}

impl ManifestInstructionSet for InstructionV1 {
    fn map_ref<M: InstructionRefMapper>(&self, mapper: M) -> M::Output<'_> {
        match self {
            InstructionV1::TakeFromWorktop(x) => mapper.apply(x),
            InstructionV1::TakeNonFungiblesFromWorktop(x) => mapper.apply(x),
            InstructionV1::TakeAllFromWorktop(x) => mapper.apply(x),
            InstructionV1::ReturnToWorktop(x) => mapper.apply(x),
            InstructionV1::BurnResource(x) => mapper.apply(x),
            InstructionV1::AssertWorktopContainsAny(x) => mapper.apply(x),
            InstructionV1::AssertWorktopContains(x) => mapper.apply(x),
            InstructionV1::AssertWorktopContainsNonFungibles(x) => mapper.apply(x),
            InstructionV1::CreateProofFromBucketOfAmount(x) => mapper.apply(x),
            InstructionV1::CreateProofFromBucketOfNonFungibles(x) => mapper.apply(x),
            InstructionV1::CreateProofFromBucketOfAll(x) => mapper.apply(x),
            InstructionV1::CreateProofFromAuthZoneOfAmount(x) => mapper.apply(x),
            InstructionV1::CreateProofFromAuthZoneOfNonFungibles(x) => mapper.apply(x),
            InstructionV1::CreateProofFromAuthZoneOfAll(x) => mapper.apply(x),
            InstructionV1::CloneProof(x) => mapper.apply(x),
            InstructionV1::DropProof(x) => mapper.apply(x),
            InstructionV1::PushToAuthZone(x) => mapper.apply(x),
            InstructionV1::PopFromAuthZone(x) => mapper.apply(x),
            InstructionV1::DropAuthZoneProofs(x) => mapper.apply(x),
            InstructionV1::DropAuthZoneRegularProofs(x) => mapper.apply(x),
            InstructionV1::DropAuthZoneSignatureProofs(x) => mapper.apply(x),
            InstructionV1::DropNamedProofs(x) => mapper.apply(x),
            InstructionV1::DropAllProofs(x) => mapper.apply(x),
            InstructionV1::CallFunction(x) => mapper.apply(x),
            InstructionV1::CallMethod(x) => mapper.apply(x),
            InstructionV1::CallRoyaltyMethod(x) => mapper.apply(x),
            InstructionV1::CallMetadataMethod(x) => mapper.apply(x),
            InstructionV1::CallRoleAssignmentMethod(x) => mapper.apply(x),
            InstructionV1::CallDirectVaultMethod(x) => mapper.apply(x),
            InstructionV1::AllocateGlobalAddress(x) => mapper.apply(x),
        }
    }

    fn map_self<M: OwnedInstructionMapper>(self, mapper: M) -> M::Output {
        match self {
            InstructionV1::TakeFromWorktop(x) => mapper.apply(x),
            InstructionV1::TakeNonFungiblesFromWorktop(x) => mapper.apply(x),
            InstructionV1::TakeAllFromWorktop(x) => mapper.apply(x),
            InstructionV1::ReturnToWorktop(x) => mapper.apply(x),
            InstructionV1::BurnResource(x) => mapper.apply(x),
            InstructionV1::AssertWorktopContainsAny(x) => mapper.apply(x),
            InstructionV1::AssertWorktopContains(x) => mapper.apply(x),
            InstructionV1::AssertWorktopContainsNonFungibles(x) => mapper.apply(x),
            InstructionV1::CreateProofFromBucketOfAmount(x) => mapper.apply(x),
            InstructionV1::CreateProofFromBucketOfNonFungibles(x) => mapper.apply(x),
            InstructionV1::CreateProofFromBucketOfAll(x) => mapper.apply(x),
            InstructionV1::CreateProofFromAuthZoneOfAmount(x) => mapper.apply(x),
            InstructionV1::CreateProofFromAuthZoneOfNonFungibles(x) => mapper.apply(x),
            InstructionV1::CreateProofFromAuthZoneOfAll(x) => mapper.apply(x),
            InstructionV1::CloneProof(x) => mapper.apply(x),
            InstructionV1::DropProof(x) => mapper.apply(x),
            InstructionV1::PushToAuthZone(x) => mapper.apply(x),
            InstructionV1::PopFromAuthZone(x) => mapper.apply(x),
            InstructionV1::DropAuthZoneProofs(x) => mapper.apply(x),
            InstructionV1::DropAuthZoneRegularProofs(x) => mapper.apply(x),
            InstructionV1::DropAuthZoneSignatureProofs(x) => mapper.apply(x),
            InstructionV1::DropNamedProofs(x) => mapper.apply(x),
            InstructionV1::DropAllProofs(x) => mapper.apply(x),
            InstructionV1::CallFunction(x) => mapper.apply(x),
            InstructionV1::CallMethod(x) => mapper.apply(x),
            InstructionV1::CallRoyaltyMethod(x) => mapper.apply(x),
            InstructionV1::CallMetadataMethod(x) => mapper.apply(x),
            InstructionV1::CallRoleAssignmentMethod(x) => mapper.apply(x),
            InstructionV1::CallDirectVaultMethod(x) => mapper.apply(x),
            InstructionV1::AllocateGlobalAddress(x) => mapper.apply(x),
        }
    }
}

impl TryFrom<AnyInstruction> for InstructionV1 {
    type Error = ();

    fn try_from(value: AnyInstruction) -> Result<Self, Self::Error> {
        let mapped = match value {
            AnyInstruction::TakeFromWorktop(x) => x.into(),
            AnyInstruction::TakeNonFungiblesFromWorktop(x) => x.into(),
            AnyInstruction::TakeAllFromWorktop(x) => x.into(),
            AnyInstruction::ReturnToWorktop(x) => x.into(),
            AnyInstruction::BurnResource(x) => x.into(),
            AnyInstruction::AssertWorktopContainsAny(x) => x.into(),
            AnyInstruction::AssertWorktopContains(x) => x.into(),
            AnyInstruction::AssertWorktopContainsNonFungibles(x) => x.into(),
            AnyInstruction::AssertWorktopResourcesOnly(_) => return Err(()),
            AnyInstruction::AssertWorktopResourcesInclude(_) => return Err(()),
            AnyInstruction::AssertNextCallReturnsOnly(_) => return Err(()),
            AnyInstruction::AssertNextCallReturnsInclude(_) => return Err(()),
            AnyInstruction::AssertBucketContents(_) => return Err(()),
            AnyInstruction::CreateProofFromBucketOfAmount(x) => x.into(),
            AnyInstruction::CreateProofFromBucketOfNonFungibles(x) => x.into(),
            AnyInstruction::CreateProofFromBucketOfAll(x) => x.into(),
            AnyInstruction::CreateProofFromAuthZoneOfAmount(x) => x.into(),
            AnyInstruction::CreateProofFromAuthZoneOfNonFungibles(x) => x.into(),
            AnyInstruction::CreateProofFromAuthZoneOfAll(x) => x.into(),
            AnyInstruction::CloneProof(x) => x.into(),
            AnyInstruction::DropProof(x) => x.into(),
            AnyInstruction::PushToAuthZone(x) => x.into(),
            AnyInstruction::PopFromAuthZone(x) => x.into(),
            AnyInstruction::DropAuthZoneProofs(x) => x.into(),
            AnyInstruction::DropAuthZoneRegularProofs(x) => x.into(),
            AnyInstruction::DropAuthZoneSignatureProofs(x) => x.into(),
            AnyInstruction::DropNamedProofs(x) => x.into(),
            AnyInstruction::DropAllProofs(x) => x.into(),
            AnyInstruction::CallFunction(x) => x.into(),
            AnyInstruction::CallMethod(x) => x.into(),
            AnyInstruction::CallRoyaltyMethod(x) => x.into(),
            AnyInstruction::CallMetadataMethod(x) => x.into(),
            AnyInstruction::CallRoleAssignmentMethod(x) => x.into(),
            AnyInstruction::CallDirectVaultMethod(x) => x.into(),
            AnyInstruction::AllocateGlobalAddress(x) => x.into(),
            AnyInstruction::YieldToParent(_) => return Err(()),
            AnyInstruction::YieldToChild(_) => return Err(()),
            AnyInstruction::VerifyParent(_) => return Err(()),
        };
        Ok(mapped)
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
