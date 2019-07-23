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

use gtk::prelude::*;
use gtk::{Window, GtkWindowExt,MessageDialog, ButtonsType, DialogFlags, MessageType};

///  Display a Message
/// 
/// This method is called to display an error message to the user before exiting.
/// The method should not return until the user has acknowledged
/// the message. This method may be called before the window
/// system has been initialized. The program should exit after calling this method.
pub fn display_message(message: &str, title: &str) -> Result<(), String> {
    message_dialog(message, title)
}

fn message_dialog(message: &str, title: &str) -> Result<(), String> {
    let _gtk = gtk::init().map_err(|_| "Initialization of GTK failed.")?;
    let dialog = MessageDialog::new(None::<&Window>,
                       DialogFlags::DESTROY_WITH_PARENT,
                       MessageType::Error,
                       ButtonsType::Close,
                       message);
    dialog.set_title(title);
    dialog.run();
    Ok(())
}