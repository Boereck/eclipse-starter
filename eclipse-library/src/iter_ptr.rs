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

use std::ptr;

pub struct PtrIter<T>(*mut T);

/// Warning: do not use for zero-sized types
pub fn iter<T>(ptr: *mut T) -> PtrIter<T> {
    PtrIter(ptr)
}

impl <T> Iterator for PtrIter<T>  {
    type Item = T;
    
    fn next(&mut self) -> Option<T> {
        let result = unsafe { ptr::read(self.0) };
        self.0 = self.0.wrapping_offset(1);
        Some(result)
    }
}