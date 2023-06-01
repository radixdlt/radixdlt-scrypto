//=================================================================================
// See REP-82 for justification behind this preparation strategy.
//
// Roughly:
// * Preparation: decoding + hash calculation
// * Validation: further checks + signature verification
//=================================================================================

mod decoder;
mod references;
mod summarized_composite;
mod summarized_raw;
mod summary;
mod traits;
pub use decoder::*;
pub use references::*;
pub use summarized_composite::*;
pub use summarized_raw::*;
pub use summary::*;
pub use traits::*;
