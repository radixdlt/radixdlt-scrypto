# Access Control

Unlike the majority of blockchains which rely on a caller identifier for access control,
the Access Control system uses a more distributed "Proof" system. Before accessing a protected
method a caller must provide specific "Proofs" of resources they have access to. These proofs
must then match the required proofs defined by protected method or function of the callee.

The Access Control System is composed of four parts:

1. An [Access Control Blueprint Module](role_definition.md),
which defines function rules and roles available to use for a given blueprint in a package and which roles are able
to access which methods.
2. A [Role Assignment Object Module](role_assignment.md),
which assigns proof rules for each role defined in the object's blueprint's role definition.
3. An [AuthZone Blueprint](../../architecture/application/blueprint/README.md), which allows
a caller to update their current proofs.
4. An AuthZone System Module, which creates a new AuthZone for every new call frame and verifies
   that AuthZone proofs match the requirements of accessing an object's function.