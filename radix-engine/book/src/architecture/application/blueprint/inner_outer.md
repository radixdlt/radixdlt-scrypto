# Inner and Outer Blueprints

> **_NOTE:_** Inner Blueprints are currently only available for use by native packages.

A blueprint may be specified as either an Outer or Inner Blueprint. Inner blueprints must specify
an associated outer blueprint defined in the same package.

![](inner_outer_blueprints.drawio.svg)

Inner blueprint objects may only be instantiated by an object of the associated outer blueprint.
