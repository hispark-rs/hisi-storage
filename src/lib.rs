//! Chip-neutral, read-first storage primitives for HiSilicon firmware.
//!
//! This crate deliberately does not own partition tables or persistent formats.
//! It provides bounded byte-addressed storage and the ecosystem-standard
//! [`embedded_storage::ReadStorage`] contract. Erase/write remains outside the
//! stable surface until XIP, cache, interrupt, and power-loss invariants close.

#![no_std]

use core::convert::Infallible;

pub use embedded_storage::ReadStorage;

/// A read-only region backed by a byte slice.
#[derive(Debug, Clone, Copy)]
pub struct SliceStorage<'a> {
    bytes: &'a [u8],
}

impl<'a> SliceStorage<'a> {
    /// Wrap a bounded byte slice as read-only storage.
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Return the complete backing slice.
    pub const fn as_slice(&self) -> &'a [u8] {
        self.bytes
    }
}

impl ReadStorage for SliceStorage<'_> {
    type Error = StorageError<Infallible>;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let start = usize::try_from(offset).map_err(|_| StorageError::OutOfBounds)?;
        let end = start
            .checked_add(bytes.len())
            .filter(|end| *end <= self.bytes.len())
            .ok_or(StorageError::OutOfBounds)?;
        bytes.copy_from_slice(&self.bytes[start..end]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.bytes.len()
    }
}

/// Read-only storage over an explicitly provided memory-mapped region.
#[derive(Debug, Clone, Copy)]
pub struct MemoryMappedStorage<'a> {
    inner: SliceStorage<'a>,
}

impl<'a> MemoryMappedStorage<'a> {
    /// Construct from a previously validated memory-mapped slice.
    pub const fn from_slice(bytes: &'a [u8]) -> Self {
        Self {
            inner: SliceStorage::new(bytes),
        }
    }

    /// Construct a bounded storage region from a raw memory-mapped address.
    ///
    /// # Safety
    ///
    /// `address..address + length` must remain readable for `'a`, must refer to
    /// memory with ordinary byte-read semantics, and must not be concurrently
    /// remapped or made inaccessible. The caller owns the platform XIP/cache
    /// contract.
    pub unsafe fn from_raw_parts(address: *const u8, length: usize) -> Self {
        // SAFETY: delegated to the caller by this function's contract.
        let bytes = unsafe { core::slice::from_raw_parts(address, length) };
        Self::from_slice(bytes)
    }
}

impl ReadStorage for MemoryMappedStorage<'_> {
    type Error = StorageError<Infallible>;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.inner.read(offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}

/// A bounded window into another storage object.
#[derive(Debug)]
pub struct StorageRegion<S> {
    storage: S,
    offset: u32,
    length: usize,
}

impl<S: ReadStorage> StorageRegion<S> {
    /// Create a region, rejecting arithmetic overflow or an out-of-range end.
    pub fn try_new(storage: S, offset: u32, length: usize) -> Result<Self, StorageError<S::Error>> {
        let start = usize::try_from(offset).map_err(|_| StorageError::OutOfBounds)?;
        start
            .checked_add(length)
            .filter(|end| *end <= storage.capacity())
            .ok_or(StorageError::OutOfBounds)?;
        Ok(Self {
            storage,
            offset,
            length,
        })
    }

    /// Return the wrapped storage.
    pub fn into_inner(self) -> S {
        self.storage
    }
}

impl<S: ReadStorage> ReadStorage for StorageRegion<S> {
    type Error = StorageError<S::Error>;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let relative = usize::try_from(offset).map_err(|_| StorageError::OutOfBounds)?;
        relative
            .checked_add(bytes.len())
            .filter(|end| *end <= self.length)
            .ok_or(StorageError::OutOfBounds)?;
        let absolute = self
            .offset
            .checked_add(offset)
            .ok_or(StorageError::OutOfBounds)?;
        self.storage
            .read(absolute, bytes)
            .map_err(StorageError::Backend)
    }

    fn capacity(&self) -> usize {
        self.length
    }
}

/// Storage access failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError<E> {
    /// The requested byte range is outside the selected storage region.
    OutOfBounds,
    /// The underlying storage backend failed.
    Backend(E),
}

#[cfg(test)]
mod tests {
    use super::{ReadStorage, SliceStorage, StorageError, StorageRegion};

    #[test]
    fn slice_storage_checks_bounds() {
        let mut storage = SliceStorage::new(&[1, 2, 3, 4]);
        let mut out = [0; 2];
        storage.read(1, &mut out).unwrap();
        assert_eq!(out, [2, 3]);
        assert_eq!(storage.read(3, &mut out), Err(StorageError::OutOfBounds));
    }

    #[test]
    fn region_translates_offsets_and_checks_its_boundary() {
        let storage = SliceStorage::new(&[0, 1, 2, 3, 4, 5]);
        let mut region = StorageRegion::try_new(storage, 2, 3).unwrap();
        let mut out = [0; 2];
        region.read(1, &mut out).unwrap();
        assert_eq!(out, [3, 4]);
        assert_eq!(region.read(2, &mut out), Err(StorageError::OutOfBounds));
    }
}
