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
 *     IBM Corporation - original C header
 *     Max Bureck (Fraunhofer FOKUS)
 *******************************************************************************/

use jni::objects::{JObject, JString};
use jni::sys::{jlong, jstring};
use jni::JNIEnv;
use jni_mangle::jni_mangle;
use std::error::Error;
use crate::shared_mem::{crete_shared_mem_ref, SharedMemRef};

/// org_eclipse_equinox_launcher_JNIBridge#_set_exit_data
/// Signature: (Ljava/lang/String;Ljava/lang/String;)V
#[no_mangle]
#[jni_mangle("org.eclipse.equinox.launcher.JNIBridge")]
#[allow(unused_must_use)] // We ignore this, because we cannot do anything if exception suppression fails
pub extern "system" fn set_exit_data(env: JNIEnv, object: JObject, id: JString, s: JString) {
    let result = set_exit_data_internal(&env, object, id, s);
    // we do not want to throw on Java side
    if result.is_err() {
        env.exception_describe();
        env.exception_clear();
    }
}

fn set_exit_data_internal(env: &JNIEnv, _object: JObject, id: JString, s: JString) -> Result<(),Box<dyn Error>> {
    let s_jstr = env.get_string(s)?;
    let s_rstr = s_jstr.to_str()?;
    
    let id_jstr = env.get_string(id)?;
    let id_rstr = id_jstr.to_str()?;
    
    crete_shared_mem_ref(id_rstr)?.write(s_rstr)?;
    Ok(())
}

/// org_eclipse_equinox_launcher_JNIBridge#_set_launcher_info
/// Signature: (Ljava/lang/String;Ljava/lang/String;)V
#[no_mangle]
#[jni_mangle("org.eclipse.equinox.launcher.JNIBridge")]
pub extern "system" fn set_launcher_info(
    env: JNIEnv,
    object: JObject,
    launcher: JString,
    name: JString,
) {
}

/// org_eclipse_equinox_launcher_JNIBridge#_update_splash
/// Signature: ()V
#[no_mangle]
#[jni_mangle("org.eclipse.equinox.launcher.JNIBridge")]
pub extern "system" fn update_splash(env: JNIEnv, object: JObject) -> () {}

/// org_eclipse_equinox_launcher_JNIBridge#_get_splash_handle
/// Signature: ()J
#[no_mangle]
#[jni_mangle("org.eclipse.equinox.launcher.JNIBridge")]
pub extern "system" fn get_splash_handle(env: JNIEnv, object: JObject) -> jlong {
    0
}

/// org_eclipse_equinox_launcher_JNIBridge#_show_splash
/// Signature: (Ljava/lang/String;)V
#[no_mangle]
#[jni_mangle("org.eclipse.equinox.launcher.JNIBridge")]
pub extern "system" fn show_splash(env: JNIEnv, obj: JObject, s: JString) {}

/// org_eclipse_equinox_launcher_JNIBridge#_takedown_splash
/// Signature: ()V
#[no_mangle]
#[jni_mangle("org.eclipse.equinox.launcher.JNIBridge")]
pub extern "system" fn takedown_splash(env: JNIEnv, object: JObject) {}

/// org_eclipse_equinox_launcher_JNIBridge#_get_os_recommended_folder
/// Signature: ()Ljava/lang/String
#[no_mangle]
#[jni_mangle("org.eclipse.equinox.launcher.JNIBridge")]
pub extern "system" fn get_os_recommended_folder(env: JNIEnv, object: JObject) -> jstring {
    if cfg!(target_os = "macos") {
        // TODO: call env.new_string(getFolderForApplicationData()).into_inner()
    }
    std::ptr::null_mut()
}
