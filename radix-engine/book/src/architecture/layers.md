# Architecture

Radix Engine is organized into 5 layers. Each layer has specific responsibilities and
provides an API to the layer above. Middle layers also provide a Callback API which the
layer above must implement.

| Layer Name  | Responsibilities                                                                                                                                                  |
|-------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Application | Defines Blueprint Application Logic                                                                                                                               |
| VM          | Executes Application Code                                                                                                                                         |
| System      | Defines Actor abstraction (Memory Protection)<br>Defines Package, Blueprint, Object abstractions<br>Defines System Standards such as Authorization and Versioning |
| Kernel      | Defines Node, Partition, Substate abstractions<br>Maintains Call Frame Stack<br>Maintains Ownership/Reference invariants                                          |
| Database    | Defines PartitionKey, SortKey abstractions                                                                                                                        |

