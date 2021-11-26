/// Resource can be transferred.
pub const TRANSFERABLE: u16 = 1u16 << 0;

/// Resource can be transferred by the holder, without any authorization.
pub const FREELY_TRANSFERABLE: u16 = 1u16 << 1;

/// Resource can be burned.
pub const BURNABLE: u16 = 1u16 << 2;

/// Resource can be burned by the holder, without any authorization.
pub const FREELY_BURNABLE: u16 = 1u16 << 3;

/// New supply can be minted.
pub const MINTABLE: u16 = 1u16 << 4;

/// Resource can be seized from any vault if proper authorization is presented.
pub const CLAWBACKABLE: u16 = 1u16 << 5;

/// Top-level metadata can be changed.
pub const SHARED_METADATA_MUTABLE: u16 = 1u16 << 6;

/// The mutable data part of an individual NFT can be modified.
pub const INDIVIDUAL_METADATA_MUTABLE: u16 = 1u16 << 7;

/// All resources flags.
pub const ALL_FLAGS: u16 = !0u16;
