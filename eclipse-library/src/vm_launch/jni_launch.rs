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

use super::{os, StopAction, RESTART_LAST_EC, RESTART_NEW_EC};
use crate::errors::{EclipseLibErr, VmRunErr, VmStartErr};
use crate::shared_mem::SharedMem;
use crate::vm_command::VmArgs;
use dlopen::symbor::{Library, SymBorApi, Symbol};
use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::*;
use jni::sys::{jclass, jint, jmethodID, jobject, jobjectArray, JavaVMOption, JavaVMInitArgs, JNI_VERSION_1_4, JNI_VERSION_1_2, JNI_TRUE, };
use jni::{JNIEnv, JavaVM, };
use std::ffi::c_void;
use std::path::Path;
use std::time::{Duration, Instant};
use std::os::raw::c_char;
use std::convert::TryFrom;
use std::convert::TryInto;

#[derive(WrapperApi)]
struct JvmLibrary {
    #[dlopen_name = "JNI_CreateJavaVM"]
    jni_create_java_vm: unsafe extern "C" fn(
        java_vm: *mut *mut JavaVM,
        env: *mut *mut JNIEnv,
        args: *mut c_void,
    ) -> jint,
}

pub(super) fn launch_jni<S: SharedMem>(
    jni_lib: &Path,
    jar_file: &Path,
    args: &VmArgs<'_>,
    shared_mem: &S,
) -> Result<StopAction, EclipseLibErr> {


    // JNI reflection
    let mainClass: jclass = std::ptr::null_mut(); // The Main class to load
    let mainConstructor: jmethodID = std::ptr::null_mut(); // Main's default constructor Main()
    let mainObject: jobject = std::ptr::null_mut(); // An instantiation of the main class
    let runMethod: jmethodID = std::ptr::null_mut(); // Main.run(String[])
    let methodArgs: jobjectArray = std::ptr::null_mut(); // Arguments to pass to run

    if args.vm_args.is_empty() {
        return Err(VmStartErr::NoVmArgs)?;
    }

    let lib: Container<JvmLibrary> =
        unsafe { Container::load(jni_lib) }.map_err(VmStartErr::VmLoadLibErr)?;

    let mut vm_options: Vec<JavaVMOption> = args.vm_args.iter().map(|s| JavaVMOption {
        optionString: (*s).as_ptr() as *mut c_char, // we assume the JVM is only reading!
        extraInfo: std::ptr::null_mut(),
    }).collect();

    let init_args = JavaVMInitArgs {
        version: if cfg!(target_os="macos") { JNI_VERSION_1_4 } else {JNI_VERSION_1_2},
        options: vm_options.as_mut_ptr(),
        nOptions: args.vm_args.len().try_into().unwrap_or(i32::max_value()),
        ignoreUnrecognized: JNI_TRUE, 
    };
//    lib.jni_create_java_vm(&mut jvm, &mut env, &mut init_args);
    // TODO implement
    unimplemented!();
}

fn get_main_class(env: *mut JNIEnv) -> String {
    
    unimplemented!();
}