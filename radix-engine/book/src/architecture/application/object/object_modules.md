# Object Modules

The system may define additional state/logic to be stored per globalized object known as
an *Object Module*. The system can define whether an Object Module is required or optional.

An Object Module itself has a Blueprint type along with associated logic to manipulate
the state of the object module.

![](object_modules.drawio.svg)

Currently, there exists three object modules:
* [RoleAssignment](../../../native/auth/role_assignment.md) (Required)
* [Metadata](../../../native/metadata/object_module.md) (Required)
* [Component Royalties](../../../native/royalties/component_royalties.md) (Optional)
