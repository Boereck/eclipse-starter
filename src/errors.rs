use core::fmt::Display;
use std::os::raw::c_int;
use std::fmt;

#[derive(Debug)]
pub enum LauncherError {
    LibraryLookupError(String),
    SecurityError(String),
    GeneralError(String),
    RunError(String, c_int),    
}

/// Automatically converts `String` and `&str` to 
/// `LaunchError::GeneralError`
impl <T> From<T> for LauncherError where T : AsRef<str> {

    fn from(msg: T) -> LauncherError {
        LauncherError::GeneralError(msg.as_ref().into())
    }

}

impl Display for LauncherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use LauncherError::*;
        let msg = match self {
            LibraryLookupError(msg) => msg,
            SecurityError(msg) => msg,
            GeneralError(msg) => msg,
            RunError(msg, _) => msg,
        };
        f.write_str(&msg)
    }
}