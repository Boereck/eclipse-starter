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

use eclipse_common::arg_parser::OptionalParam;

#[derive(Default, Debug)]
pub struct EclipseParams {
    /// equivalent to setting osgi.console to [[host:]port].
    pub console: OptionalParam,

    /// equivalent to setting eclipse.consoleLog to "true"
    pub console_log: bool,

    /// equivalent to setting osgi.debug to [options file] or the empty string to simply
    /// enable debug (i.e., if the options file location is not specified)
    pub debug: OptionalParam,

    /// equivalent to setting osgi.os to <operating system>
    pub os: Option<String>,

    /// equivalent to setting osgi.arch to <architecture>
    pub arch: Option<String>,

    /// controls whether or not the splash screen is shown
    pub nosplash: bool,

    /// Specifies the bitmap to use in the splash screen. If specified, the launcher may be able
    /// to show the splash screen before starting the Java VM.  If not specified,
    /// Main will find the bitmap using the osgi.splashLocation and osgi.splashPath properties
    pub showsplash: OptionalParam,

    /// The location of jar used to startup eclipse. The jar referred to should have the Main-Class
    /// attribute set to org.eclipse.equinox.launcher.Main. If this parameter is not set,
    /// the executable will look in the plugins directory for the org.eclipse.equinox.launcher
    /// bundle with the highest version
    pub startup: Option<String>,

    /// When passed to the Eclipse executable, this option is used to locate the Java VM to use to run Eclipse.
    /// It should be the full file system path to an appropriate: Java jre/bin directory, Java Executable,
    /// Java shared library (jvm.dll or libjvm.so), or a Java VM Execution Environment description file.
    /// If not specified, the Eclipse executable uses a search algorithm to locate a suitable VM.
    /// In any event, the executable then passes the path to the actual VM used to Java Main using the -vm argument.
    /// Java Main then stores this value in eclipse.vm.
    pub vm: Option<String>,

    /// Equivalent to setting osgi.ws to <window system>
    pub ws: Option<String>,

    /// The name to be displayed in the task bar item for the splash screen when the application starts up (not applicable on Windows).
    /// Also used as the title of error dialogs opened by the launcher.
    /// When not set, the name is the name of the executable.
    pub name: Option<String>,
    pub protect: Option<String>,
    pub openfile: Option<Vec<String>>,
    pub default_action: Option<String>,
    pub timeout: Option<String>,
    pub suppress_errors: bool,

    /// The location of the eclipse executable's companion shared library.  
    /// If not specified the executable looks in the plugins directory for the appropriate
    /// org.eclipse.equinox.launcher.[platform] fragment with the highest version and uses
    /// the shared library named eclipse_* inside.
    pub library: Option<String>,

    /// The location of the product .ini file to use. If not specified the executable will look
    /// for a file beside the launcher with the same name and the extension .ini.
    /// (i.e. eclipse.exe looks for eclipse.ini, product.exe looks for product.ini)
    pub ini: Option<String>,

    /// If specified, any VM arguments on the commandline will be appended to any VM arguments
    /// specified in the launcher .ini file. Using this option is recommended in every launcher
    /// .ini file that specifies VM arguments, because the default behavior of overriding VM arguments
    /// can have unexpected side-effects.
    pub append_vmargs: bool,
    pub override_vmargs: bool,
    pub second_thread: bool,
    pub perm_gen: bool,
    pub gtk_version: Option<String>,
}

#[derive(Default, Debug)]
pub struct EclipseEEProps {
    pub ee_executable: Option<String>,
    pub ee_console: Option<String>,
    pub ee_vm_libary: Option<String>,
    pub ee_lib_path: Option<Vec<String>>,

    /// Note that this parameter is not read from commandline,
    /// but set by the launcher for VM
    pub ee_filename: String,

    /// This parameter is not read from commandline,
    /// it is the directory in which the ee file is located
    pub ee_home: String,
}
