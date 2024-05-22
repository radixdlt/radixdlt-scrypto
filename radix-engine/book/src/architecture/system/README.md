# System Layer

The System Layer is responsible for:
* Defining Actor abstraction and memory protection
* Defining the Package/Blueprint/Object abstraction
* Maintaining a set of System Modules, or pluggable software, which extends the
functionality of the system.

## Implementation

The System Layer is implemented by defining the Kernel Callback Object and defining the
Actor/Package/Blueprint/Object abstractings on top of the kernel's Node/Partition/Substate
abstractions.
