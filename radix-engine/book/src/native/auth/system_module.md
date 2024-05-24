# Auth System Module

The Auth System Module is a [system module](../../architecture/system/system_modules.md) which operates
on every invocation:
1. Creates a new AuthZone
2. Resolve the requirements to access the method/function invocation
3. Verifies that the Global Caller AuthZone has sufficient proofs to meet the requirements

## AuthZone Creation

At the start of every invocation, the access control system module creates a new
[AuthZone](authzone.md) in the call frame of the caller and adds a reference to this object
in the callee's call frame. This AuthZone effectively becomes the "Local AuthZone" of the callee.

Every AuthZone references a global caller AuthZone and a parent AuthZone, the values of which
are dependent on if the invocation is a global object context switch or not.

If the invocation is a global object context switch, the global caller of the new AuthZone
will reference the caller's AuthZone and will not have a parent AuthZone. If the invocation
is a local context switch, the caller's global caller is copied into the new AuthZone and the
parent will reference the caller's AuthZone.

This pattern generates a stack which looks like:

![](auth_stack.drawio.svg)

## Permission Resolving

Permission resolving involves loading up relevant state of the callee and generating a permission
object from this state.

If the callee is a function then the permission is loaded from the function access rules
specified in the blueprint's [access control blueprint module](blueprint_module.md).

If the callee is a method then the Method Accessibility is loaded from the callee's
[access control blueprint module](blueprint_module.md) as well as the state in the callee's
[Role Assignment Object Module](role_assignment.md). From these two states, the permission to
access the method is derived.

## Auth Verification

Auth verification then checks the resolved permission against the AuthZones in the current
global context as well as the Global Caller's context.

![](auth_zones.drawio.svg)

In the above drawing, Call Frame 6 is making a new invocation and the AuthZones checked are
3/4/5/6, the AuthZones belonging to the current Global Context as well as the Global Caller's
Context.

