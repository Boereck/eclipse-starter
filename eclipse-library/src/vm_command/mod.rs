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

#[cfg_attr(target_os = "macos", path = "macos.rs")]
#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod os;

use crate::params::EclipseParams;
use crate::vm_lookup::JvmLaunchMode;
use core::cmp::min;
use eclipse_common::arg_parser::OptionalParam;
use eclipse_common::eclipse_params_flags::*;
use eclipse_common::path_util::strip_unc_prefix;
use os::{default_vm_args, is_modular_vm};
use std::borrow::Cow;
use std::path::Path;

const CLASSPATH_PREFIX: &str = "-Djava.class.path=";
const ADDMODULES: &str = "--add-modules";

#[derive(Debug)]
pub struct VmArgs<'e> {
    pub vm_args: Vec<Cow<'e, str>>,
    pub program_args: Vec<Cow<'e, str>>,
}

/// Get the command and arguments to start the Java VM.
pub fn get_vm_command<'a, 'b>(
    launch_mode: &'a JvmLaunchMode,
    args: &'a [&str],
    user_vm_args: &'a [Cow<'a, str>],
    jar_file: &'a Path,
    params: &'a EclipseParams,
    exitdata: &'a str,
    program_path: &'a Path,
) -> VmArgs<'b>
where
    'a: 'b,
{
    // Collect resulting JVM arguments first

    let mut vm_args: Vec<Cow<'b, str>> = if user_vm_args.is_empty() {
        default_vm_args()
    } else {
        user_vm_args.to_owned()
    };

    // the startup jarFile goes on the classpath
    let jar_file_cow = jar_file.to_string_lossy();
    let jar_file_str = strip_unc_prefix(&jar_file_cow);
    // TODO: maybe use smallvec here for a stack allocated vector
    let classpath_vars: Vec<Cow<'_, str>> = match launch_mode {
        // JNI launching, classpath is set using -Djava.class.path
        JvmLaunchMode::LaunchJni { .. } => {
            let cp_param = CLASSPATH_PREFIX.to_string() + jar_file_str;
            vec![cp_param.into()]
        }
        JvmLaunchMode::LaunchExe { .. } => vec![JAR.into(), jar_file_str.to_string().into()],
    };

    adjust_vm_args(&launch_mode, &mut vm_args);
    // if the user specified a classpath, skip it
    let classpath_pos_opt = vm_args
        .iter()
        .position(|ref s| s == &"-classpath" || s == &"-cp");
    if let Some(classpath_pos) = classpath_pos_opt {
        vm_args.drain(classpath_pos..=classpath_pos + 1);
    }

    // Add ee vm arguments
    let add_vm_args = match launch_mode {
        JvmLaunchMode::LaunchJni { add_vm_args, .. } => add_vm_args,
        JvmLaunchMode::LaunchExe { add_vm_args, .. } => add_vm_args,
    };

    let capacity = vm_args.len() + add_vm_args.len() + classpath_vars.len();
    let mut result_vm_args: Vec<Cow<'b, str>> = Vec::with_capacity(capacity);

    result_vm_args.extend_from_slice(&vm_args);
    let add_vm_args_iter = add_vm_args.iter().map(Into::into);
    // Add additional args as Cow<'_,str>
    result_vm_args.extend(add_vm_args_iter);
    result_vm_args.extend_from_slice(&classpath_vars);

    // Now collect resulting program arguments
    let result_program_arts = get_program_args(
        args,
        &result_vm_args,
        params,
        exitdata,
        program_path,
        launch_mode,
    );

    VmArgs {
        vm_args: result_vm_args,
        program_args: result_program_arts,
    }
}

fn get_program_args<'a, 'b, 'c, S: AsRef<str>>(
    args: &'a [S],
    vm_args: &'c [Cow<'b, str>],
    params: &'a EclipseParams,
    exitdata: &'a str,
    program_path: &'a Path,
    launch_mode: &'a JvmLaunchMode,
) -> Vec<Cow<'b, str>>
where
    'a: 'b,
{
    // Count capacity needed. We do so by referencing parameters that are used
    // (so build fails if parameters go away), but in const fns, so all computation
    // can be done at build time.
    let result_prog_args_capacity = opt_opt_param_count(&params.showsplash)
        + param_count(&program_path)
        + opt_param_count(&params.ws)
        + opt_param_count(&params.arch)
        + opt_param_count(&params.name)
        + opt_param_count(&params.library)
        + opt_param_count(&params.os)
        + param_count(&exitdata)
        + opt_param_count(&params.startup)
        + opt_param_count(&params.vm)
        + opt_opt_param_count(&params.console)
        + opt_opt_param_count(&params.debug)
        + opt_flag_count(params.console_log)
        + opt_flag_count(params.append_vmargs) // or override
        + args.len()
        + flag_count(VM)
        + vm_args.len();
    let mut result_program_params: Vec<Cow<'b, str>> =
        Vec::with_capacity(result_prog_args_capacity);

    // showsplash
    use OptionalParam::*;
    match &params.showsplash {
        SetNoVal => result_program_params.push(SHOWSPLASH.into()),
        Set(ref s) => {
            result_program_params.push(SHOWSPLASH.into());
            result_program_params.push(s.into());
        }
        _ => {}
    }

    // Most optional parameters are set by now, but we refrain from unwrapping

    // Append the launcher command
    result_program_params.push(LAUNCHER.into());
    let program_str = strip_unc_prefix(&program_path.to_string_lossy()).to_string();
    result_program_params.push(program_str.into());

    // Append the name command
    if let Some(name) = params.name.as_ref() {
        result_program_params.push(NAME.into());
        result_program_params.push(name.into());
    }

    // And the shared library
    if let Some(ref lib) = params.library {
        result_program_params.push(LIBRARY.into());
        result_program_params.push(lib.into());
    }

    // The startup jar
    if let Some(ref startup) = params.startup {
        result_program_params.push(STARTUP.into());
        result_program_params.push(startup.into());
    }

    // Protect mode
    if let Some(ref protect) = params.protect {
        result_program_params.push(PROTECT.into());
        result_program_params.push(protect.into());
    }

    // Override or append vm args
    if params.append_vmargs {
        result_program_params.push(APPEND_VMARGS.into());
    } else {
        result_program_params.push(OVERRIDE_VMARGS.into());
    }

    // Append the exit data command.
    result_program_params.push(EXITDATA.into());
    result_program_params.push(exitdata.into());

    // The following three parameters are used by the launcher, but are also
    // delegated to the java program

    // Append the console command, if set add value
    match &params.console {
        SetNoVal => result_program_params.push(CONSOLE.into()),
        Set(param_val) => {
            result_program_params.push(CONSOLE.into());
            result_program_params.push(param_val.into());
        },
        _ => {}
    }

    // Append the console command, if set add value
    match &params.debug {
        SetNoVal => result_program_params.push(DEBUG.into()),
        Set(param_val) => {
            result_program_params.push(DEBUG.into());
            result_program_params.push(param_val.into());
        },
        _ => {}
    }

    if params.console_log {
            result_program_params.push(CONSOLELOG.into());
    }

    // Append the remaining user defined arguments.
    result_program_params.extend(args.iter().map(AsRef::as_ref).map(Into::into));

    // Append VM and VMARGS to be able to relaunch using exit data.
    result_program_params.push(VM.into());
    let vm_location: &'a Path = match launch_mode {
        JvmLaunchMode::LaunchJni { jni_lib, .. } => jni_lib,
        JvmLaunchMode::LaunchExe { exe, .. } => exe,
    };
    let vm_cow = vm_location.to_string_lossy();
    let vm_str = strip_unc_prefix(&vm_cow).to_string();
    result_program_params.push(vm_str.into());

    result_program_params.push(VMARGS.into());
    // cloning of Cows can be expensive for owned variants,
    // but up until now we have at most one owned vm_arg
    // (classpath def on JvmLaunchMode::LaunchExe)
    let vm_arg_iter = vm_args.iter().map(Cow::clone);
    result_program_params.extend(vm_arg_iter);

    result_program_params
}

const fn opt_param_count(_opt: &Option<String>) -> usize {
    2
}

const fn param_count<T>(_param: &T) -> usize {
    2
}

const fn opt_opt_param_count(_opt: &OptionalParam) -> usize {
    2
}

const fn flag_count(_flag: &str) -> usize {
    1
}

const fn opt_flag_count(_opt: bool) -> usize {
    1
}

fn adjust_vm_args(launch_mode: &JvmLaunchMode, vm_args: &mut Vec<Cow<'_, str>>) {
    // JVMs whose version is >= 9 need an extra VM argument (--add-modules) to start eclipse but earlier versions
    // do not recognize this argument, remove it from the list of VM arguments when the JVM version is below 9

    // skipping java 9 param removal on MacOS shared libraries
    // TODO: is this behavior correct? The C version only skipps if
    // library was set via -vm option, not if library vm was detected and not specified
    if cfg!(target_os = "macos") {
        if let JvmLaunchMode::LaunchJni { .. } = launch_mode {
            return;
        }
    }
    if !is_modular_vm(launch_mode) {
        remove_modular_vm_args(vm_args);
    }
}

fn remove_modular_vm_args(vm_args: &mut Vec<Cow<'_, str>>) {
    // remove --add-modules arguments
    let mut skip: usize = 0;
    while let Some(index) = vm_args
        .iter()
        .skip(skip)
        .position(|s| s.starts_with(ADDMODULES))
    {
        let abs_index = index + skip;
        skip = abs_index; // since we're about to remove element at abs_index, start next iteration at this index
        let mod_arg = &vm_args[abs_index];
        if mod_arg.contains('=') {
            // --add-modules=<value>
            // only remove this parameter
            vm_args.remove(abs_index);
        } else if mod_arg == ADDMODULES {
            let args_len = vm_args.len();
            // --add-modules <value> OR --add-modules <end-of-vm_args>
            let upper_index = min(abs_index + 1, args_len - 1);
            vm_args.drain(abs_index..=upper_index);
        } else {
            // Probable new argument e.g. --add-modules-if-required or misspelled argument e.g. --add-modulesq
            skip += 1; // we skip this element which remains in the vec
        }
    }
}

#[cfg(test)]
mod test {
    use super::remove_modular_vm_args;
    use super::ADDMODULES;
    use std::borrow::Cow;

    macro_rules! vec_into {
        ($($e:expr,)*) => ( vec![$($e.into(),)*] )
    }

    #[test]
    fn test_remove_modular_vm_args_nothing_to_remove() {
        let mut args: Vec<Cow<'_, str>> = vec_into![
            "-cp",
            "myjar.jar",
            "-Dosgi.requiredJavaVersion=1.8",
            "-XX:+UseG1GC",
        ];
        let expected = args.clone();
        remove_modular_vm_args(&mut args);
        assert_eq!(args, expected);
    }

    #[test]
    fn test_remove_modular_vm_args_remove_combined_middle() {
        let modules = ADDMODULES.to_string() + "=ALL-SYSTEM";
        let mut args: Vec<Cow<'_, str>> = vec_into![
            "-cp",
            "myjar.jar",
            "-Dosgi.requiredJavaVersion=1.8",
            "-XX:+UseG1GC",
        ];
        let expected = args.clone();
        args.insert(2, modules.into());
        remove_modular_vm_args(&mut args);
        assert_eq!(args, expected);
    }

    #[test]
    fn test_remove_modular_vm_args_remove_combined_last() {
        let modules = ADDMODULES.to_string() + "=ALL-SYSTEM";
        let mut args: Vec<Cow<'_, str>> = vec_into![
            "-cp",
            "myjar.jar",
            "-Dosgi.requiredJavaVersion=1.8",
            "-XX:+UseG1GC",
            modules,
        ];
        let upper = args.len() - 1;
        let expected: Vec<Cow<'_, str>> = args[..upper].iter().map(Clone::clone).collect();
        remove_modular_vm_args(&mut args);
        assert_eq!(args, expected);
    }

    #[test]
    fn test_remove_modular_vm_args_remove_separate_middle() {
        let mut args: Vec<Cow<'_, str>> =
            vec_into!["-cp", "myjar.jar", ADDMODULES, "ALL-SYSTEM", "-XX:+UseG1GC",];
        let mut expected = args.clone();
        expected.drain(2..4);
        remove_modular_vm_args(&mut args);
        assert_eq!(args, expected);
    }

    #[test]
    fn test_remove_modular_vm_args_remove_separate_last() {
        let mut args: Vec<Cow<'_, str>> =
            vec_into!["-cp", "myjar.jar", "-XX:+UseG1GC", ADDMODULES, "ALL-SYSTEM",];
        let mut expected = args.clone();
        expected.drain(3..5);
        remove_modular_vm_args(&mut args);
        assert_eq!(args, expected);
    }

    #[test]
    fn test_remove_modular_vm_args_remove_multiple() {
        let modules = ADDMODULES.to_string() + "=ALL-SYSTEM";
        let mut args: Vec<Cow<'_, str>> = vec_into![
            "-cp",
            "myjar.jar",
            modules,
            "-XX:+UseG1GC",
            ADDMODULES,
            "ALL-SYSTEM",
        ];
        let mut expected = args.clone();
        expected.drain(4..6);
        expected.remove(2);
        remove_modular_vm_args(&mut args);
        assert_eq!(args, expected);
    }

    #[test]
    fn test_remove_modular_vm_args_keep_unknown_with_addmodule_prefix() {
        let mut args: Vec<Cow<'_, str>> = vec_into![
            "-cp",
            "myjar.jar",
            "-Dosgi.requiredJavaVersion=1.8",
            "--add-modules-if-required",
            "-XX:+UseG1GC",
        ];
        let expected = args.clone();
        remove_modular_vm_args(&mut args);
        assert_eq!(args, expected);
    }

    #[test]
    fn test_remove_modular_vm_args_keep_multiple_unknown_with_addmodule_prefix() {
        let mut args: Vec<Cow<'_, str>> = vec_into![
            "-cp",
            "myjar.jar",
            "-Dosgi.requiredJavaVersion=1.8",
            "--add-modules-if-required",
            "-XX:+UseG1GC",
            "--add-modulesp",
        ];
        let expected = args.clone();
        remove_modular_vm_args(&mut args);
        assert_eq!(args, expected);
    }
}
