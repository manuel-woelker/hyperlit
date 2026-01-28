use std::path::{Path, PathBuf};

/* 📖 # Why use a FilePath newtype wrapper?

FilePath provides type safety and clarity about which paths are managed by the PAL.
This prevents accidentally mixing regular filesystem paths with PAL-relative paths,
catching such errors at compile time. It's self-documenting—code clearly shows
intent and enables future validation/normalization in one place.
*/

/// Type-safe wrapper for file paths relative to PAL base directory.
///
/// This newtype prevents mixing up regular paths with PAL-managed paths, providing
/// compile-time safety and clarity about which paths are relative to the PAL's base
/// directory.
///
/// # Examples
///
/// ```
/// use hyperlit_base::FilePath;
///
/// let path1 = FilePath::from("src/main.rs");
/// let path2 = FilePath::from(String::from("tests/data.txt"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilePath(PathBuf);

impl FilePath {
    /// Returns the underlying PathBuf as a reference.
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Consumes the FilePath and returns the underlying PathBuf.
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

impl From<&str> for FilePath {
    fn from(s: &str) -> Self {
        Self(PathBuf::from(s))
    }
}

impl From<String> for FilePath {
    fn from(s: String) -> Self {
        Self(PathBuf::from(s))
    }
}

impl From<PathBuf> for FilePath {
    fn from(p: PathBuf) -> Self {
        Self(p)
    }
}

impl From<&Path> for FilePath {
    fn from(p: &Path) -> Self {
        Self(PathBuf::from(p))
    }
}

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_path_from_str() {
        let path = FilePath::from("src/main.rs");
        assert_eq!(path.as_path(), Path::new("src/main.rs"));
    }

    #[test]
    fn test_file_path_from_string() {
        let path = FilePath::from(String::from("tests/data.txt"));
        assert_eq!(path.as_path(), Path::new("tests/data.txt"));
    }

    #[test]
    fn test_file_path_from_pathbuf() {
        let pb = PathBuf::from("docs/readme.md");
        let path = FilePath::from(pb);
        assert_eq!(path.as_path(), Path::new("docs/readme.md"));
    }

    #[test]
    fn test_file_path_equality() {
        let path1 = FilePath::from("test.txt");
        let path2 = FilePath::from("test.txt");
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_file_path_inequality() {
        let path1 = FilePath::from("test1.txt");
        let path2 = FilePath::from("test2.txt");
        assert_ne!(path1, path2);
    }

    #[test]
    fn test_file_path_display() {
        let path = FilePath::from("src/main.rs");
        assert_eq!(path.to_string(), "src/main.rs".to_string());
    }

    #[test]
    fn test_file_path_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(FilePath::from("test1.txt"));
        set.insert(FilePath::from("test2.txt"));
        assert!(set.contains(&FilePath::from("test1.txt")));
        assert!(!set.contains(&FilePath::from("test3.txt")));
    }
}
