# Fields

> **_NOTE:_** Use of more than one Field, Field Conditions and Field Transience are currently only available for use by native packages.

A field is object state which gets loaded at once and maps to a single substate. A schema which
describes what is in the data must be specified for every field.

Fields are identified by field index.

## Field Condition

Fields may be conditionally included in an object depending on the features instantiated
with that object. There are currently three options for field conditions:

| Name           | Description                                                                           |
|----------------|---------------------------------------------------------------------------------------|
| Always         | Always include the field                                                              |
| IfFeature      | Only include the field if a given feature is specified                                |
| IfOuterFeature | Only include the field if a given feature in the associated outer object is specified |

## Field Transience

Fields may be specified to be transient. In this case, the field is never persisted. Instead, a default
value is initially loaded on first read and may be updated over the course of a transaction. At the end
of a transaction the field's value gets discarded.
