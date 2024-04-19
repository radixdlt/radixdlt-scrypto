# System Layer

The System Layer is responsible for maintaining a set of System Modules, or pluggable software which can extend the functionality of the system.

On every system call, each system module gets called before the system layer passes control to the kernel layer. When called, each system module may update some particular state (e.g. update fees spent) or panic to end the transaction (e.g. if the type checker fails).