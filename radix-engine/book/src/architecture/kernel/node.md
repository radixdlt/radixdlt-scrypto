# Node

A node is a movable entity with state which may own other nodes. Child nodes are exclusively
owned by its parent node. If a node has no owner then it is a "globalized" node.

![](node_model.drawio.svg)

A node has no notion of blueprints, types, or object modules. These abstractions are added
on top of the node abstraction by the system layer.