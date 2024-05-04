# VM Layer

The VM Layer is responsible for providing a Turing-complete computing environment and the
system layer interface to the application layer. The VM Layer does this by defining the
System Callback Object.

Radix Engine currently supports two VMs:
* A Scrypto WASM VM which exposes the system layer interface through WASM extern functions
* A Native VM which is currently compiled directly in the host's environment with the Radix Engine
