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
 *     Max Bureck (Fraunhofer FOKUS)
 *******************************************************************************/

use std::os::raw::c_int;

#[cfg(not(windows))]
use std::os::raw::c_char;

// On Windows we use UTF-16 chars
#[cfg(windows)]
pub(super) type NativeString = *const u16;

#[cfg(not(windows))]
pub(super) type NativeString = *const c_char;

pub(super) type RunMethod = unsafe extern "C" fn(c_int, *const NativeString, *const NativeString) -> c_int;
pub(super) type SetInitialArgs = unsafe extern "C" fn(c_int, *const NativeString, NativeString) -> ();