# Inner and Outer Blueprints

> **_NOTE:_** Inner Blueprints are currently only available for use by native packages.

A blueprint may be specified as either an Outer or Inner Blueprint. Inner blueprints must specify
an associated outer blueprint defined in the same package.

![](inner_outer_blueprints.drawio.svg)

Inner blueprint objects may only be instantiated by an associated outer blueprint object. After
instantiation, inner blueprint objects may directly access the state of its outer blueprint component
avoiding invocation and new call frame overhead + costs.

![](inner_outer_objects.drawio.svg)

