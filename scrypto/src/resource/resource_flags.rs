const RESOURCE_FLAGS_SHIFT: u8 = 0;
const RESOURCE_FLAGS_MASK: u64 = 0xffff_ffff;

macro_rules! resource_flags {
    ( $f:expr ) => {
        ($f as u64) << RESOURCE_FLAGS_SHIFT
    };
}

#[inline]
pub fn resource_flags_are_valid(flags: u64) -> bool {
    (flags & RESOURCE_FLAGS_MASK) == flags
}

/// Resource can only be taken from vault with authority present.
pub const RESTRICTED_TRANSFER: u64 = resource_flags!(1u32 << 0);

/// Resource can be burned.
pub const BURNABLE: u64 = resource_flags!(1u32 << 1);

/// Resource can be burned by the holder, without any authority.
pub const FREELY_BURNABLE: u64 = resource_flags!(1u32 << 2);

/// New supply can be minted.
pub const MINTABLE: u64 = resource_flags!(1u32 << 3);

/// (Not implemented) Resource can be seized from any vault if proper authority is presented.
pub const RECALLABLE: u64 = resource_flags!(1u32 << 4);

/// Top-level resource metadata can be changed.
pub const SHARED_METADATA_MUTABLE: u64 = resource_flags!(1u32 << 5);

/// The mutable data part of an individual NFT can be modified.
pub const INDIVIDUAL_METADATA_MUTABLE: u64 = resource_flags!(1u32 << 6);

/// All resources flags.
pub const ALL_FLAGS: u64 = resource_flags!(!0u32);
