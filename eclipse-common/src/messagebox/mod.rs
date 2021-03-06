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

//! This module provides the function `display_message` which can be used
//! to display an error message to the user using a windowing system native
//! to the OS (cocoa on MacOS, win32 on Windows, GTK on Linux).

#[cfg_attr(target_os = "macos", path = "macos/mod.rs")]
#[cfg_attr(target_os = "linux", path = "gtk.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
// TODO support gtk on windows/macos via feature flag?
mod os;

pub use os::display_message;
