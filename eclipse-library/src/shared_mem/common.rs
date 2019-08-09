/*******************************************************************************
 * Copyright (c) 2019 Fraunhofer FOKUS and others.
 *
 * This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License 2.0
 * which accompanies this distribution, and is available at
 * https://www.eclipse.org/legal/epl-2.0/
 *
 * SPDX-License-Identifier: EPL-2.0
 *
 * Contributors:
 *     IBM Corporation - Initial C implementation and documentation
 *     Max Bureck (Fraunhofer FOKUS)
 *******************************************************************************/

use crate::errors::EclipseLibErr;

/// Value that should usually be used to pass it as `mem_size` to 
/// `SharedMem::create` or as `max_size` to `SharedMemRef::from_id`.
pub const MAX_SHARED_LENGTH: usize = 16 * 1024;
pub(super) const ECLIPSE_UNITIALIZED: &str = "ECLIPSE_UNITIALIZED";

/// Represents and manages a segment of shared memory. Instantiate such a segment
/// using the `create` method on a concrete implementation type.
/// The `Drop` implementation shall close the shared memory segment if 
/// `close` was not called.
pub trait SharedMem : Sized + Drop {

    /// Creates a `SharedMem` instance managing a shared memory segment
    /// with the specified `mem_size`. To destroy the shared memory,
    /// call the `close` function. Note that dropping the instance will
    /// also close the memory segment, but will give no feedback if 
    /// closing was successful. The value `MAX_SHARED_LENGTH` should usually
    /// be used as `mem_size`.
    /// Creating shared memory may fail, in this case an `Err(EclipseLibErr::SharedMemoryInitFail)`
    /// is returned. If the initialization of the shared memory fails, an
    /// `Err(EclipseLibErr::SharedMemoryWriteFail)` is returned.
    fn create(mem_size: usize) -> Result<Self, EclipseLibErr>;

    /// Reads a null terminated UTF-8 string from shared memory 
    /// and copies the bytes to a new `String` that is returned.
    fn read(&self) -> Result<String, EclipseLibErr>;

    /// Writes the given string `s` as a UTF-8 encoded, null-terminated
    /// string into the managed shared memory. Note that the string will
    /// be truncated if it is longer than `mem_size - 1`, where `mem_size`
    /// is the value that was passed to the `create` method.
    fn write(&self, s: &str) -> Result<(), EclipseLibErr>;

    /// Retruns string of an ID, that can later be used to construct a `SharedMemRef`.
    fn get_id(&self) -> &str;

    /// Consumes the `SharedMem` and closes the shared memory segment. If
    /// closing the segment fails, an `Err(EclipseLibErr::SharedMemoryCloseFail)`
    /// will be returned.
    /// 
    /// Note: if this method is not called before an instance of this trait
    /// is dropped the memory shall be closed by the trait implementation without 
    /// reporting an error.
    fn close(self) -> Result<(), EclipseLibErr>;
}

// This is not creating shared memory, but accessing shared memory from
// An ID created from a `SharedMem`.
pub trait SharedMemRef : Sized {

    /// Creates a `SharedMemRef` from an ID provided by `SharedMem::get_id`.
    /// The value `MAX_SHARED_LENGTH` should usually be used as `max_size`.
    fn from_id(id: &str, max_size: usize) -> Result<Self, EclipseLibErr>;

    /// Writes the given string `s` as a UTF-8 encoded, null-terminated
    /// string into the managed shared memory. Note that the string will
    /// be truncated if it is longer than `max_size - 1`, where `max_size`
    /// is the value that was passed to the `from_id` method.
    fn write(&self, s: &str) -> Result<(), EclipseLibErr>;

    fn close(self) -> Result<(), EclipseLibErr> ;
}
