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

//! This module allows reading parameters regarding OS, Architecture, or windowsing system
//! that are either set by the caller of the compiler via environment variable,
//! or detected based on compilation target information given by the compile if not set 
//! by the caller of the compiler.

static DEFAULT_OS: Option<&str> = option_env!("DEFAULT_OS");

static DEFAULT_OS_ARCH: Option<&str> = option_env!("DEFAULT_OS_ARCH");

static DEFAULT_WS: Option<&str> = option_env!("DEFAULT_WS");

// make const as soon as get_default_os and get_default_arch are const
pub fn is_macos_non_x86_64() -> bool {
    get_default_os() == "macosx" && get_default_arch() != "x86_64"
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "windows")]
pub fn get_default_os() -> &'static str {
    DEFAULT_OS.unwrap_or("win32")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "macos")]
pub fn get_default_os() -> &'static str {
    DEFAULT_OS.unwrap_or("macosx")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "linux")]
pub fn get_default_os() -> &'static str {
    DEFAULT_OS.unwrap_or("linux")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_arch = "x86")]
pub fn get_default_arch() -> &'static str {
    DEFAULT_ARCH.unwrap_or("x86")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_arch = "x86_64")]
pub fn get_default_arch() -> &'static str {
    DEFAULT_OS_ARCH.unwrap_or("x86_64")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_arch = "powerpc")]
pub fn get_default_arch() -> &'static str {
    DEFAULT_OS_ARCH.unwrap_or("ppc")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_arch = "arm")]
pub fn get_default_arch() -> &'static str {
    DEFAULT_OS_ARCH.unwrap_or("arm")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_arch = "aarch64")]
pub fn get_default_arch() -> &'static str {
    DEFAULT_OS_ARCH.unwrap_or("aarch64")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "windows")]
pub fn get_default_ws() -> &'static str {
    DEFAULT_WS.unwrap_or("win32")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "macos")]
pub fn get_default_ws() -> &'static str {
    DEFAULT_WS.unwrap_or("cocoa")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "linux")]
pub fn get_default_ws() -> &'static str {
    DEFAULT_WS.unwrap_or("gtk")
}