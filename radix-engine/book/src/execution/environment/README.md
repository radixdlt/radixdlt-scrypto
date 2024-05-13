# Application Environment

Every method/function execution has a call frame associated with it managed by the Kernel.

A call frame contains all owned and referenced objects usable by the running function. These objects
are referrable by `NodeId`.

## Invocations

Owned and referenced objects may have methods invoked (creating a new call frame). Owned objects may be
passed in as arguments and may be received in these invocations.

## Object Creation/Destruction/Globalization

Objects of the current blueprint may be instantiated, creating a new owned object into the call frame,
or dropped, in which case the owned object gets removed from the call frame.

## Actor State Read/Write

A call frame also contains a reference to the *actor*, or callee object (i.e. *self* in object-oriented
languages). This is maintained to allow read/writes of state for the given actor.

## System Module Functions

Additional system functions are available to the application layer implemented by System Modules.
Currently, these include:
* Events
* Logging
* Costing
* Transaction Runtime
