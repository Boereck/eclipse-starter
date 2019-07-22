use std::os::raw::c_int;
use std::ffi::OsString;
use std::os::windows::prelude::*;

// On Windows we use UTF-16 chars
#[cfg(windows)]
type NativeString = *const u16;

#[cfg(not(windows))]
type NativeString = *const c_char;


#[no_mangle]
pub unsafe extern fn runW(args_size: c_int, args: *const NativeString, vm_args: *const NativeString) -> c_int {
    let args_slice : &[NativeString] = 
        std::slice::from_raw_parts(args, (args_size as usize)-1);
//    println!("args: ");
//    args_slice.iter().map(|s_ptr: &NativeString| {
//        let mut count = 0isize;
//        while *s_ptr.offset(count) != 0 {
//            count = count + 1;
//        }
//        let result : &[u16] = std::slice::from_raw_parts(*s_ptr, count as usize);
//        result
//    }).filter_map(|char_slice| {
//        OsString::from_wide(char_slice)
//            .to_str()
//            .map(|s|s.to_string())
//    }).for_each(|s| println!("{}", s));
    0
}

#[no_mangle]
pub extern fn setInitialArgsW(args_size: c_int, args: *const NativeString, library: NativeString) -> () {
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
