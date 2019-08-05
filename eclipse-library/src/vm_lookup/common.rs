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

use crate::params::{EclipseEEProps, EclipseParams};
use eclipse_common::path_buf;
use eclipse_common::path_util::PATHS_SEPARATOR;
use std::path::{Path, PathBuf};

pub fn is_vm_library(path: &Path) -> bool {
    let ext = if let Some(ext) = path.extension() {
        ext.to_str().unwrap_or_default()
    } else {
        return false;
    };
    is_vm_library_ext(ext)
}

pub fn is_vm_library_ext(file_ext: &str) -> bool {
    let is_win = cfg!(target_os = "windows");

    (is_win && file_ext == "dll")
        || ((!is_win) && (file_ext == "so" || file_ext == "jnilib" || file_ext == "dylib"))
}

pub fn get_vm_library_search_path(
    lib_path: &Path,
    params: &EclipseParams,
    ee_props: Option<&EclipseEEProps>,
) -> Vec<PathBuf> {
    // If we ee_lib_path in ee props set, return the paths from there
    if let Some(EclipseEEProps {
        ee_lib_path: Some(paths),
        ..
    }) = ee_props
    {
        return paths.iter().map(PathBuf::from).collect();
    }
    // "regular" case: determine from lib_path

    // add lib_path, its parent and grandparent/lib/arg
    let mut paths = vec![lib_path.to_path_buf()];
    if let Some(parent) = lib_path.parent() {
        paths.push(parent.to_path_buf());
        if let Some(grandparent) = parent.parent() {
            // trying grandparent/lib/arch
            let vm_arch = get_vm_arch(params);
            let path = path_buf![grandparent, "lib", vm_arch,];
            if path.exists() {
                paths.push(path);
            }
        }
    }
    paths
}

fn get_vm_arch(params: &EclipseParams) -> &str {
    let os_str: &str = if let Some(os_str) = &params.os {
        os_str
    } else {
        return "";
    };
    match os_str {
        "x86_64" => "amd64",
        "x86" => "i386",
        _ => os_str,
    }
}

const DIR_PATH_SEPARATOR: [char; 2] = [std::path::MAIN_SEPARATOR, PATHS_SEPARATOR];

/// `path` contains a pathSeparator separated list of paths, check
/// that it contains all the `paths` given.  Each path is expected to be
/// terminated with a pathSeparator character.
pub fn contains_paths<P: AsRef<Path>>(path: &str, paths: &[P]) -> bool {
    let path = if path.ends_with(PATHS_SEPARATOR) {
        path.to_string()
    } else {
        format!("{}{}", path, PATHS_SEPARATOR)
    };

    for p in paths {
        let s = if let Some(s) = p.as_ref().to_str() {
            s
        } else {
            continue;
        };
        if let Some(index) = path.find(s) {
            let (before, found_and_rest) = path.split_at(index);
            let after = &found_and_rest[s.len()..];

            // There shall be PATHS_SEPARATOR before and after path. The end may have an additional path separator before the PATHS_SEPARATOR
            let dir_path_sep: String = DIR_PATH_SEPARATOR.iter().collect();
            if !(before.is_empty() || before.ends_with(PATHS_SEPARATOR))
                || !(after.starts_with(PATHS_SEPARATOR) || after.starts_with(&dir_path_sep))
            {
                return false;
            }
        } else {
            // Not found
            return false;
        }
    }
    true
}

#[cfg(test)]
mod test {
    use super::contains_paths;
    use eclipse_common::path_util::PATHS_SEPARATOR;
    use std::path::Path;

    #[test]
    fn test_contains_paths_first_path() {
        let mut paths = String::from("/foo/bar/baz");
        paths.push(PATHS_SEPARATOR);
        paths.push_str("/var/bar/tar");
        let path = Path::new("/foo/bar/baz");
        assert!(contains_paths(&paths, &[path]));
    }

    #[test]
    fn test_contains_paths_first_path_only_prefix() {
        let mut paths = String::from("/foo/bar/baz/boo");
        paths.push(PATHS_SEPARATOR);
        paths.push_str("/var/bar/tar");
        let path = Path::new("/foo/bar/baz");
        assert!(!contains_paths(&paths, &[path]));
    }

    #[test]
    fn test_contains_paths_first_path_only_postfix() {
        let mut paths = String::from("boo/foo/bar/baz");
        paths.push(PATHS_SEPARATOR);
        paths.push_str("/var/bar/tar");
        let path = Path::new("/foo/bar/baz");
        assert!(!contains_paths(&paths, &[path]));
    }

    #[test]
    fn test_contains_paths_first_path_trailing_sep() {
        let mut paths = String::from("/foo/bar/baz");
        paths.push(std::path::MAIN_SEPARATOR);
        paths.push(PATHS_SEPARATOR);
        paths.push_str("/var/bar/tar");
        let path = Path::new("/foo/bar/baz");
        assert!(contains_paths(&paths, &[path]));
    }

    #[test]
    fn test_contains_paths_last_path() {
        let mut paths = String::from("/var/bar/tar");
        paths.push(PATHS_SEPARATOR);
        paths.push_str("/foo/bar/baz");
        let path = Path::new("/foo/bar/baz");
        assert!(contains_paths(&paths, &[path]));
    }

    #[test]
    fn test_contains_paths_last_path_only_prefix() {
        let mut paths = String::from("/var/bar/tar");
        paths.push(PATHS_SEPARATOR);
        paths.push_str("/foo/bar/baz/boo");
        let path = Path::new("/foo/bar/baz");
        assert!(!contains_paths(&paths, &[path]));
    }

    #[test]
    fn test_contains_paths_last_path_only_postfix() {
        let mut paths = String::from("/var/bar/tar");
        paths.push(PATHS_SEPARATOR);
        paths.push_str("boo/foo/bar/baz");
        let path = Path::new("/foo/bar/baz");
        assert!(!contains_paths(&paths, &[path]));
    }

    #[test]
    fn test_contains_paths_last_path_trailing_path_sep() {
        let mut paths = String::from("/var/bar/tar");
        paths.push(PATHS_SEPARATOR);
        paths.push_str("/foo/bar/baz");
        paths.push(std::path::MAIN_SEPARATOR);
        let path = Path::new("/foo/bar/baz");
        assert!(contains_paths(&paths, &[path]));
    }

    #[test]
    fn test_contains_paths_not_contained() {
        let mut paths = String::from("/hui/boo/baz");
        paths.push(PATHS_SEPARATOR);
        paths.push_str("/var/bar/tar");
        let path = Path::new("/foo/bar/baz");
        assert!(!contains_paths(&paths, &[path]));
    }
}
