use basic_well_known_types::ANY_TYPE;

use crate::internal_prelude::*;
use crate::schema::*;
use crate::traversal::*;
use crate::BASIC_SBOR_V1_MAX_DEPTH;
use radix_rust::rust::fmt::Write;

mod comparable_schema;
mod comparisons_and_assertions;
mod schema_comparison_kernel;
mod schema_comparison_result;
mod schema_comparison_settings;

pub use comparable_schema::*;
pub use comparisons_and_assertions::*;
use schema_comparison_kernel::*;
pub use schema_comparison_result::*;
pub use schema_comparison_settings::*;

/// Marker traits intended to be implemented by the SborAssert macros
pub trait CheckedFixedSchema<S: CustomSchema>: CheckedBackwardsCompatibleSchema<S> {}
pub trait CheckedBackwardsCompatibleSchema<S: CustomSchema> {}

// NOTE: Types are in sbor-tests/tests/schema_comparison.rs
