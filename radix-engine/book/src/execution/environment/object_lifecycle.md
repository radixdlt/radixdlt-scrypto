# Object Lifecycle

![](object_lifecycle.drawio.svg)

## Instantiation

An object may be instantiated by using one of the system calls:
* `object_new_simple_object`
* `object_new_object`

On instantiation the set of features, generic arguments, and initial state must be passed in to
construct the object. Only blueprints of the currently acting package may be instantiated. If the
blueprint is an inner blueprint, only an acting outer blueprint component may instantiate that inner
blueprint.

## Destruction

An object may be dropped by using the `object_drop` system call.

## Globalization

An object may be globalized using one of the system calls:
* `object_globalize`
* `object_globalize_with_address_and_create_inner_object_and_emit_event`

Once globalized an object is associated with a global address and may be referenced without ownership
of the object. Thus, it may be referenced in a transaction or in blueprint code.

## Moved to ownership by another object

A call frame owned object may be moved to ownership by another object if it is moved via one of the
system calls:
* `object_new_simple_object`
* `object_new_object`
* `key_value_entry_set`
* `field_write`
