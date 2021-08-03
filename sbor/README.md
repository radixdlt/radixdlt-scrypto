# Scrypto Binary Object Representation

Scrypto Binary Object Representation (SBOR) is an open, efficient and Rust-native data format used by Scrypto and Radix Engine V2.

## Why Another Data Format?

Data serialization and deserialization are required for Scrypto in many places, e.g. system function calls and component interactions. We need a framework that supports efficient data encoding, decoding and describing.

Serde and its supported data formats have been a good start, but didn't meet all our requirements.
- Bincode is performant but require data schema for decoding;
- JSON is self-descriptive but is slow because of its text-based representation;
- Neither supports schema generation.

## Design Objectives

- **Rust Native**: It should support most, if not all, Rust types.
- **Schemaless**: It should support schemaless data encoding and encoding.
- **Fully Specified**: It should come with full specification.
- **Fast**: It should be fast.