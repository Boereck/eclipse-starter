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
 *     IBM Corporation - Initial C implementation and documentation
 *     Max Bureck (Fraunhofer FOKUS)
 *******************************************************************************/

//! This module implements showing/updating/termination the splash screen.

use crate::errors::EclipseLibErr;
use lazy_static::lazy_static;
use std::sync::{LockResult, Mutex, MutexGuard};

lazy_static! {
    static ref splash_singleton: Mutex<Option<OsSplash>> = Mutex::default();
}

pub trait Splash {
    fn get_handle(&self) -> u64;

    fn show_image(&mut self);

    fn update(&self);

    fn take_down(self);
}

//fn get_or_create_splash() -> Result<impl Splash, EclipseLibErr> {
//    // TODO implement here
//    unimplemented!();
//}

// TODO create OS specific versions
struct OsSplash {}
