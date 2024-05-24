# Invocations

An invocation is started by the application layer by calling one of the invocation system calls:
* `object_call_method`
* `object_call_direct_access_method`
* `object_call_module_method`
* `blueprint_call_function`

On one of these calls, the system then follows three phases:
1. Call Frame Setup
2. Execution
3. Call Frame Exit and Return

## Call Frame Setup

System module does its own checks (e.g. auth).

Kernel invoke is called which setups a new call frame. The arguments are verified against the input
schema of the function defined by the blueprint definition.

## Execution

Once the new call frame is setup, execution is passed to the application layer which may then execute it's
own logic in its application environment.

## Call Frame Exit

Once finished the system layer checks that the return value is of the correct schema given by the
blueprint definition. The kernel verifies that owned objects and references in the return value are
valid and the caller call frame is updated with any of these owned objects/references.