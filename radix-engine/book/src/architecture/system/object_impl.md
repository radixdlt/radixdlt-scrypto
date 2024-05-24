# Object Implementation

The system layer defines the [Object](../application/object/README.md) abstraction on top of the
kernel's Node/Partition/Substate abstraction.

The system layer maps every object to a unique NodeId and under every NodeId the partitions are
mapped in the following manner:

|                   | Partition Number |
|-------------------|------------------|
| Type Info         | 0                |
| Schema Data       | 1                |
| Object Modules    | 2-31             |
| Reserved          | 32-63            |
| Application State | 64-255           |
