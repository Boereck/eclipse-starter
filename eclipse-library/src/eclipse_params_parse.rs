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

//! Parses commandline arguments into an instance of `EclipseParams`

use eclipse_common::arg_parser::{Parser, ListParseStyle};
use super::params::EclipseParams;

// Argument names
static CONSOLE: &str = "-console";
static CONSOLELOG : &str = "-consoleLog";
static DEBUG: &str = "-debug";
static OS: &str = "-os";
static OSARCH: &str = "-arch";
static NOSPLASH: &str = "-nosplash";
static SHOWSPLASH: &str = "-showsplash";
static STARTUP: &str = "-startup";
static VM: &str = "-vm";
static WS: &str = "-ws";
static NAME: &str = "-name";
static PROTECT: &str = "-protect";

static OPENFILE: &str = "--launcher.openFile";
static DEFAULTACTION: &str = "--launcher.defaultAction";
static TIMEOUT: &str = "--launcher.timeout";
static LIBRARY: &str = "--launcher.library";
static SUPRESSERRORS: &str = "--launcher.suppressErrors";
static INI: &str = "--launcher.ini";
static APPEND_VMARGS: &str = "--launcher.appendVmargs";
static OVERRIDE_VMARGS: &str = "--launcher.overrideVmargs";
static SECOND_THREAD: &str = "--launcher.secondThread";
static PERM_GEN: &str = "--launcher.XXMaxPermSize";
static GTK_VERSION: &str = "--launcher.GTK_version";


pub(super) fn parse_args<T : AsRef<str>>(args : &[T]) -> EclipseParams {
    
    let mut parser = Parser::new();
    
    let console_id = parser.add_optional_option(CONSOLE);
    let console_log_id = parser.add_flag(CONSOLELOG);
    let debug_id = parser.add_optional_option(DEBUG);
    let os_id = parser.add_option(OS);
    let arch_id = parser.add_option(OSARCH);
    let nosplash_id = parser.add_flag(NOSPLASH);
    let showsplash_id = parser.add_optional_option(SHOWSPLASH);
    let startup_id = parser.add_option(STARTUP);
    let vm_id = parser.add_option(VM);
    let ws_id = parser.add_option(WS);
    let name_id = parser.add_option(NAME);
    let protect_id = parser.add_option(PROTECT);
    let openfile_id = parser.add_list(OPENFILE, ListParseStyle::UntilDashPrefix);
    let default_action_id = parser.add_option(DEFAULTACTION);
    let timeout_id = parser.add_option(TIMEOUT);
    let suppress_errors_id = parser.add_flag(SUPRESSERRORS);
    let library_id = parser.add_option(LIBRARY);
    let ini_id = parser.add_option(INI);
    let append_vmargs_id = parser.add_flag(APPEND_VMARGS);
    let override_vmargs_id = parser.add_flag(OVERRIDE_VMARGS);
    let second_thread_id = parser.add_flag(SECOND_THREAD);
    let perm_gen_id = parser.add_flag(PERM_GEN);
    let gtk_version_id = parser.add_option(GTK_VERSION);
    
    let iter = args.iter().map(|s| s.as_ref());
    let mut parse_result = parser.parse(iter);
    //TODO adjust paths of openfile
    EclipseParams {
        console: parse_result.take_optional_option(console_id),
        console_log: parse_result.take_flag(console_log_id),
        debug: parse_result.take_optional_option(debug_id),
        os: parse_result.take_option(os_id),
        arch: parse_result.take_option(arch_id),
        nosplash: parse_result.take_flag(nosplash_id),
        showsplash: parse_result.take_optional_option(showsplash_id),
        startup: parse_result.take_option(startup_id),
        vm: parse_result.take_option(vm_id),
        ws: parse_result.take_option(ws_id),
        name: parse_result.take_option(name_id),
        protect: parse_result.take_option(protect_id),
        openfile: parse_result.take_list(openfile_id),
        default_action: parse_result.take_option(default_action_id),
        timeout: parse_result.take_option(timeout_id),
        suppress_errors: parse_result.take_flag(suppress_errors_id),
        library: parse_result.take_option(library_id),
        ini: parse_result.take_option(ini_id),
        append_vmargs: parse_result.take_flag(append_vmargs_id),
        override_vmargs: parse_result.take_flag(override_vmargs_id),
        second_thread: parse_result.take_flag(second_thread_id),
        perm_gen: parse_result.take_flag(perm_gen_id),
        gtk_version: parse_result.take_option(gtk_version_id),
    }
}