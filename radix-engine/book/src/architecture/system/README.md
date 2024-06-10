# System Layer

The System Layer is responsible for:
* Defining the [Object](object_impl.md), [Blueprint](blueprint_impl.md), [Type System](type_system_impl.md), and [Package](package_impl.md) abstraction
* Defining [Actor](actor_impl.md) abstraction and memory protection
* Maintaining a set of [System Modules](system_modules.md), or pluggable software, which extends the
functionality of the system.

## Implementation

The System Layer is implemented by defining the Kernel Callback Object and defining the
Actor/Package/Blueprint/Object abstractings on top of the kernel's Node/Partition/Substate
abstractions.
