# VM Layer

The VM Layer is responsible for providing the application layer a Turing-complete computing
environment and the interface to the system layer interface.

Radix Engine currently supports two VM environments:
* A Scrypto WASM VM which exposes the system layer interface through WASM extern functions
* A Native VM which directly compiles applications with Radix Engine in the host's environment.

## Implementation

The VM Layer is implemented by defining the System Callback Object, which requires two callback
implementations:
1. `init` which is called on [transaction bootup](../../execution/transaction_lifecycle/bootup.md)
to initialize the vm layer
2. `invoke` which is called on any function/method call