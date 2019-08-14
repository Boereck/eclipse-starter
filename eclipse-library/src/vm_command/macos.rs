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

use crate::vm_lookup::JvmLaunchMode;
use std::borrow::Cow;

static DEFAULT_ARGS: [&str;1] = ["-XstartOnFirstThread"];

pub fn is_modular_vm(vm_path: &JvmLaunchMode) -> bool {
    unimplemented!()
}

pub fn default_vm_args() -> Vec<Cow<'static,str>> {
    DEFAULT_ARGS.iter().map(|s| Cow::from(*s)).collect()
}