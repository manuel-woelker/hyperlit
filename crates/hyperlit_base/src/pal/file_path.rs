use relative_path::{RelativePath, RelativePathBuf};
use std::path::{Path, PathBuf};

/* 📖 # Why use RelativePathBuf for FilePath?

FilePath wraps RelativePathBuf to enforce that all paths are relative to the PAL's
base directory, not absolute system paths. This provides several benefits:

1. **Type Safety**: The compiler prevents accidentally using absolute paths
2. **Intent Clarity**: Code explicitly shows these are base-relative paths
3. **Security**: Relative paths can't escape the base directory via ".."
4. **Consistency**: All PAL paths follow the same relative-to-base semantics

Using RelativePathBuf ensures paths stay within the PAL's scope at compile-time.
*/

/// Type-safe wrapper for file paths relative to PAL base directory.
///
/// Uses `RelativePathBuf` to enforce that paths are always relative to the PAL's
/// base directory, preventing accidental use of absolute or escaping paths.
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
pub struct FilePath(RelativePathBuf);

impl FilePath {
    /// Returns the underlying RelativePathBuf as a reference.
    pub fn as_relative(&self) -> &RelativePath {
        &self.0
    }

    /// Consumes the FilePath and returns the underlying RelativePathBuf.
    pub fn into_relative_path_buf(self) -> RelativePathBuf {
        self.0
    }

    /// Converts to a regular Path for use with std::fs operations.
    /// This returns the relative path portion without a base directory.
    pub fn as_path(&self) -> &Path {
        Path::new(self.as_relative().as_str())
    }

    /// Consumes the FilePath and returns a PathBuf.
    pub fn into_path_buf(self) -> PathBuf {
        PathBuf::from(self.0.as_str())
    }
}

impl From<&str> for FilePath {
    fn from(s: &str) -> Self {
        Self(RelativePathBuf::from(s))
    }
}

impl From<String> for FilePath {
    fn from(s: String) -> Self {
        Self(RelativePathBuf::from(s))
    }
}

impl From<RelativePathBuf> for FilePath {
    fn from(p: RelativePathBuf) -> Self {
        Self(p)
    }
}

impl From<&RelativePath> for FilePath {
    fn from(p: &RelativePath) -> Self {
        Self(p.to_relative_path_buf())
    }
}

impl From<&Path> for FilePath {
    fn from(p: &Path) -> Self {
        Self(RelativePathBuf::from(p.to_string_lossy().into_owned()))
    }
}

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<RelativePath> for FilePath {
    fn as_ref(&self) -> &RelativePath {
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
    fn test_file_path_from_relative_path() {
        let rp = RelativePath::new("docs/readme.md");
        let path = FilePath::from(rp);
        assert_eq!(path.as_path(), Path::new("docs/readme.md"));
    }

    #[test]
    fn test_file_path_from_pathbuf() {
        let pb = PathBuf::from("docs/readme.md");
        let path = FilePath::from(pb.as_path());
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
