/// May transfer owned resources.
pub const MAY_TRANSFER: u16 = 1u16 << 0;

/// May burn owned resources.
pub const MAY_BURN: u16 = 1u16 << 2;

/// May create new supply.
pub const MAY_MINT: u16 = 1u16 << 4;

/// May seize from any vault.
pub const MAY_CLAWBACK: u16 = 1u16 << 5;

/// May change top-level resource metadata, e.g. name and symbol.
pub const MAY_CHANGE_SHARED_METADATA: u16 = 1u16 << 6;

/// May change the mutable data part of an individual NFT.
pub const MAY_CHANGE_INDIVIDUAL_METADATA: u16 = 1u16 << 7;

/// May change mutable flags.
pub const MAY_CHANGE_FLAGS: u16 = 1u16 << 7;

/// All resources permissions.
pub const ALL_PERMISSIONS: u16 = !0u16;
