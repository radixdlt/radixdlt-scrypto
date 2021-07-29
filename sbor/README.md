# SBOR

SBOR stands for Scrypto Binary Object Representation. It's an open data exchange format used by Scrypto and Radix Engine V2.

## Why Another Data Format?

Data serialization and deserialization are required for Scrypto in many places, e.g. system function calls and component interactions. We need a framework that supports efficient value encoding, decoding and describing.

Serde and its supported data formats was a good start, but didn't meet all our requirements.
- `bincode` is fast but data decoding requires the schema beforehand;
- `serde_json` provides all the feature but is slow given it's a text-based representation;
- Neither supports object schema generation.

## Design Objectives

- **Rust native**: Should support most, if not all, rust types with 1:1 mapping.
- **Schemaless**: Data should encode both type and value; schema is not required for decoding.
- **Fully specified**: Should come with full specification and reference implementation.
- **Fast**: De-/serialization should be fast.