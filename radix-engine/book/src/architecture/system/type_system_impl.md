# Type System

The system layer is responsible for implementing the [type system abstraction](../application/type_system/README.md).

For a given object, the `BlueprintId`, `GenericSubstitutions`, and other type-related info is stored
under the object's `NodeId` in the TypeInfo substate found in `PartitionNumber 0` and `SubstateKey::Field 0`.

Local Scrypto Schemas for the object are stored in the object's `NodeId` with `PartitionNumber 2` with
a content addressable substate key.

Remote Scrypto Schemas are stored in the blueprint's package `NodeId` with `PartitionNumber 2` with
a content addressable substate key.
