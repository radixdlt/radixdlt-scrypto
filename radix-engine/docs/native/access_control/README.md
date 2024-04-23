# Access Control

Unlike the majority of blockchains which rely on a caller identifier for access control, the Access Control
system uses a more distributed "Proof" system. Before accessing a protected method a caller must provide
specific "Proofs" of resources they have access to. These proofs must then match the required proofs
defined by protected method or function of the callee.

The Access Control System is composed of four parts:

1. A Package Module, which defines roles available to use for a given blueprint in a package
2. An Object Module, which provides role assignment or which resources are required to show proof for a given role
3. A System Module, which maintains an AuthZone, or the current proofs a runtime caller has
3. An AuthZone Blueprint, which allows a caller to update their current proofs

These three modules allow a package creator to define roles which a component creator can assign definitions to,
which a caller must show proof for by updating their AuthZone.

