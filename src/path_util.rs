use lazy_static::lazy_static;
use regex::{Match, Regex};
use std::fs::DirEntry;
use std::path::Path;
use std::path::PathBuf;

pub fn has_parent(path: &Path) -> bool {
    // TODO: more performant way to check if parent exists?
    path.components().next().is_some()
}

pub fn has_extension_exe(path: &PathBuf) -> bool {
    path.extension().filter(|ext| *ext == "exe").is_some()
}

/// If path is relative, attempt to make it absolute by
/// 1) check relative to working directory
/// 2) check relative to provided program_dir
/// If reverse_order, then check the program_dir before the working dir
pub fn check_path(path: &Path, program_dir: &Path, reverse_order: bool) -> PathBuf {
    // if path is absolute we do not need to make it absolute
    if path.is_absolute() {
        return path.to_path_buf();
    }

    // if we cannot get working_dir, we use an empty path
    let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::new());
    let search_paths = if reverse_order {
        [&working_dir, program_dir]
    } else {
        [program_dir, &working_dir]
    };
    for search_path in search_paths.iter() {
        let absolute_path: PathBuf = [search_path, path].iter().collect();
        if absolute_path.exists() {
            return absolute_path;
        }
    }
    // we found nothing, simply return original path
    path.to_path_buf()
}

/// Find a file in directory `location`, where the file name has the 
/// `name_prefix`. If there are multiple matches, the file name is parsed
/// for a version number (at best effort) and the file with the maximum
/// version number is returned. If no such file is found, an empty optional
/// is returned.
pub fn find_file(location: &Path, name_prefix: &str) -> Option<PathBuf> {
    if !location.exists() || !location.is_dir() {
        return None;
    }

    std::fs::read_dir(location)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| is_file(entry))
        .filter_map(filename_entry_tuple)
        .filter(|(name, _)| name.starts_with(name_prefix))
        .max_by_key(|(name, _)| {
            let compare_str = name.trim_start_matches(name_prefix);
            get_version(compare_str)
        })
        .map(|(_, entry)| entry.path())
}

/// Returns an option holding the tuple of the given `entry` and it's file name
/// as a `&str`. If the file name cannot be determined, an empty option is returned.
fn filename_entry_tuple(entry: DirEntry) -> Option<(String, DirEntry)> {
    let name = entry.file_name().into_string().ok()?;
    Some((name, entry))
}

fn is_file(entry: &DirEntry) -> bool {
    entry.file_type().map(|t| t.is_file()).unwrap_or(false)
}

#[derive(Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    qualifier: String,
}

fn get_version(file_name: &str) -> Version {
    lazy_static! {
        /// Regex matching "_major.minor.patch.qualifier" where the part after mayor is optional
        static ref VERSION_REGEX: Regex = Regex::new(r"_(?P<major>\d+)(\.(?P<minor>\d+)\.(?P<patch>\d+)\.(?P<qualifier>.*))?.*").unwrap();
    }
    let captures_opt = VERSION_REGEX.captures(file_name);
    if let Some(captures) = captures_opt {
        Version {
            major: to_u32(captures.name("major")),
            minor: to_u32(captures.name("minor")),
            patch: to_u32(captures.name("patch")),
            qualifier: to_string(captures.name("qualifier")),
        }
    } else {
        Version::default()
    }
}

fn to_u32(m: Option<Match>) -> u32 {
    m.and_then(|m| m.as_str().parse().ok()).unwrap_or(0)
}

fn to_string(m: Option<Match>) -> String {
    m.map(|m| m.as_str().to_string())
        .unwrap_or_else(String::new)
}
