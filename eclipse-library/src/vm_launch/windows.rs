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
use super::common::StringHolder;
use eclipse_common::native_str;
use eclipse_common::native_str::NativeString;
use libc::free;
use core::ffi::c_void;

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

pub fn to_string_holder<'i, 'o>(strs: impl Iterator<Item = &'i str>)-> impl StringHolder<'o> + 'i 
    where 'i : 'o {

    let utf16_strs: Vec<_> = strs.map(native_str::to_native_str).collect();
    let platform_strs = to_default_platform_encoding( utf16_strs.iter().map(|(_,s)| *s) );
    let platform_strs_raw: Vec<_> = platform_strs.iter().map(|ps| ps.default_enc_str).collect();
    
    StringHolderOs{
        strs: platform_strs,
        strs_raw: platform_strs_raw,
        marker: std::marker::PhantomData{},
    }
}

fn to_default_platform_encoding(strs: impl Iterator<Item = NativeString>) -> Vec<DefaultEncStrRef> {
    unimplemented!()
}

/// Simple wrapper around pointer, freeing the memory pointed to when
/// being dropped.
struct DefaultEncStrRef {
    default_enc_str: *mut i8,
}

impl Drop for DefaultEncStrRef {
    fn drop(&mut self) {
        unsafe {
            free(self.default_enc_str as *mut c_void);
        }
    }
}

pub struct StringHolderOs<'i> {
    strs: Vec<DefaultEncStrRef>,
    strs_raw: Vec<*mut i8>,
    marker: std::marker::PhantomData<&'i str>,
}

impl <'i, 'o> StringHolder<'o> for StringHolderOs<'i> where 'i : 'o {
    type NativeStrIter = std::iter::Cloned<std::slice::Iter<'o, *mut i8>>;

    fn get_strings(&'o self) -> Self::NativeStrIter {
        self.strs_raw.iter().cloned()
    }
}