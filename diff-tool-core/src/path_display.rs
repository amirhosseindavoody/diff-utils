//! Abbreviate two file paths for display by eliding shared prefix/suffix
//! components and replacing them with `...`.

use std::ffi::OsString;
use std::path::{Component, Path};

/// Return display strings for the left and right panel titles.
///
/// When both paths are present they are abbreviated against each other; when
/// only one side has a file the full path is shown for that side.
pub fn abbreviated_path_titles(
    left: Option<&Path>,
    right: Option<&Path>,
) -> (Option<String>, Option<String>) {
    match (left, right) {
        (Some(l), Some(r)) => {
            let (a, b) = abbreviate_paths(l, r);
            (Some(a), Some(b))
        }
        (Some(l), None) => (Some(l.display().to_string()), None),
        (None, Some(r)) => (None, Some(r.display().to_string())),
        (None, None) => (None, None),
    }
}

/// Abbreviate two paths by replacing shared leading and trailing components
/// with `...`.
pub fn abbreviate_paths(left: &Path, right: &Path) -> (String, String) {
    let left_components = path_components(left);
    let right_components = path_components(right);

    if left_components == right_components {
        return (
            left.display().to_string(),
            right.display().to_string(),
        );
    }

    let common_prefix = left_components
        .iter()
        .zip(&right_components)
        .take_while(|(a, b)| a == b)
        .count();

    let left_rest = left_components.len().saturating_sub(common_prefix);
    let right_rest = right_components.len().saturating_sub(common_prefix);

    let mut common_suffix = 0;
    while common_suffix < left_rest
        && common_suffix < right_rest
        && left_components[left_components.len() - 1 - common_suffix]
            == right_components[right_components.len() - 1 - common_suffix]
    {
        common_suffix += 1;
    }

    let left_unique_end = left_components.len().saturating_sub(common_suffix);
    let right_unique_end = right_components.len().saturating_sub(common_suffix);

    let left_middle = &left_components[common_prefix..left_unique_end];
    let right_middle = &right_components[common_prefix..right_unique_end];
    let suffix = &left_components[left_unique_end..];

    let has_shared_prefix = common_prefix > 0;
    (
        build_abbreviated(has_shared_prefix, left_middle, suffix),
        build_abbreviated(has_shared_prefix, right_middle, suffix),
    )
}

fn path_components(path: &Path) -> Vec<OsString> {
    path.components()
        .filter(|c| !matches!(c, Component::RootDir | Component::Prefix(_)))
        .map(|c| c.as_os_str().to_os_string())
        .collect()
}

fn build_abbreviated(prefix_elided: bool, middle: &[OsString], suffix: &[OsString]) -> String {
    let mut parts: Vec<String> = Vec::new();
    if prefix_elided {
        parts.push("...".to_string());
    }
    for component in middle.iter().chain(suffix.iter()) {
        parts.push(component.to_string_lossy().into_owned());
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn identical_paths_show_full() {
        let path = PathBuf::from("/home/user/project/src/main.rs");
        let (left, right) = abbreviate_paths(&path, &path);
        assert_eq!(left, path.display().to_string());
        assert_eq!(right, path.display().to_string());
    }

    #[test]
    fn differs_only_in_one_directory() {
        let left = PathBuf::from("/home/user/project-a/src/main.rs");
        let right = PathBuf::from("/home/user/project-b/src/main.rs");
        let (l, r) = abbreviate_paths(&left, &right);
        assert_eq!(l, ".../project-a/src/main.rs");
        assert_eq!(r, ".../project-b/src/main.rs");
    }

    #[test]
    fn differs_only_in_filename() {
        let left = PathBuf::from("/tmp/foo/bar.txt");
        let right = PathBuf::from("/tmp/foo/baz.txt");
        let (l, r) = abbreviate_paths(&left, &right);
        assert_eq!(l, ".../bar.txt");
        assert_eq!(r, ".../baz.txt");
    }

    #[test]
    fn differs_in_middle_with_shared_suffix() {
        let left = PathBuf::from("/a/b/c/d/e.txt");
        let right = PathBuf::from("/a/x/c/d/e.txt");
        let (l, r) = abbreviate_paths(&left, &right);
        assert_eq!(l, ".../b/c/d/e.txt");
        assert_eq!(r, ".../x/c/d/e.txt");
    }

    #[test]
    fn no_shared_prefix_shows_unique_parts() {
        let left = PathBuf::from("foo/a.txt");
        let right = PathBuf::from("bar/b.txt");
        let (l, r) = abbreviate_paths(&left, &right);
        assert_eq!(l, "foo/a.txt");
        assert_eq!(r, "bar/b.txt");
    }

    #[test]
    fn single_side_uses_full_path() {
        let left = PathBuf::from("/very/long/path/to/file.txt");
        let (l, r) = abbreviated_path_titles(Some(&left), None);
        assert_eq!(l.as_deref(), Some(left.display().to_string().as_str()));
        assert_eq!(r, None);
    }
}
