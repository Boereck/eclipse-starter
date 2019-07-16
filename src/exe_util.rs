use std::path::PathBuf;
use std::io;
use crate::path_util::*;

/// If `exe_path` is a symlink, the symlink is resolved 
/// (and if the resolved file is a symlink as well it will continue resolving)
/// and the resolved file will be returned.
pub fn get_exe_path() -> Result<PathBuf,io::Error> {
    // First try to find program from first command line argument
    if let Some(first_arg) = std::env::args().next() {
        let mut program_path = PathBuf::from(first_arg);
        // On windows we always use file extension .exe
        if cfg!(windows) && !has_extension_exe(&program_path) {
            program_path.set_extension("exe");
        }
        if let Some(program_location) = find_program(&program_path) {
            return Ok(program_location);
        }
    }
    // If we were not able to detect program location from arguments, we use OS functions
    let exe_path = std::env::current_exe()?;
    // Just to be sure, and to resolve symlinks, conconicalize the path
    std::fs::canonicalize(exe_path)
}

fn find_program(path: &PathBuf) -> Option<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else if has_parent(path) {
        std::fs::canonicalize(path).unwrap().exists();
        std::fs::canonicalize(dbg!(path)).ok()?
    } else {
        let path_str = path.to_str()?;
        search_on_path_env(path_str)?
    };
    Some(path)
}

fn search_on_path_env(path: &str) -> Option<PathBuf> {
    // TODO implement
    // on windows prepend curr dir
    None
}

