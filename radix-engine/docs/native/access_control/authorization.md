# Authorization

On rule check time, the AuthModule checks the proofs on the caller's AuthZone and determines which roles for this
object the caller has. If any of these roles match the list of roles assigned to the method, the caller is then
authorized to make the call.

Unlike traditional RBAC where the role a user is acting is explicit, in this model roles are more implicit and
defined on what proofs the user has in their AuthZone. This makes it a cross between the well-known RBAC and
ABAC models.
