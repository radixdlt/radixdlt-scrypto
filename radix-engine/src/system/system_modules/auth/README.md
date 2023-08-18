# Auth Module

Unlike the majority of blockchains which rely on a caller identifier for auth, the Auth Module
uses a more distributed "Proof" system. Before accessing a protected method a caller must provide
specific "Proofs" of resources they have access to. These proofs must then match the required proofs
defined by protected method or function of the callee.

## AuthZone

To call a protected method, the caller must place these proofs into their [AuthZone](../../../blueprints/resource/auth_zone),
a space dedicated for using proofs for the purpose of authorized method access. On method call, the Auth Module
then checks the caller's AuthZone and compares it to the rules specified by the Callee.

These rules are specified by the Callee on Object instantiation and are defined by using a mixture of **Role-Based
Access Control** (RBAC) and **Attribute-Based Access Control** (ABAC) techniques.

## Role Assignment

On the callee side, instead of roles being assigned to a "user" (of which there is no concept in our decentralized
ledger), roles are assigned through resource ownership. For example, a "Staff" role could be assigned to anyone who
can show proof that they own a "Staff" resource token. This is defined at the callee's Object instantiation
through the [RoleAssignment](../../node_module/role_assignment) object module.

## Role Definition

All roles for a given object are defined (though not assigned!) at the Blueprint level by the Package creator. This
includes definitions regarding which methods a given role is given access to. 

## Authorization

On rule check time, the AuthModule checks the proofs on the caller's AuthZone and determines which roles for this
object the caller has. If any of these roles match the list of roles assigned to the method, the caller is then
authorized to make the call.

Unlike traditional RBAC where the role a user is acting is explicit, in this model roles are more implict and
defined on what proofs the user has in their AuthZone. This makes it a cross between the well-known RBAC and
ABAC models.
