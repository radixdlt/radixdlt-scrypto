# Outer Objects

Objects which have a blueprint which is an [Inner Blueprint](inner_outer_objects.md) will have an
associated Outer object of a given outer Blueprint. Objects of an Inner Blueprint may directly access
the state of its outer object avoiding invocation and new call frame overhead + costs.

![](inner_outer_objects.drawio.svg)