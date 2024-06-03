# Package Implementation

The package abstraction is implemented as a native blueprint. In order to get around the
[circular definition problem](../application/package/README.md#package-blueprint-and-package-package),
the package logic and structure must be [flashed into the system](../../protocol/genesis_bootstrap.md)
at genesis.
