# Architecture

Radix Engine is organized into 5 layers. Each layer has specific responsibilities and
provides an API to the layer above. Middle layers also provide a Callback API which the
layer above must implement.

| Layer Name  | Layer ID | Responsibilities                                                                                                                                                  |
|-------------|----------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Application | 5        | Defines Blueprint Application Logic                                                                                                                               |
| VM          | 4        | Executes Application Code                                                                                                                                         |
| System      | 3        | Defines Actor abstraction (Memory Protection)<br>Defines Package, Blueprint, Object abstractions<br>Defines System Standards such as Authorization and Versioning |
| Kernel      | 2        | Defines Node, Partition, Substate abstractions<br>Maintains Call Frame Stack<br>Maintains Ownership/Reference invariants                                          |
| Database    | 1        | Defines PartitionKey, SortKey abstractions                                                                                                                        |

