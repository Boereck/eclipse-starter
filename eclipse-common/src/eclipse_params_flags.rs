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

//! This module provides the constant flag values of parameters passed to 
//! The launcher.

// Argument names for native launcher, passed on to java launcher
pub const CONSOLE: &str = "-console";
pub const CONSOLELOG: &str = "-consoleLog";
pub const  DEBUG: &str = "-debug";
pub const OS: &str = "-os";
pub const OSARCH: &str = "-arch";
pub const NOSPLASH: &str = "-nosplash";
pub const SHOWSPLASH: &str = "-showsplash";
pub const STARTUP: &str = "-startup";
pub const VM: &str = "-vm";
pub const WS: &str = "-ws";
pub const NAME: &str = "-name";
pub const PROTECT: &str = "-protect";

pub const OPENFILE: &str = "--launcher.openFile";
pub const DEFAULTACTION: &str = "--launcher.defaultAction";
pub const TIMEOUT: &str = "--launcher.timeout";
pub const LIBRARY: &str = "--launcher.library";
pub const SUPRESSERRORS: &str = "--launcher.suppressErrors";
pub const INI: &str = "--launcher.ini";
pub const APPEND_VMARGS: &str = "--launcher.appendVmargs";
pub const OVERRIDE_VMARGS: &str = "--launcher.overrideVmargs";
pub const SECOND_THREAD: &str = "--launcher.secondThread";
pub const PERM_GEN: &str = "--launcher.XXMaxPermSize";
pub const GTK_VERSION: &str = "--launcher.GTK_version";
pub const VMARGS: &str = "-vmargs";

// Only needed for the java launcher
pub const JAR: &str = "-jar";
pub const LAUNCHER: &str = "-launcher";
pub const EXITDATA: &str = "-exitdata";