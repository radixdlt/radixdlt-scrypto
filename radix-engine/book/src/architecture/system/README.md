# System Layer

The System Layer is responsible for:
* Defining Actor abstraction and memory protection
* Defining the Package/Blueprint/Object abstraction
* Maintaining a set of System Modules, or pluggable software, which extends the
functionality of the system.

## Implementation

The System Layer implements this by defining the Kernel Callback Object and using the
kernel Node/Partition/Substate abstractions.
