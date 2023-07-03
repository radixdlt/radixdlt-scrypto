use crate::prelude::*;
use radix_engine_common::data::scrypto::model::*;
use radix_engine_interface::data::manifest::{model::*, ManifestValue};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use radix_engine_interface::*;

/*
=================================================================================
NOTE: For now, we only support "dynamic" addresses for CALL instructions.
=================================================================================
This is to reduce the scope of change and make manifest easier to reason about.

In theory, we can apply it to all types of global addresses (`GlobalAddress`,
`PackageAddress`, `ResourceAddress` and `ComponentAddress`).

Then, we can do more advanced stuff in manifest, such as
```
ALLOCATE_GLOBAL_ADDRESS
    Address("{resource_package}")
    "FungibleResourceManager"
    AddressReservation("address_reservation")
    NamedAddress("new_resource")
;
CALL_FUNCTION
    Address("{resource_package}")
    "FungibleResourceManager"
    "create_with_initial_supply_and_address"
    Decimal("10")
    AddressReservation("address_reservation")
;
TAKE_FROM_WORKTOP
    NamedAddress("new_resource")
    Decimal("5.0")
    Bucket("bucket1")
;
TAKE_FROM_WORKTOP
    NamedAddress("new_resource")
    Decimal("5.0")
    Bucket("bucket2")
;
```
*/

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor)]
pub enum DynamicGlobalAddress {
    Static(GlobalAddress),
    Named(u32),
}

impl DynamicGlobalAddress {
    /// This is to support either `Address("static_address")` or `NamedAddress("abc")` in manifest instruction,
    /// instead of `Enum<0u8>(Address("static_address"))`.
    pub fn to_instruction_argument(&self) -> ManifestValue {
        match self {
            Self::Static(address) => ManifestValue::Custom {
                value: ManifestCustomValue::Address(ManifestAddress::Static(
                    address.into_node_id(),
                )),
            },
            Self::Named(id) => ManifestValue::Custom {
                value: ManifestCustomValue::Address(ManifestAddress::Named(*id)),
            },
        }
    }

    pub fn is_static_global_package(&self) -> bool {
        match self {
            Self::Static(address) => address.as_node_id().is_global_package(),
            Self::Named(_) => false,
        }
    }

    pub fn is_static_global_fungible_resource_manager(&self) -> bool {
        match self {
            Self::Static(address) => address.as_node_id().is_global_fungible_resource_manager(),
            Self::Named(_) => false,
        }
    }
    pub fn is_static_global_non_fungible_resource_manager(&self) -> bool {
        match self {
            Self::Static(address) => address
                .as_node_id()
                .is_global_non_fungible_resource_manager(),
            Self::Named(_) => false,
        }
    }
}

impl From<GlobalAddress> for DynamicGlobalAddress {
    fn from(value: GlobalAddress) -> Self {
        Self::Static(value)
    }
}

impl From<PackageAddress> for DynamicGlobalAddress {
    fn from(value: PackageAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<ResourceAddress> for DynamicGlobalAddress {
    fn from(value: ResourceAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<ComponentAddress> for DynamicGlobalAddress {
    fn from(value: ComponentAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<u32> for DynamicGlobalAddress {
    fn from(value: u32) -> Self {
        Self::Named(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor)]
pub enum DynamicPackageAddress {
    Static(PackageAddress),
    Named(u32),
}

impl DynamicPackageAddress {
    /// This is to support either `Address("static_address")` or `NamedAddress("abc")` in manifest instruction,
    /// instead of `Enum<0u8>(Address("static_address"))`.
    pub fn to_instruction_argument(&self) -> ManifestValue {
        match self {
            Self::Static(address) => ManifestValue::Custom {
                value: ManifestCustomValue::Address(ManifestAddress::Static(
                    address.into_node_id(),
                )),
            },
            Self::Named(id) => ManifestValue::Custom {
                value: ManifestCustomValue::Address(ManifestAddress::Named(*id)),
            },
        }
    }

    pub fn is_static_global_package_of(&self, package_address: &PackageAddress) -> bool {
        match self {
            Self::Static(address) => address.as_node_id().eq(package_address.as_node_id()),
            Self::Named(_) => false,
        }
    }
}

impl From<PackageAddress> for DynamicPackageAddress {
    fn from(value: PackageAddress) -> Self {
        Self::Static(value.into())
    }
}

impl From<u32> for DynamicPackageAddress {
    fn from(value: u32) -> Self {
        Self::Named(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor)]
pub enum InstructionV1 {
    //==============
    // Worktop
    //==============
    /// Takes resource from worktop.
    #[sbor(discriminator(INSTRUCTION_TAKE_ALL_FROM_WORKTOP_DISCRIMINATOR))]
    TakeAllFromWorktop { resource_address: ResourceAddress },

    /// Takes resource from worktop by the given amount.
    #[sbor(discriminator(INSTRUCTION_TAKE_FROM_WORKTOP_DISCRIMINATOR))]
    TakeFromWorktop {
        resource_address: ResourceAddress,
        amount: Decimal,
    },

    /// Takes resource from worktop by the given non-fungible IDs.
    #[sbor(discriminator(INSTRUCTION_TAKE_NON_FUNGIBLES_FROM_WORKTOP_DISCRIMINATOR))]
    TakeNonFungiblesFromWorktop {
        resource_address: ResourceAddress,
        ids: Vec<NonFungibleLocalId>,
    },

    /// Returns a bucket of resource to worktop.
    #[sbor(discriminator(INSTRUCTION_RETURN_TO_WORKTOP_DISCRIMINATOR))]
    ReturnToWorktop { bucket_id: ManifestBucket },

    /// Asserts worktop contains any specified resource.
    #[sbor(discriminator(INSTRUCTION_ASSERT_WORKTOP_CONTAINS_ANY_DISCRIMINATOR))]
    AssertWorktopContainsAny { resource_address: ResourceAddress },

    /// Asserts worktop contains resource by at least the given amount.
    #[sbor(discriminator(INSTRUCTION_ASSERT_WORKTOP_CONTAINS_DISCRIMINATOR))]
    AssertWorktopContains {
        resource_address: ResourceAddress,
        amount: Decimal,
    },

    /// Asserts worktop contains resource by at least the given non-fungible IDs.
    #[sbor(discriminator(INSTRUCTION_ASSERT_WORKTOP_CONTAINS_NON_FUNGIBLES_DISCRIMINATOR))]
    AssertWorktopContainsNonFungibles {
        resource_address: ResourceAddress,
        ids: Vec<NonFungibleLocalId>,
    },

    //==============
    // Auth zone
    //==============
    /// Takes the last proof from the auth zone.
    #[sbor(discriminator(INSTRUCTION_POP_FROM_AUTH_ZONE_DISCRIMINATOR))]
    PopFromAuthZone,

    /// Adds a proof to the auth zone.
    #[sbor(discriminator(INSTRUCTION_PUSH_TO_AUTH_ZONE_DISCRIMINATOR))]
    PushToAuthZone { proof_id: ManifestProof },

    /// Clears the auth zone.
    #[sbor(discriminator(INSTRUCTION_CLEAR_AUTH_ZONE_DISCRIMINATOR))]
    ClearAuthZone,

    /// Creates a proof from the auth zone
    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_DISCRIMINATOR))]
    CreateProofFromAuthZone { resource_address: ResourceAddress },

    /// Creates a proof from the auth zone, by the given amount
    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT_DISCRIMINATOR))]
    CreateProofFromAuthZoneOfAmount {
        resource_address: ResourceAddress,
        amount: Decimal,
    },

    /// Creates a proof from the auth zone, by the given non-fungible IDs.
    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES_DISCRIMINATOR))]
    CreateProofFromAuthZoneOfNonFungibles {
        resource_address: ResourceAddress,
        ids: Vec<NonFungibleLocalId>,
    },

    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL_DISCRIMINATOR))]
    CreateProofFromAuthZoneOfAll { resource_address: ResourceAddress },

    /// Drop all virtual proofs (can only be auth zone proofs).
    #[sbor(discriminator(INSTRUCTION_CLEAR_SIGNATURE_PROOFS_DISCRIMINATOR))]
    ClearSignatureProofs,

    //==============
    // Named bucket
    //==============
    /// Creates a proof from a bucket.
    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_BUCKET_DISCRIMINATOR))]
    CreateProofFromBucket { bucket_id: ManifestBucket },

    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_AMOUNT_DISCRIMINATOR))]
    CreateProofFromBucketOfAmount {
        bucket_id: ManifestBucket,
        amount: Decimal,
    },

    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES_DISCRIMINATOR))]
    CreateProofFromBucketOfNonFungibles {
        bucket_id: ManifestBucket,
        ids: Vec<NonFungibleLocalId>,
    },

    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_ALL_DISCRIMINATOR))]
    CreateProofFromBucketOfAll { bucket_id: ManifestBucket },

    #[sbor(discriminator(INSTRUCTION_BURN_RESOURCE_DISCRIMINATOR))]
    BurnResource { bucket_id: ManifestBucket },

    //==============
    // Named proof
    //==============
    /// Clones a proof.
    #[sbor(discriminator(INSTRUCTION_CLONE_PROOF_DISCRIMINATOR))]
    CloneProof { proof_id: ManifestProof },

    /// Drops a proof.
    #[sbor(discriminator(INSTRUCTION_DROP_PROOF_DISCRIMINATOR))]
    DropProof { proof_id: ManifestProof },

    //==============
    // Invocation
    //==============
    #[sbor(discriminator(INSTRUCTION_CALL_FUNCTION_DISCRIMINATOR))]
    CallFunction {
        package_address: DynamicPackageAddress,
        blueprint_name: String,
        function_name: String,
        args: ManifestValue,
    },

    #[sbor(discriminator(INSTRUCTION_CALL_METHOD_DISCRIMINATOR))]
    CallMethod {
        address: DynamicGlobalAddress,
        method_name: String,
        args: ManifestValue,
    },

    #[sbor(discriminator(INSTRUCTION_CALL_ROYALTY_METHOD_DISCRIMINATOR))]
    CallRoyaltyMethod {
        address: DynamicGlobalAddress,
        method_name: String,
        args: ManifestValue,
    },

    #[sbor(discriminator(INSTRUCTION_CALL_METADATA_METHOD_DISCRIMINATOR))]
    CallMetadataMethod {
        address: DynamicGlobalAddress,
        method_name: String,
        args: ManifestValue,
    },

    #[sbor(discriminator(INSTRUCTION_CALL_ACCESS_RULES_METHOD_DISCRIMINATOR))]
    CallAccessRulesMethod {
        address: DynamicGlobalAddress,
        method_name: String,
        args: ManifestValue,
    },

    #[sbor(discriminator(INSTRUCTION_CALL_DIRECT_VAULT_METHOD_DISCRIMINATOR))]
    CallDirectVaultMethod {
        address: InternalAddress,
        method_name: String,
        args: ManifestValue,
    },

    //==============
    // Complex
    //==============
    /// Drops all proofs, both named proofs and auth zone proofs.
    #[sbor(discriminator(INSTRUCTION_DROP_ALL_PROOFS_DISCRIMINATOR))]
    DropAllProofs,

    #[sbor(discriminator(INSTRUCTION_ALLOCATE_GLOBAL_ADDRESS_DISCRIMINATOR))]
    AllocateGlobalAddress {
        package_address: PackageAddress,
        blueprint_name: String,
    },
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
pub const INSTRUCTION_CLEAR_AUTH_ZONE_DISCRIMINATOR: u8 = 0x12;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_DISCRIMINATOR: u8 = 0x13;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT_DISCRIMINATOR: u8 = 0x14;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES_DISCRIMINATOR: u8 = 0x15;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL_DISCRIMINATOR: u8 = 0x16;
pub const INSTRUCTION_CLEAR_SIGNATURE_PROOFS_DISCRIMINATOR: u8 = 0x17;

//==============
// Named bucket
//==============
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_DISCRIMINATOR: u8 = 0x20;
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
pub const INSTRUCTION_CALL_ACCESS_RULES_METHOD_DISCRIMINATOR: u8 = 0x44;
pub const INSTRUCTION_CALL_DIRECT_VAULT_METHOD_DISCRIMINATOR: u8 = 0x45;

//==============
// Complex
//==============
pub const INSTRUCTION_DROP_ALL_PROOFS_DISCRIMINATOR: u8 = 0x50;
pub const INSTRUCTION_ALLOCATE_GLOBAL_ADDRESS_DISCRIMINATOR: u8 = 0x51;
