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

//! This is a submodule of modul "vm_lookup" providing platform specific functionality.

mod nix;

use crate::params::EclipseParams;
pub use nix::{find_vm_library, console_needed};

static DEFAULT_VM: &str = "java";
pub static VM_LIBRARY: &str = "libjvm.so";
pub static SHIPPED_VM_DIR: &str = "jre/bin/";

pub fn get_default_vm(params: &EclipseParams) -> &'static str {
    DEFAULT_VM
}