//! This module is for representing SBOR via the serde serialization format.
//! In particular, it's been optimised for serializing to JSON, but can also be serialized into other formats.
//!
//! To use this module, you need to enable the `serde` feature.
//!
//! You can then:
//! ```ignore
//!     // Ensure the ContextualSerialize trait is in scope.
//!     // You will need to enable the "sbor" feature of utils.
//!     use radix_rust::*;
//!     use sbor::representations::*;
//!
//!     let payload = BasicPayload::new(&payload_bytes);
//!     let serializable = payload.serializable(
//!         // Provide some SerializationParameters
//!     );
//!
//!     // You can then make use of the serializable value using a serde serializer - eg serde_json.
//!     // NB: If you are using std, it is recommended to enable the `preserve_order` feature of serde_json,
//!     // which ensures discriminators are printed first where possible, which can make deserialization more
//!     // efficient in some cases.
//!     let json = serde_json::to_string(&serializable).unwrap();
//! ```

// Imports and Exports
mod contextual_serialize;
mod serde_serializer;
mod traits;
mod value_map_aggregator;

pub use contextual_serialize::*;
pub use serde_serializer::*;
pub use traits::*;
pub use value_map_aggregator::*;
