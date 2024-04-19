# Kernel Layer
The kernel layer is responsible for the two core functionalities of Radix Engine: storage access and communication between applications. This is somewhat similar to the traditional Operating System’s responsibility for disk and network access.

For Radix Engine, this includes the following low-level management:

* Check that move/borrow semantics are maintained on any invocation or data write. The single owner rule and borrow rules are enforced by the kernel. On failure on any of these rules, the transaction will panic.
* Manage transient vs. persistent objects. An object at any point in time may be in the global space or may be owned by a call frame. The kernel maintains correct pointers to these objects during runtime as references to these objects are passed around.
* Manage transaction state updates. The kernel keeps track of the state updates which have occurred during a transaction and which will be subsequently committed to the database at the end of the transaction.