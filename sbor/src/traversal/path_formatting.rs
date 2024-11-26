use core::ops::Deref;

use crate::rust::prelude::*;

pub struct AnnotatedSborAncestor<'a> {
    /// Ideally the type's type name, else the value kind name or type name, depending on what's available
    pub name: Cow<'a, str>,
    pub container: AnnotatedSborAncestorContainer<'a>,
}

impl<'a> AnnotatedSborAncestor<'a> {
    pub fn write(
        &self,
        f: &mut impl core::fmt::Write,
        is_start_of_path: bool,
    ) -> core::fmt::Result {
        let AnnotatedSborAncestor { name, container } = self;

        if !is_start_of_path {
            write!(f, "->")?;
        }

        write!(f, "{name}")?;
        container.write(f)?;

        Ok(())
    }
}

pub enum AnnotatedSborAncestorContainer<'a> {
    Tuple {
        field_index: usize,
        field_name: Option<Cow<'a, str>>,
    },
    EnumVariant {
        discriminator: u8,
        variant_name: Option<Cow<'a, str>>,
        field_index: usize,
        field_name: Option<Cow<'a, str>>,
    },
    Array {
        /// If locating a value, we will have indices, if locating a type, we won't
        index: Option<usize>,
    },
    Map {
        /// If locating a value, we will have indices, if locating a type, we won't
        index: Option<usize>,
        entry_part: MapEntryPart,
    },
}

impl<'a> AnnotatedSborAncestorContainer<'a> {
    pub fn write(&self, f: &mut impl core::fmt::Write) -> core::fmt::Result {
        // This should align with AnnotatedSborPartialLeafLocator
        match self {
            Self::Tuple {
                field_index,
                field_name,
            } => {
                if let Some(field_name) = field_name {
                    write!(f, ".[{field_index}|{field_name}]")?;
                } else {
                    write!(f, ".[{field_index}]")?;
                }
            }
            Self::EnumVariant {
                discriminator: variant_discriminator,
                variant_name,
                field_index,
                field_name,
            } => {
                if let Some(variant_name) = variant_name {
                    write!(f, "::{{{variant_discriminator}|{variant_name}}}")?;
                } else {
                    write!(f, "::{{{variant_discriminator}}}")?;
                }
                if let Some(field_name) = field_name {
                    write!(f, ".[{field_index}|{field_name}]")?;
                } else {
                    write!(f, ".[{field_index}]")?;
                }
            }
            Self::Array { index } => {
                if let Some(index) = index {
                    write!(f, ".[{index}]")?;
                }
            }
            Self::Map { index, entry_part } => {
                if let Some(index) = index {
                    write!(f, ".[{index}]")?;
                }
                match entry_part {
                    MapEntryPart::Key => write!(f, ".Key")?,
                    MapEntryPart::Value => write!(f, ".Value")?,
                }
            }
        }
        Ok(())
    }
}

pub struct AnnotatedSborPartialLeaf<'a> {
    /// Ideally the type's type name, else the value kind name or type name, depending on what's available
    pub name: Cow<'a, str>,
    pub partial_leaf_locator: Option<AnnotatedSborPartialLeafLocator<'a>>,
}

impl<'a> AnnotatedSborPartialLeaf<'a> {
    pub fn write(
        &self,
        f: &mut impl core::fmt::Write,
        is_start_of_path: bool,
    ) -> core::fmt::Result {
        let AnnotatedSborPartialLeaf {
            name,
            partial_leaf_locator: partial_kinded_data,
        } = self;

        if !is_start_of_path {
            write!(f, "->")?;
        }

        write!(f, "{}", name.deref())?;
        if let Some(partial_kinded_data) = partial_kinded_data {
            partial_kinded_data.write(f)?;
        }

        Ok(())
    }
}

pub enum AnnotatedSborPartialLeafLocator<'a> {
    Tuple {
        field_index: Option<usize>,
        field_name: Option<Cow<'a, str>>,
    },
    EnumVariant {
        variant_discriminator: Option<u8>,
        variant_name: Option<Cow<'a, str>>,
        field_index: Option<usize>,
        field_name: Option<Cow<'a, str>>,
    },
    Array {
        index: Option<usize>,
    },
    Map {
        index: Option<usize>,
        entry_part: Option<MapEntryPart>,
    },
}

impl<'a> AnnotatedSborPartialLeafLocator<'a> {
    pub fn write(&self, f: &mut impl core::fmt::Write) -> core::fmt::Result {
        // This should align with AnnotatedSborAncestorContainer
        match self {
            Self::Tuple {
                field_index,
                field_name,
            } => {
                if let Some(field_index) = field_index {
                    if let Some(field_name) = field_name {
                        write!(f, ".[{field_index}|{field_name}]")?;
                    } else {
                        write!(f, ".[{field_index}]")?;
                    }
                }
            }
            Self::EnumVariant {
                variant_discriminator,
                variant_name,
                field_index,
                field_name,
            } => {
                if let Some(variant_discriminator) = variant_discriminator {
                    if let Some(variant_name) = variant_name {
                        write!(f, "::{{{variant_discriminator}|{variant_name}}}")?;
                    } else {
                        write!(f, "::{{{variant_discriminator}}}")?;
                    }
                    if let Some(field_index) = field_index {
                        if let Some(field_name) = field_name {
                            write!(f, ".[{field_index}|{field_name}]")?;
                        } else {
                            write!(f, ".[{field_index}]")?;
                        }
                    }
                }
            }
            Self::Array { index } => {
                if let Some(index) = index {
                    write!(f, ".[{index}]")?;
                }
            }
            Self::Map { index, entry_part } => {
                if let Some(index) = index {
                    write!(f, ".[{index}]")?;
                }
                if let Some(entry_part) = entry_part {
                    match entry_part {
                        MapEntryPart::Key => write!(f, ".Key")?,
                        MapEntryPart::Value => write!(f, ".Value")?,
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MapEntryPart {
    Key,
    Value,
}

pub trait PathAnnotate {
    fn iter_ancestor_path(&self) -> Box<dyn Iterator<Item = AnnotatedSborAncestor<'_>> + '_>;

    fn annotated_leaf(&self) -> Option<AnnotatedSborPartialLeaf<'_>>;

    fn format_path(&self) -> String {
        let mut buf = String::new();
        self.write_path(&mut buf).unwrap();
        buf
    }

    fn write_path(&self, f: &mut impl core::fmt::Write) -> core::fmt::Result {
        let mut is_start_of_path = true;
        for ancestor in self.iter_ancestor_path() {
            ancestor.write(f, is_start_of_path)?;
            is_start_of_path = false;
        }

        if let Some(leaf) = self.annotated_leaf() {
            leaf.write(f, is_start_of_path)?;
        } else {
            if is_start_of_path {
                write!(f, "[Root]")?;
            }
        }

        Ok(())
    }
}
