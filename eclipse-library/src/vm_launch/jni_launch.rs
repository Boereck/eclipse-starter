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

use super::common::StringHolder;
use super::os::to_string_holder;
use super::{os, StopAction, RESTART_LAST_EC, RESTART_NEW_EC};
use crate::eclipse_jni::*;
use crate::errors::{EclipseLibErr, VmRunErr, VmStartErr};
use crate::shared_mem::SharedMem;
use crate::vm_command::VmArgs;
use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::*;
use eclipse_common::native_str::to_native_str;
use eclipse_common::path_util::strip_unc_prefix;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys;
use jni::sys::{
    jclass, jint, jmethodID, jobject, jobjectArray, JNINativeMethod, JavaVMInitArgs, JavaVMOption,
    JNI_OK, JNI_TRUE, JNI_VERSION_1_2, JNI_VERSION_1_4,
};
use jni::{JNIEnv, JavaVM};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::ffi::c_void;
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(WrapperApi)]
struct JvmLibrary {
    /// Creates a JVM and returns `JNI_OK` if the operation succeeds.
    /// This API is made typesafe, since the original definition
    /// defines `void` pointers for `env` and `args` [see here](https://docs.oracle.com/javase/9/docs/specs/jni/invocation.html#jni_createjavavm)
    #[dlopen_name = "JNI_CreateJavaVM"]
    jni_create_java_vm: unsafe extern "C" fn(
        java_vm: *mut *mut sys::JavaVM,
        env: *mut *mut c_void, // is always sys::JNIEnv
        args: *mut c_void,     // is always JavaVMInitArgs
    ) -> jint,
}

pub(super) fn launch_jni<'a, 'o, S: SharedMem>(
    jni_lib: &Path,
    jar_file: &Path,
    args: &'a VmArgs<'a>,
    shared_mem: &S,
) -> Result<StopAction, EclipseLibErr>
where
    'a: 'o,
{
    let mut jvm: *mut sys::JavaVM = std::ptr::null_mut();
    let mut env_raw: *mut sys::JNIEnv = std::ptr::null_mut();

    if args.vm_args.is_empty() {
        return Err(VmStartErr::NoVmArgs)?;
    }

    let jni_lib_str = jni_lib.to_string_lossy();
    let jni_lib_stripped = strip_unc_prefix(&jni_lib_str);
    let lib: Container<JvmLibrary> =
        unsafe { Container::load(jni_lib_stripped) }.map_err(VmStartErr::VmLoadLibErr)?;
    
    let vm_args_ref: &'a _ = &args.vm_args;
    let arg_strs_itr = vm_args_ref.iter().map(|s| -> &'a str { &s });
    let platform_strs_holder = to_string_holder::<'a, 'o>(arg_strs_itr);
	
	// TODO remove when code below works
	Err(VmStartErr::NoVmArgs)?

    // Block only needed to reduce lifetime scope of refernece
//    let platform_strs_holder_ref: &'o _ = &platform_strs_holder;
//    let mut vm_options: Vec<JavaVMOption> = platform_strs_holder_ref
//        .get_strings()
//        .map(|s_ptr| JavaVMOption {
//            optionString: s_ptr, // we assume the JVM is only reading!
//            extraInfo: std::ptr::null_mut(),
//        })
//        .collect();
//
//    let mut init_args = JavaVMInitArgs {
//        version: if cfg!(target_os = "macos") {
//            JNI_VERSION_1_4
//        } else {
//            JNI_VERSION_1_2
//        },
//        options: vm_options.as_mut_ptr(),
//        nOptions: vm_options.len().try_into().unwrap_or(i32::max_value()),
//        ignoreUnrecognized: JNI_TRUE,
//    };
//
//    let jvm_ptr: *mut *mut sys::JavaVM = &mut jvm;
//    let env_raw_ptr: *mut *mut sys::JNIEnv = &mut env_raw;
//    let init_args_ptr: *mut JavaVMInitArgs = &mut init_args;
//    let vm_create_result = unsafe {
//        lib.jni_create_java_vm(
//            jvm_ptr,
//            env_raw_ptr as *mut *mut c_void,
//            init_args_ptr as *mut c_void,
//        )
//    };
//
//    if vm_create_result != JNI_OK {
//        Err(VmStartErr::CreateVmErr)?;
//    }
//
//    let env: JNIEnv<'a> =
//        unsafe { JNIEnv::from_raw(env_raw) }.map_err(|_| VmStartErr::CreateVmErr)?;
//    register_natives(&env, env_raw);
//
//    let main_class = get_main_class(&env, jar_file)
//        .or_else(|| {
//            // fallback to hardcoded name
//            clear_exception(&env);
//            env.find_class("org/eclipse/equinox/launcher/Main").ok()
//        })
//        .ok_or(VmStartErr::MainClassNotFound)?;
//
//    let ctor_args = [];
//    let main_obj = env
//        .new_object(main_class, "<init>", &ctor_args)
//        .map_err(|_| VmStartErr::RunMethodNotInvokable)?;
//
//    let run_args = create_run_args(&env, args)?;
//
//    let run_result = env
//        .call_method(main_obj, "run", "([Ljava/lang/String;)I", &run_args)
//        .map_err(|_| VmStartErr::RunMethodNotInvokable)?;
//
//    // TODO: port (*env)->DeleteLocalRef(env, methodArgs);
//    clear_exception(&env);
//
//    match run_result {
//        JValue::Int(return_value) => result_from_jni_exit_code(return_value, shared_mem),
//        _ => Err(VmRunErr::UnexpectedReturnValue.into()),
//    }
}

fn result_from_jni_exit_code<S: SharedMem>(
    return_code: i32,
    shared_mem: &S,
) -> Result<StopAction, EclipseLibErr> {
    unimplemented!();
}

fn get_main_class<'a>(env: &JNIEnv<'a>, jar_file: &Path) -> Option<JClass<'a>> {
    unimplemented!();
}

fn register_natives(env: &JNIEnv, env_raw: *mut sys::JNIEnv) {
    // TODO can we somehow have a nicer way to refecence the methods?
    // procedural macros can currently not expand to expressions
    let update_splash: extern "system" fn(JNIEnv, JObject) =
        Java_org_eclipse_equinox_launcher_JNIBridge__1update_1splash;
    let update_splash_name = CString::new("_update_splash").unwrap_or_default();
    let update_splash_sig = CString::new("()V").unwrap_or_default();

    let mut natives: Vec<JNINativeMethod> = vec![
        JNINativeMethod {
            fnPtr: update_splash as *mut c_void,
            name: to_mut_ptr(&update_splash_name),
            signature: to_mut_ptr(&update_splash_sig),
        },
        // TODO other functions!
    ];

    let env_ref = &unsafe { **env_raw };
    let find_class = match env_ref.FindClass {
        Some(find_class) => find_class,
        _ => return,
    };
    let bridge_name = CString::new("org/eclipse/equinox/launcher/JNIBridge").unwrap_or_default();
    let bridge: jclass = unsafe { find_class(env_raw, bridge_name.as_ptr()) };
    if let Some(reg_natives) = env_ref.RegisterNatives {
        let len = natives.len().try_into().unwrap_or(0);
        let reg_res = unsafe {
            reg_natives(env_raw, bridge, natives.as_mut_ptr(), len);
        };
    }
    clear_exception(&env);
}

fn to_mut_ptr(s: &CString) -> *mut i8 {
    s.as_ptr() as *mut i8
}

#[allow(unused_must_use)] // we don't care if these ops fail
fn clear_exception(env: &JNIEnv) {
    if env.exception_occurred().is_ok() {
        env.exception_describe();
        env.exception_clear();
    }
}

fn create_run_args<'a>(
    env: &JNIEnv<'a>,
    args: &VmArgs<'_>,
) -> Result<Vec<JValue<'a>>, EclipseLibErr> {
    unimplemented!();
}
