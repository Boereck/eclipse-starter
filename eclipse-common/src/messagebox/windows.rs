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

use crate::native_str::{to_native_str, NativeString};
use std::io::Error;
use std::ptr::null_mut;
use winapi::um::winuser::{MessageBoxW, MB_OK};

///  Display a Message
///
/// This method is called to display an error message to the user before exiting.
/// The method should not return until the user has acknowledged
/// the message. This method may be called before the window
/// system has been initialized. The program should exit after calling this method.
pub fn display_message(message: &str, title: &str) -> Result<(), String> {
    let (_msg_container, msg_ptr) = to_native_str(message);
    let (_title_container, title_ptr) = to_native_str(title);
    message_box(msg_ptr, title_ptr).map_err(|err| format!("{}", err))
}

fn message_box(msg: NativeString, caption: NativeString) -> Result<(), Error> {
    let ret = unsafe { MessageBoxW(null_mut(), msg, caption, MB_OK) };
    if ret == 0 {
        Err(Error::last_os_error())
    } else {
        Ok(())
    }
}
