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
    let win_console = std::env::var("CARGO_FEATURE_WIN_CONSOLE").is_ok();
    // Only specify resource if not windows console version
    if !win_console {
        if let Err(err) = set_resource_info() {
            eprintln!("{}", err);
        }
    }
}

fn set_resource_info() -> Result<(), io::Error>  {
    // On windows build (not console version), set resource info (icon and manifest)
    // TODO: should manifest be set also for console-version? It contains info about DPI awareness.
    if cfg!(target_os = "windows") {
        let mut res = WindowsResource::new();
        res.set_resource_file("res/eclipse.rc");
        res.compile()?;
    }
    Ok(())
}