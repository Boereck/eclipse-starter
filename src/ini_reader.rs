
use std::path::PathBuf;
use std::io::prelude::*;
use std::io::{BufReader, Error};
use std::fs::File;
use std::ffi::OsStr;

pub fn read_ini(user_defined_config: &Option<String>, exe_path: &PathBuf) -> Result<impl Iterator<Item=String>, Error> {
    let ini_path = if let Some(user_ini) = user_defined_config {
        PathBuf::from(user_ini)
    } else {
        exe_to_ini_path(&exe_path)
    };
    let ini_file = File::open(ini_path)?;
    let reader = BufReader::new(ini_file);
    // Only take successfuly read lines, omit IO errors
    let result = reader.lines()
        .filter_map(Result::ok)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.trim_end().to_string());
    Ok(result)
}

// TODO: MacOS Version
fn exe_to_ini_path(exe_path: &PathBuf) -> PathBuf {
    let mut ini_path = exe_path.clone();
    strip_extension(&mut ini_path);
    ini_path.set_extension("ini");
    ini_path
    // TODO if windows and console launcher
//    if let Some(file_name) = exe_path.file_name() {
//        if let Some(file_name_str) = file_name.to_str() {
//            if(file_name_str.ends_with("c")) {
//                let new_name = file_name_str.to_string();
//                new_name.pop();
//                exe_path.set_file_name(new_name);
//            }
//        }
//    }
}

fn strip_extension(exe_path: &mut PathBuf) {
    // remove extension, by setting file stem as file name
    if let Some(stem) = exe_path.file_stem().map(OsStr::to_owned) {
        exe_path.set_file_name(stem);
    }
}