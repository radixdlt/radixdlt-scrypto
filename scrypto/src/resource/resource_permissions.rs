const RESOURCE_PERMISSIONS_SHIFT: u8 = 32;
const RESOURCE_PERMISSIONS_MASK: u64 = 0xffff_ffff_0000_0000;

macro_rules! resource_permissions {
    ( $f:expr ) => {
        ($f as u64) << RESOURCE_PERMISSIONS_SHIFT
    }
}

#[inline]
pub fn resource_permissions_are_valid(flags: u64) -> bool {
    (flags & RESOURCE_PERMISSIONS_MASK) == flags
}

/// May transfer owned resources.
pub const MAY_TRANSFER: u64 = resource_permissions!(1u32 << 0);

/// May burn owned resources.
pub const MAY_BURN: u64 = resource_permissions!(1u32 << 2);

/// May create new supply.
pub const MAY_MINT: u64 = resource_permissions!(1u32 << 4);

/// (Not implemented) May seize from any vault.
pub const MAY_RECALL: u64 = resource_permissions!(1u32 << 5);

/// May change top-level resource metadata, e.g. name and symbol.
pub const MAY_CHANGE_SHARED_METADATA: u64 = resource_permissions!(1u32 << 6);

/// May change the mutable data part of an individual NFT.
pub const MAY_CHANGE_INDIVIDUAL_METADATA: u64 = resource_permissions!(1u32 << 7);

/// May change mutable flags.
pub const MAY_MANAGE_RESOURCE_FLAGS: u64 = resource_permissions!(1u32 << 8);

/// All permissions.
pub const ALL_PERMISSIONS: u64 = resource_permissions!(!0u32);
