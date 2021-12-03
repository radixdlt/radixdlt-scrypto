/// (Not implemented) Resource can only be taken from vault with authority present.
pub const RESTRICTED_TRANSFER: u16 = 1u16 << 0;

/// Resource can be burned.
pub const BURNABLE: u16 = 1u16 << 1;

/// Resource can be burned by the holder, without any authority.
pub const FREELY_BURNABLE: u16 = 1u16 << 2;

/// New supply can be minted.
pub const MINTABLE: u16 = 1u16 << 3;

/// (Not implemented) Resource can be seized from any vault if proper authority is presented.
pub const RECALLABLE: u16 = 1u16 << 4;

/// Top-level resource metadata can be changed.
pub const SHARED_METADATA_MUTABLE: u16 = 1u16 << 5;

/// The mutable data part of an individual NFT can be modified.
pub const INDIVIDUAL_METADATA_MUTABLE: u16 = 1u16 << 6;

/// All resources flags.
pub const ALL_FLAGS: u16 = !0u16;
