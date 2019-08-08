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

pub(super) static ECLIPSE_UNITIALIZED: &str = "ECLIPSE_UNITIALIZED";

/// Represents and manages a segment of shared memory. Instantiate such a segment
/// using the `create` method on a concrete implementation type.
/// The `Drop` implementation shall close the shared memory segment if 
/// `close` was not called.
pub trait SharedMem : Sized + Drop {

    /// Creates a `SharedMem` instance managing a shared memory segment
    /// with the specified `mem_size`. To destroy the shared memory,
    /// call the `close` function. Note that dropping the instance will
    /// also close the memory segment, but will give no feedback if 
    /// closing was successful.
    fn create(mem_size: usize) -> Result<Self, EclipseLibErr>;

    fn read(&self) -> Result<String, EclipseLibErr>;

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
    fn from_id(id: &str) -> Result<Self, EclipseLibErr>;

    fn write(&self, s: &str) -> Result<(), EclipseLibErr>;
}
