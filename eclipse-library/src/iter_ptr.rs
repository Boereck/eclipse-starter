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

//! This module allows (endless) iteration over raw pointers,
//! which will move the pointer forward in steps of the size of the element pointed to.

use std::ptr;

pub struct PtrIter<T>(*mut T);

/// Creates an iterator, starting at the element the given `ptr`
/// is pointed to. The returned iterator is unbound simply moving
/// the pointer to the next <T> element in memory.
///
/// Warning: do not use for zero-sized types
pub fn iter<T>(ptr: *mut T) -> PtrIter<T> {
    PtrIter(ptr)
}

impl<T> Iterator for PtrIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let result = unsafe { ptr::read(self.0) };
        self.0 = self.0.wrapping_offset(1);
        Some(result)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn testIter() {
        let mut elements = [42, 38, 4711, 0];
        let el_ptr = elements.as_mut_ptr();
        let mut iter = super::iter(el_ptr);
        let first = iter.next().unwrap();
        assert_eq!(42, first);
        let second = iter.next().unwrap();
        assert_eq!(38, second);
        let third = iter.next().unwrap();
        assert_eq!(4711, third);
        let fourth = iter.next().unwrap();
        assert_eq!(0, fourth);
    }
}
// TODO: tests
