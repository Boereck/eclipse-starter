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

#[cfg_attr(not(target_os = "windows"), path = "nix.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod os;
mod common;

pub use common::{SharedMem, SharedMemRef};
use crate::errors::EclipseLibErr;

pub fn create_shared_mem(size: usize) -> Result<impl SharedMem, EclipseLibErr> {
    os::SharedMemOS::create(size)
}

pub fn crete_shared_mem_ref(id: &str) -> Result<impl SharedMemRef, EclipseLibErr> {
    os::SharedMemRefOS::from_id(id)
}