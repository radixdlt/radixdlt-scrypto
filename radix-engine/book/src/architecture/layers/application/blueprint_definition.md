# Blueprint Definition

A Blueprint Definition defines everything about a Blueprint outside of the code logic itself.

The structure used to initialize the definition is `BlueprintDefinitionInit` found in [invocations.rs](../../../../../radix-engine-interface/src/blueprints/package/invocations.rs):

```
pub struct BlueprintDefinitionInit {
    pub blueprint_type: BlueprintType,
    pub is_transient: bool,
    pub feature_set: IndexSet<String>,
    pub dependencies: IndexSet<GlobalAddress>,
    pub schema: BlueprintSchemaInit,
    pub royalty_config: PackageRoyaltyConfig,
    pub auth_config: AuthConfig,
}
```

## Properties

A description of each property is as follows.

| Property Name    | Description                                                                                                                                                                                                                                                                                                                |
|------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Blueprint Type   | A blueprint may be specified as either an Outer or Inner Blueprint. If an inner blueprint, an associated outer blueprint must be specified. Inner blueprint components may access the state of its outer blueprint component directly.<br><br> *Inner Blueprints are currently only available for use by native packages.* |
| Transience       | If a blueprint is specified to be transient, all components of this blueprint type may not be persisted.<br><br>*Transience is currently only available for use by native packages.*                                                                                                                                       |
| Feature Set      | Features provide a mechanism to express conditional execution and stored state. The feature set is the set of all possible features a component instantiator may specify.<br><br> *A non-empty Feature Set is currently only available for use by native packages.*                                                        |
| Dependencies     | The set of all addresses which will always be visible to the call frames of this blueprint.                                                                                                                                                                                                                                |
| Blueprint Schema | The schema of the blueprint including generics, interface, state, and events.                                                                                                                                                                                                                                              |
| Royalty Config   | Royalty configuration for this blueprint.                                                                                                                                                                                                                                                                                  |
| Auth Config      | Auth configuration such as role definitions for this blueprint.                                                                                                                                                                                                                                                            |
