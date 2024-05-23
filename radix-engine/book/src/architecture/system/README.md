# System Layer

The System Layer is responsible for:
* Defining the Package/Blueprint/[Object](object_implementation.md) abstraction
* Defining [Actor](actor_implementation.md) abstraction and memory protection
* Maintaining a set of System Modules, or pluggable software, which extends the
functionality of the system.

The System Layer is implemented by defining the Kernel Callback Object and defining the
Actor/Package/Blueprint/Object abstractings on top of the kernel's Node/Partition/Substate
abstractions.
