# VM Layer
The VM Layer is responsible for providing the computing environment to the application layer. This includes a Turing-complete VM as well as the interface to access the system layer.

Radix Engine currently supports two VMs: a Scrypto WASM VM used to execute Scrypto applications and a native VM which executes native packages which are compiled to the host’s environment.