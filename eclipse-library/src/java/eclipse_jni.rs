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
use jni::sys::{jstring, jlong};
use jni::JNIEnv;
use jni_mangle::jni_mangle;

/// org_eclipse_equinox_launcher_JNIBridge#_set_exit_data
/// Signature: (Ljava/lang/String;Ljava/lang/String;)V
#[no_mangle]
#[jni_mangle("org.eclipse.equinox.launcher.JNIBridge")]
pub extern "system" fn set_exit_data(env: JNIEnv, object: JObject, id: JString, s: JString) -> () {}

/// org_eclipse_equinox_launcher_JNIBridge#_set_launcher_info
/// Signature: (Ljava/lang/String;Ljava/lang/String;)V
#[no_mangle]
#[jni_mangle("org.eclipse.equinox.launcher.JNIBridge")]
pub extern "system" fn set_launcher_info(env: JNIEnv, object: JObject, launcher: JString, name: JString) -> () {}

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
pub extern "system" fn show_splash(env: JNIEnv, obj: JObject, s: JString) ->() {}

/// org_eclipse_equinox_launcher_JNIBridge#_takedown_splash
/// Signature: ()V
#[no_mangle]
#[jni_mangle("org.eclipse.equinox.launcher.JNIBridge")]
pub extern "system" fn takedown_splash(env: JNIEnv, object: JObject) -> () {}

/// org_eclipse_equinox_launcher_JNIBridge#_get_os_recommended_folder
/// Signature: ()Ljava/lang/String
#[no_mangle]
#[jni_mangle("org.eclipse.equinox.launcher.JNIBridge")]
pub extern "system" fn get_os_recommended_folder(env: JNIEnv, object: JObject)-> jstring {
    unimplemented!()
}


