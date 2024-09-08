use crate::internal_prelude::*;

/// A nicer, grouped representation of a Transaction Instruction
#[derive(Clone, Copy)]
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
    WorktopAssertion {
        assertion: WorktopAssertion<'a>,
    },
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy)]
pub enum BucketDestination<'a> {
    Worktop,
    Burned,
    Invocation(InvocationKind<'a>),
}

#[derive(Clone, Copy)]
pub enum ProofDestination<'a> {
    AuthZone,
    Drop,
    Invocation(InvocationKind<'a>),
}

#[derive(Clone, Copy)]
pub enum AddressReservationDestination<'a> {
    Invocation(InvocationKind<'a>),
}

#[derive(Clone, Copy)]
pub enum ExpressionDestination<'a> {
    Invocation(InvocationKind<'a>),
}

#[derive(Clone, Copy)]
pub enum BlobDestination<'a> {
    Invocation(InvocationKind<'a>),
}

#[derive(Clone, Copy)]
pub enum WorktopAssertion<'a> {
    AnyAmountGreaterThanZero {
        resource_address: &'a ResourceAddress,
    },
    AtLeastAmount {
        resource_address: &'a ResourceAddress,
        amount: Decimal,
    },
    AtLeastNonFungibles {
        resource_address: &'a ResourceAddress,
        ids: &'a [NonFungibleLocalId],
    },
    IsEmpty,
}

impl<'a> WorktopAssertion<'a> {
    pub fn resource_address(&self) -> &'a ResourceAddress {
        match self {
            Self::AnyAmountGreaterThanZero { resource_address }
            | Self::AtLeastAmount {
                resource_address, ..
            }
            | Self::AtLeastNonFungibles {
                resource_address, ..
            } => resource_address,
        }
    }
}
