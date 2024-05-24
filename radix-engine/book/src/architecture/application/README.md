# Application Layer

The application layer is responsible for defining high level logic which manipulates objects
and produces events for the eventual use by off-ledger systems such as wallets and DApps.

## Implementation

An application is added to the system by publishing a *Package*, which contain zero or
more *Blueprints*. Each blueprint defines object type information and logic which can create,
manipulate and destroy objects of that blueprint type.

