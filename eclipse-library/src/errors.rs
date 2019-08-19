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

//! Provides this crate's error type `EclipseLibErr`.

use std::error::Error;
use std::fmt;

/// This is the error type for this crate. All introduced errors shall be
/// a variant of this type.
#[derive(Debug)]
pub enum EclipseLibErr {
    HomeNotFound,
    JvmNotFound(String),
    NoStartupJarFound,
    SharedMemoryInitFail,
    SharedMemoryReadFail,
    SharedMemoryWriteFail,
    SharedMemoryCloseFail,
    SharedMemoryReadInvalidStr,
    SharedMemoryIdParseFail,
    LaunchFail(VmLaunchErr),
}

impl From<VmLaunchErr> for EclipseLibErr {

    fn from(err: VmLaunchErr) -> EclipseLibErr {
        EclipseLibErr::LaunchFail(err)

    }
}

impl fmt::Display for EclipseLibErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: implement
        write!(f, "EclipseLibErr")
    }
}

impl Error for EclipseLibErr {
}

#[derive(Debug)]
pub enum VmLaunchErr {
    StartFail(VmStartErr),
    RunFail(VmRunErr),
}

impl From<VmRunErr> for EclipseLibErr {
    fn from(val: VmRunErr) -> EclipseLibErr {
        EclipseLibErr::LaunchFail(VmLaunchErr::RunFail(val))
    }
}

impl From<VmStartErr> for EclipseLibErr {
    fn from(val: VmStartErr) -> EclipseLibErr {
        EclipseLibErr::LaunchFail(VmLaunchErr::StartFail(val))
    }
}

#[derive(Debug)]
pub enum VmRunErr {
    FailureReturnCode(i32),
    TerminationErr(std::io::Error),
    UnknownErr,
}

#[derive(Debug)]
pub enum VmStartErr {
    ExeStartErr(std::io::Error),
}