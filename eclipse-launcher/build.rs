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

use std::io;
use winres::WindowsResource;

fn main() {
    if let Err(err) = set_resource_info() {
        eprintln!("{}", err);
    }
}

fn set_resource_info() -> Result<(), io::Error>  {
    if cfg!(target_os = "windows") {
        let mut res = WindowsResource::new();
        res.set_resource_file("res/eclipse.rc");
        res.compile()?;
    }
    Ok(())
}