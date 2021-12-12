use super::Index;
use super::ResizableLimits;
use super::{AsContext, AsContextMut, Store, Stored};
use crate::FuncRef;
use core::fmt;
use core::fmt::Display;

/// A raw index to a table entity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableIdx(usize);

impl Index for TableIdx {
    fn into_usize(self) -> usize {
        self.0
    }

    fn from_usize(value: usize) -> Self {
        Self(value)
    }
}

/// Errors that may occur upon operating with table entities.
#[derive(Debug)]
#[non_exhaustive]
pub enum TableError {
    GrowOutOfBounds {
        maximum: usize,
        current: usize,
        grow_by: usize,
    },
    AccessOutOfBounds {
        current: usize,
        offset: usize,
    },
}

impl Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GrowOutOfBounds {
                maximum,
                current,
                grow_by,
            } => {
                write!(
                    f,
                    "tried to grow table with size of {} and maximum of {} by {} out of bounds",
                    current, maximum, grow_by
                )
            }
            Self::AccessOutOfBounds { current, offset } => {
                write!(
                    f,
                    "out of bounds access of table element {} of table with size {}",
                    offset, current,
                )
            }
        }
    }
}

/// A Wasm table entity.
#[derive(Debug)]
pub struct TableEntity {
    limits: ResizableLimits,
    elements: Vec<Option<FuncRef>>,
}

impl TableEntity {
    /// Creates a new table entity with the given resizable limits.
    pub fn new(limits: ResizableLimits) -> Self {
        Self {
            elements: vec![None; limits.initial()],
            limits,
        }
    }

    /// Returns the resizable limits of the table.
    pub fn limits(&self) -> ResizableLimits {
        self.limits
    }

    /// Returns the current length of the table.
    ///
    /// # Note
    ///
    /// The returned length must be valid within the
    /// resizable limits of the table entity.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Grows the table by the given amount of elements.
    ///
    /// # Note
    ///
    /// The newly added elements are initialized to `None`.
    ///
    /// # Errors
    ///
    /// If the table is grown beyond its maximum limits.
    pub fn grow(&mut self, grow_by: usize) -> Result<(), TableError> {
        let maximum = self.limits.maximum().unwrap_or(u32::MAX as usize);
        let current = self.len();
        let new_len = current
            .checked_add(grow_by)
            .filter(|&new_len| new_len < maximum)
            .ok_or(TableError::GrowOutOfBounds {
                maximum,
                current,
                grow_by,
            })?;
        self.elements.resize(new_len, None);
        Ok(())
    }

    /// Returns the element at the given offset if any.
    ///
    /// # Errors
    ///
    /// If the accesses element is out of bounds of the table.
    pub fn get(&self, offset: usize) -> Result<Option<FuncRef>, TableError> {
        let element = self
            .elements
            .get(offset)
            .cloned() // TODO: change to .copied()
            .ok_or_else(|| TableError::AccessOutOfBounds {
                current: self.len(),
                offset,
            })?;
        Ok(element)
    }

    /// Sets a new value to the table element at the given offset.
    ///
    /// # Errors
    ///
    /// If the accesses element is out of bounds of the table.
    pub fn set(&mut self, offset: usize, new_value: Option<FuncRef>) -> Result<(), TableError> {
        let current = self.len();
        let element = self
            .elements
            .get_mut(offset)
            .ok_or(TableError::AccessOutOfBounds { current, offset })?;
        *element = new_value;
        Ok(())
    }
}

/// A Wasm table reference.
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Table(Stored<TableIdx>);

impl Table {
    /// Creates a new table reference.
    pub(super) fn from_inner(stored: Stored<TableIdx>) -> Self {
        Self(stored)
    }

    /// Returns the underlying stored representation.
    pub(super) fn into_inner(self) -> Stored<TableIdx> {
        self.0
    }

    /// Creates a new table to the store.
    pub fn new<T>(ctx: &mut Store<T>, limits: ResizableLimits) -> Self {
        ctx.alloc_table(TableEntity::new(limits))
    }

    /// Returns the resizable limits of the table.
    pub fn limits(&self, ctx: impl AsContext) -> ResizableLimits {
        ctx.as_context().store.resolve_table(*self).limits()
    }

    /// Returns the current length of the table.
    ///
    /// # Note
    ///
    /// The returned length must be valid within the
    /// resizable limits of the table entity.
    pub fn len(&self, ctx: impl AsContext) -> usize {
        ctx.as_context().store.resolve_table(*self).len()
    }

    /// Grows the table by the given amount of elements.
    ///
    /// # Note
    ///
    /// The newly added elements are initialized to `None`.
    ///
    /// # Errors
    ///
    /// If the table is grown beyond its maximum limits.
    pub fn grow(&mut self, mut ctx: impl AsContextMut, grow_by: usize) -> Result<(), TableError> {
        ctx.as_context_mut()
            .store
            .resolve_table_mut(*self)
            .grow(grow_by)
    }

    /// Returns the element at the given offset if any.
    ///
    /// # Errors
    ///
    /// If the accesses element is out of bounds of the table.
    pub fn get(&self, ctx: impl AsContext, offset: usize) -> Result<Option<FuncRef>, TableError> {
        ctx.as_context().store.resolve_table(*self).get(offset)
    }

    /// Sets a new value to the table element at the given offset.
    ///
    /// # Errors
    ///
    /// If the accesses element is out of bounds of the table.
    pub fn set(
        &mut self,
        mut ctx: impl AsContextMut,
        offset: usize,
        new_value: Option<FuncRef>,
    ) -> Result<(), TableError> {
        ctx.as_context_mut()
            .store
            .resolve_table_mut(*self)
            .set(offset, new_value)
    }
}