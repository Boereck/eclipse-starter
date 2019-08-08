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

use super::common::{SharedMem, SharedMemRef};
use crate::errors::EclipseLibErr;

pub struct SharedMemOS {}

impl SharedMem for SharedMemOS {

    fn create(mem_size: usize) -> Result<Self, EclipseLibErr> {
        unimplemented!()
    }

    fn read(&self) -> Result<String, EclipseLibErr> {
        unimplemented!()
    }

    fn write(&self, s: &str) -> Result<(), EclipseLibErr> {
        unimplemented!()
    }

    fn get_id(&self) -> &str {
        unimplemented!()
    }

    fn close(mut self) -> Result<(), EclipseLibErr> {
        unimplemented!()
    }
}

pub struct SharedMemRefOS{
}

impl SharedMemRef for SharedMemRefOS {
    fn from_id(id: &str) -> Result<Self, EclipseLibErr> {
        unimplemented!()
    }

    fn write(&self, s: &str) -> Result<(), EclipseLibErr> {
        unimplemented!()
    }
}