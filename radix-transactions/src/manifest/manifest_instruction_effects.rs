use crate::internal_prelude::*;

/// A nicer, grouped representation of a Transaction Instruction
#[derive(Debug, Clone, Copy)]
pub enum ManifestInstructionEffect<'a> {
    CreateBucket {
        source_amount: BucketSourceAmount<'a>,
    },
    CreateProof {
        source_amount: ProofSourceAmount<'a>,
    },
    ConsumeBucket {
        consumed_bucket: ManifestBucket,
        destination: BucketDestination<'a>,
    },
    ConsumeProof {
        consumed_proof: ManifestProof,
        destination: ProofDestination<'a>,
    },
    CloneProof {
        cloned_proof: ManifestProof,
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
    },
    ResourceAssertion {
        assertion: ResourceAssertion<'a>,
    },
    Verification {
        verification: VerificationKind,
        access_rule: &'a AccessRule,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum InvocationKind<'a> {
    Method {
        address: &'a ManifestGlobalAddress,
        module_id: ModuleId,
        method: &'a str,
    },
    Function {
        address: &'a ManifestPackageAddress,
        blueprint: &'a str,
        function: &'a str,
    },
    DirectMethod {
        address: &'a InternalAddress,
        method: &'a str,
    },
    YieldToParent,
    YieldToChild {
        child_index: ManifestNamedIntent,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum VerificationKind {
    Parent,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BucketSourceAmount<'a> {
    AllOnWorktop {
        resource_address: &'a ResourceAddress,
    },
    AmountFromWorktop {
        resource_address: &'a ResourceAddress,
        amount: Decimal,
    },
    NonFungiblesFromWorktop {
        resource_address: &'a ResourceAddress,
        ids: &'a [NonFungibleLocalId],
    },
}

impl<'a> BucketSourceAmount<'a> {
    pub fn resource_address(&self) -> &'a ResourceAddress {
        match self {
            Self::AllOnWorktop { resource_address }
            | Self::AmountFromWorktop {
                resource_address, ..
            }
            | Self::NonFungiblesFromWorktop {
                resource_address, ..
            } => resource_address,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy)]
pub enum BucketDestination<'a> {
    Worktop,
    Burned,
    Invocation(InvocationKind<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum ProofDestination<'a> {
    AuthZone,
    Drop,
    Invocation(InvocationKind<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum AddressReservationDestination<'a> {
    Invocation(InvocationKind<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum ExpressionDestination<'a> {
    Invocation(InvocationKind<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum BlobDestination<'a> {
    Invocation(InvocationKind<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceAssertion<'a> {
    Worktop(WorktopAssertion<'a>),
    NextCall(NextCallAssertion<'a>),
    Bucket(BucketAssertion<'a>),
}

#[derive(Debug, Clone, Copy)]
pub enum WorktopAssertion<'a> {
    ResourceNonZeroAmount {
        resource_address: &'a ResourceAddress,
    },
    ResourceAtLeastAmount {
        resource_address: &'a ResourceAddress,
        amount: Decimal,
    },
    ResourceAtLeastNonFungibles {
        resource_address: &'a ResourceAddress,
        ids: &'a [NonFungibleLocalId],
    },
    ResourcesOnly {
        constraints: &'a ManifestResourceConstraints,
    },
    ResourcesInclude {
        constraints: &'a ManifestResourceConstraints,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum NextCallAssertion<'a> {
    ReturnsOnly {
        constraints: &'a ManifestResourceConstraints,
    },
    ReturnsInclude {
        constraints: &'a ManifestResourceConstraints,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BucketAssertion<'a> {
    Contents {
        bucket: ManifestBucket,
        constraint: &'a ManifestResourceConstraint,
    },
}
