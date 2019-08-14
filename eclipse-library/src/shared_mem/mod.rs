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

mod common;
#[cfg_attr(not(target_os = "windows"), path = "nix.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod os;

use crate::errors::EclipseLibErr;
pub use common::{SharedMem, SharedMemRef, MAX_SHARED_LENGTH};

/// Creates a new instance of an operating system specific `SharedMem` instance.
/// Creating shared memory may fail, in this case an `Err(EclipseLibErr::SharedMemoryInitFail)`
/// is returned. If the initialization of the shared memory fails, an
/// `Err(EclipseLibErr::SharedMemoryWriteFail)` is returned.
pub fn create_shared_mem(size: usize) -> Result<impl SharedMem, EclipseLibErr> {
    os::SharedMemOS::create(size)
}

/// Creates a new instance of an operating system specific `SharedMemRef` instance,
/// which accesses the memory previously created using a `SharedMem`.
/// The `id` parameter represents a reference to an existing shared memory. The `id`
/// can be obtained from an existing `SharedMem`, using the `SharedMem::get_id` function.
/// If it is not possible to access memory from the given `id` an `Err(SharedMemoryIdParseFail)`
/// is returned.
pub fn crete_shared_mem_ref(id: &str, max_size: usize) -> Result<impl SharedMemRef, EclipseLibErr> {
    os::SharedMemRefOS::from_id(id, max_size)
}

// Currently implementation only done for windows
#[cfg(all(test, target_os = "windows"))]
mod test {

    use super::*;
    use crate::errors::EclipseLibErr;

    #[test]
    fn test_write_sharedmem_read_sharedmem_same_process() -> Result<(), EclipseLibErr> {
        let s = "Löwe 老虎 Léopard";
        let max_size = s.len() + 1;
        let shared_data = create_shared_mem(max_size)?;
        shared_data.write(s)?;
        let result = shared_data.read()?;
        shared_data.close()?;
        assert_eq!(s, result);
        Ok(())
    }

    #[test]
    fn test_write_sharedmem_read_sharedmem_same_process_trucated() -> Result<(), EclipseLibErr> {
        let s = "Fooo";
        let max_size = s.len();
        let shared_data = create_shared_mem(max_size)?;
        shared_data.write(s)?;
        let result = shared_data.read()?;
        shared_data.close()?;
        assert_eq!("Foo", result);
        Ok(())
    }

    #[test]
    fn test_write_sharedmemref_read_sharedmem_same_process() -> Result<(), EclipseLibErr> {
        let s = "Löwe 老虎 Léopard";
        let max_size = s.len() + 1;
        let shared_data = create_shared_mem(max_size)?;
        let id = shared_data.get_id();

        let shared_ref = crete_shared_mem_ref(id, max_size)?;
        shared_ref.write(s)?;
        let result = shared_data.read()?;

        shared_ref.close()?;
        shared_data.close()?;

        assert_eq!(s, result);
        Ok(())
    }

    #[test]
    fn test_write_sharedmemref_read_sharedmem_same_process_truncated() -> Result<(), EclipseLibErr>
    {
        let s = "Fooo";
        let max_size = s.len();
        let shared_data = create_shared_mem(max_size)?;
        let id = shared_data.get_id();

        let shared_ref = crete_shared_mem_ref(id, max_size)?;
        shared_ref.write(s)?;
        let result = shared_data.read()?;

        shared_ref.close()?;
        shared_data.close()?;
        assert_eq!("Foo", result);
        Ok(())
    }

}
