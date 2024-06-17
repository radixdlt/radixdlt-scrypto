# Hooks

> **_NOTE:_** In [Radix Babylon Network](../../../#radix-babylon-network),
> Hooks are currently only available for use by native packages.
 
Hooks define logic which get executed when certain system events occur.

There are currently three types of hooks:

| Hook Name    | Description                                                                      |
|--------------|----------------------------------------------------------------------------------|
| OnVirtualize | Called when a substate fault occurs on a virtual address of this blueprint type. |
| OnMove       | Called when an object of this blueprint type is moved between call frames.       |
| OnDrop       | Called when an object of this blueprint type is dropped.                         |
