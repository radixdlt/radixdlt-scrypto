# Architecture

Radix Engine is organized into 5 layers. Each layer has specific responsibilities and
provides an API to the layer above. Middle layers also provide a Callback API which the
layer above must implement.

| Layer Name  | Layer ID | Responsibilities                                                                                                          |
|-------------|----------|---------------------------------------------------------------------------------------------------------------------------|
| Application | 5        | Defines Types and Application Logic                                                                                       |
| VM          | 4        | Interprets Application Code                                                                                               |
| System      | 3        | Type Checks<br>Defines Package/Blueprint/Object abstractions<br>Defines System Standards (e.g. Authorization, Versioning) |
| Kernel      | 2        | Maintains Call Frame Stack<br>Manages Ownership/Reference handling invariants<br>Provides State Virtualization Mechanism  |
| Database    | 1        | Interacts with a Read-Only Database                                                                                       |

