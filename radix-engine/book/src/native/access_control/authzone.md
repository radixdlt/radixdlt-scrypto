# AuthZone

To call a protected method, the caller must place these proofs into their [AuthZone](../../../blueprints/resource/auth_zone),
a space dedicated for using proofs for the purpose of authorized method access. On method call, the Auth Module
then checks the caller's AuthZone and compares it to the rules specified by the Callee.

These rules are specified by the Callee on Object instantiation and are defined by using a mixture of **Role-Based
Access Control** (RBAC) and **Attribute-Based Access Control** (ABAC) techniques.