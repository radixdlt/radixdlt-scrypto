# Auth Blueprint Module

The auth [blueprint module](../../architecture/application/blueprint/blueprint_modules.md) defines three
things for every blueprint:
* Function AccessRules
* Method accessibility
* Role Specification

## Function AccessRules

Each function is assigned an immutable access rule.

## Method Accessibility

Each method is assigned an accessibility rule, of which there are four options:

| Accessibility Rule | Description                                                                    |
|--------------------|--------------------------------------------------------------------------------|
| Public             | Anyone can access the method                                                   |
| Outer Object Only  | Only outer objects may access the method                                       |
| Role Protected     | Only callers who have satisfied any role in a given list may access the method |
| Own Package Only   | Only the package this method is a part of may access the method                |

## Role Specification

The roles which must be assigned on object instantiated are defined in role specification.
Furthermore, roles which may update the rules of other roles must be specified.

For inner blueprints, it is also possible to defer role specification to the outer blueprint.