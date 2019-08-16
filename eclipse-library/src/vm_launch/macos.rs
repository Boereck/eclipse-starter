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
 
use std::time::Duration;

pub fn program_loop(mut is_term_callback: impl FnMut() -> bool) {
    let dur = Duration::from_millis(100);
    loop {
        // TODO: platform GUI specific stuff
        if is_term_callback() {
            return;
        }
        std::thread::sleep(dur);
    }
}
