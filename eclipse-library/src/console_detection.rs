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
 
//! This module provides the method `is_console_launcher` which can detect if the 
//! executable which called the library is a windows console executable.

#[cfg(target_os="windows")]
use winapi::um::wincon; 

#[cfg(target_os = "windows")]
pub fn is_console_launcher() -> bool {
    let console_hwnd = unsafe { wincon::GetConsoleWindow() };
    !console_hwnd.is_null()
}

#[cfg(not(target_os = "windows"))]
pub fn is_console_launcher() -> bool {
    false
}