## Node State

A node's substate is addressed by `<PartitionNumber> + <SubstateKey>`. A `PartitionNumber`
is a single byte representing a "partition" of state. A `SubstateKey` is variable length
and may be one of three types:

* FieldKey, which is a single byte
* MapKey, which is variable length
* SortedKey, which includes a 2 byte sort key and variable length for the rest of the key

![](partition_number_substate_key.drawio.svg)

Every partition must be composed of only one type of SubstateKey. This invariant must be
maintained by the higher layer, or in this case, the system layer.

