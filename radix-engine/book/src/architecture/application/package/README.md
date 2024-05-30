# Package

A package is a special native object which contains 0 or more blueprint definitions. Because it is an
object, packages inherit object-like qualities such as the ability to have object modules
(like metadata).

This creates the following circular definition:

`An Object is of some Blueprint type.`

`A Blueprint is part of a Package.`

`A Package is an Object.`

## Package Blueprint and Package Package

This circular definition creates the notion of the Package Blueprint and the
Package Package (similar to Class.class in java). A Package Blueprint is the
blueprint type of a Package object and Package Package is the package which
contains the Package Blueprint.