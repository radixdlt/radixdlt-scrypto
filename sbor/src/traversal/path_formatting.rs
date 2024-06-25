use core::ops::Deref;

use crate::rust::prelude::*;

pub struct AnnotatedSborAncestor<'a> {
    /// Ideally the type's type name, else the value kind name or type name, depending on what's available
    pub name: Cow<'a, str>,
    pub container: AnnotatedSborAncestorContainer<'a>,
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

pub struct AnnotatedSborPartialLeaf<'a> {
    /// Ideally the type's type name, else the value kind name or type name, depending on what's available
    pub name: Cow<'a, str>,
    pub partial_leaf_locator: Option<AnnotatedSborPartialLeafLocator<'a>>,
}

pub enum AnnotatedSborPartialLeafLocator<'a> {
    Tuple {
        field_offset: Option<usize>,
        field_name: Option<&'a str>,
    },
    EnumVariant {
        variant_discriminator: Option<u8>,
        variant_name: Option<&'a str>,
        field_offset: Option<usize>,
        field_name: Option<&'a str>,
    },
    Array {
        index: Option<usize>,
    },
    Map {
        index: Option<usize>,
        entry_part: Option<MapEntryPart>,
    },
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

    fn write_path(&self, buf: &mut impl core::fmt::Write) -> core::fmt::Result {
        let mut is_first = true;
        for AnnotatedSborAncestor { name, container } in self.iter_ancestor_path() {
            if is_first {
                is_first = false;
            } else {
                write!(buf, "->")?;
            }
            write!(buf, "{name}")?;
            match container {
                AnnotatedSborAncestorContainer::Tuple {
                    field_index: field_offset,
                    field_name,
                } => {
                    if let Some(field_name) = field_name {
                        write!(buf, ".[{field_offset}|{field_name}]")?;
                    } else {
                        write!(buf, ".[{field_offset}]")?;
                    }
                }
                AnnotatedSborAncestorContainer::EnumVariant {
                    discriminator: variant_discriminator,
                    variant_name,
                    field_index: field_offset,
                    field_name,
                } => {
                    if let Some(variant_name) = variant_name {
                        write!(buf, "::{{{variant_discriminator}|{variant_name}}}")?;
                    } else {
                        write!(buf, "::{{{variant_discriminator}}}")?;
                    }
                    if let Some(field_name) = field_name {
                        write!(buf, ".[{field_offset}|{field_name}]")?;
                    } else {
                        write!(buf, ".[{field_offset}]")?;
                    }
                }
                AnnotatedSborAncestorContainer::Array { index } => {
                    if let Some(index) = index {
                        write!(buf, ".[{index}]")?;
                    }
                }
                AnnotatedSborAncestorContainer::Map { index, entry_part } => {
                    if let Some(index) = index {
                        write!(buf, ".[{index}]")?;
                    }
                    match entry_part {
                        MapEntryPart::Key => write!(buf, ".Key")?,
                        MapEntryPart::Value => write!(buf, ".Value")?,
                    }
                }
            }
        }
        if let Some(AnnotatedSborPartialLeaf {
            name,
            partial_leaf_locator: partial_kinded_data,
        }) = self.annotated_leaf()
        {
            if !is_first {
                write!(buf, "->")?;
            }
            write!(buf, "{}", name.deref())?;
            if let Some(partial_kinded_data) = partial_kinded_data {
                match partial_kinded_data {
                    AnnotatedSborPartialLeafLocator::Tuple {
                        field_offset,
                        field_name,
                    } => {
                        if let Some(field_offset) = field_offset {
                            if let Some(field_name) = field_name {
                                write!(buf, ".[{field_offset}|{field_name}]")?;
                            } else {
                                write!(buf, ".[{field_offset}]")?;
                            }
                        }
                    }
                    AnnotatedSborPartialLeafLocator::EnumVariant {
                        variant_discriminator,
                        variant_name,
                        field_offset,
                        field_name,
                    } => {
                        if let Some(variant_discriminator) = variant_discriminator {
                            if let Some(variant_name) = variant_name {
                                write!(buf, "::{{{variant_discriminator}|{variant_name}}}")?;
                            } else {
                                write!(buf, "::{{{variant_discriminator}}}")?;
                            }
                            if let Some(field_offset) = field_offset {
                                if let Some(field_name) = field_name {
                                    write!(buf, ".[{field_offset}|{field_name}]")?;
                                } else {
                                    write!(buf, ".[{field_offset}]")?;
                                }
                            }
                        }
                    }
                    AnnotatedSborPartialLeafLocator::Array { index } => {
                        if let Some(index) = index {
                            write!(buf, ".[{index}]")?;
                        }
                    }
                    AnnotatedSborPartialLeafLocator::Map { index, entry_part } => {
                        if let Some(index) = index {
                            write!(buf, ".[{index}]")?;
                        }
                        if let Some(entry_part) = entry_part {
                            match entry_part {
                                MapEntryPart::Key => write!(buf, ".Key")?,
                                MapEntryPart::Value => write!(buf, ".Value")?,
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
