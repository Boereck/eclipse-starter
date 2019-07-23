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

#[allow(non_snake_case)]
mod nsalert;

use nsalert::NSAlert;
use cocoa::base::{id, nil};
use cocoa::foundation::NSString;

///  Display a Message
///
/// This method is called to display an error message to the user before exiting.
/// The method should not return until the user has acknowledged
/// the message. This method may be called before the window
/// system has been initialized. The program should exit after calling this method.
pub fn display_message(message: &str, title: &str) -> Result<(), String> {
    unsafe {
        let alert = NSAlert::alloc(nil).init().autorelease();
        alert.setAlertStyle(NSAlertStyle::critical);
        alert.setMessageText(NSString::alloc(nil).init_str(title));
        alert.setInformativeText(NSString::alloc(nil).init_str(message));
        alert.runModal();
    }
    Ok(())
 }