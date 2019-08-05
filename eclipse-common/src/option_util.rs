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

/// Turns an `&Option<String>` into an `Option<&str>`; this can
/// be useful if the String must not be removed from the soure optional
/// or if an optional should be compared to a static `&str`.
pub fn opt_str(opt: &Option<String>) -> Option<&str> {
    opt.as_ref().map(String::as_str)
}