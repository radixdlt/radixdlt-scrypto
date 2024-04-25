# Inner and Outer Blueprints

A blueprint may be specified as either an Outer or Inner Blueprint. If an inner blueprint, an associated outer blueprint
(from the same package) must be specified.

![](inner_outer_blueprints.drawio.svg)

Inner blueprint objects may only be instantiated by an associated outer blueprint object. After
instantiation, inner blueprint objects may access the state of its outer blueprint component directly
avoiding invocation and new call frame costs.

![](inner_outer_objects.drawio.svg)

*Inner Blueprints are currently only available for use by native packages.*
