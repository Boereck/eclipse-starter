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

mod nix;

pub use nix::find_vm_library;
use crate::params::EclipseParams;

static DEFAULT_VM: &str = "java";
pub static VM_LIBRARY: &str = "JavaVM";
pub static SHIPPED_VM_DIR: &str = "../../jre/Contents/Home/bin/"; // relative to launcher

pub fn get_default_vm(params: &EclipseParams) -> &'static str {
    DEFAULT_VM
}