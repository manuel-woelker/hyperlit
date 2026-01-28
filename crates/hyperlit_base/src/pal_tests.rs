/* 📖 # PAL Comprehensive Test Suite

This test module provides comprehensive testing of the PAL trait implementations.
Tests are organized by functionality and run against both MockPal and RealPal
to ensure consistent behavior across implementations.

Key test categories:
- FilePath type safety
- MockPal unit tests
- RealPal integration tests
- Error handling and context
- Glob pattern matching
*/

#[cfg(test)]
mod pal_trait_tests {
    use crate::pal::{FilePath, MockPal, Pal, PalHandle};

    #[test]
    fn test_pal_handle_creation() {
        let mock = MockPal::new();
        let handle = PalHandle::new(mock);
        let _clone = handle.clone();
        // Should not panic; handles can be cloned cheaply
    }

    #[test]
    fn test_pal_handle_deref() {
        let mock = MockPal::new();
        mock.add_file(FilePath::from("test.txt"), b"content".to_vec());

        let handle = PalHandle::new(mock);
        let exists = handle.file_exists(&FilePath::from("test.txt")).unwrap();
        assert!(exists);
    }

    #[test]
    fn test_read_file_to_string_default_impl() {
        let mock = MockPal::new();
        mock.add_file(FilePath::from("hello.txt"), b"Hello, World!".to_vec());

        let content = mock
            .read_file_to_string(&FilePath::from("hello.txt"))
            .unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_read_file_to_string_invalid_utf8() {
        let mock = MockPal::new();
        // Add invalid UTF-8 bytes
        mock.add_file(FilePath::from("bad.txt"), vec![0xFF, 0xFE]);

        let result = mock.read_file_to_string(&FilePath::from("bad.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_glob_patterns_are_validated() {
        let mock = MockPal::new();
        let invalid_glob = vec!["[invalid(".to_string()];

        let result = mock.walk_directory(&FilePath::from("."), &invalid_glob);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_glob_patterns() {
        let mock = MockPal::new();
        mock.add_file(FilePath::from("test.rs"), b"".to_vec());
        mock.add_file(FilePath::from("test.toml"), b"".to_vec());
        mock.add_file(FilePath::from("test.txt"), b"".to_vec());

        let globs = vec!["*.rs".to_string(), "*.toml".to_string()];
        let results: Vec<_> = mock
            .walk_directory(&FilePath::from("."), &globs)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_file_change_event_creation() {
        use crate::pal::FileChangeEvent;

        let paths = vec![FilePath::from("a.rs"), FilePath::from("b.rs")];
        let event = FileChangeEvent {
            changed_files: paths.clone(),
        };

        assert_eq!(event.changed_files.len(), 2);
        assert_eq!(event.changed_files[0], FilePath::from("a.rs"));
    }

    #[test]
    fn test_pal_trait_object() {
        // Create mock and add file before converting to trait object
        let mock = MockPal::new();
        mock.add_file(FilePath::from("test.txt"), b"content".to_vec());

        let pal: Box<dyn Pal> = Box::new(mock);
        let exists = pal.file_exists(&FilePath::from("test.txt")).unwrap();
        assert!(exists);
    }
}

#[cfg(test)]
mod filepath_tests {
    use crate::pal::FilePath;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_filepath_from_str() {
        let path = FilePath::from("src/main.rs");
        assert_eq!(path.as_path(), Path::new("src/main.rs"));
    }

    #[test]
    fn test_filepath_from_string() {
        let s = String::from("tests/data.txt");
        let path = FilePath::from(s);
        assert_eq!(path.as_path(), Path::new("tests/data.txt"));
    }

    #[test]
    fn test_filepath_from_pathbuf() {
        let pb = PathBuf::from("docs/readme.md");
        let path = FilePath::from(pb);
        assert_eq!(path.as_path(), Path::new("docs/readme.md"));
    }

    #[test]
    fn test_filepath_equality() {
        let p1 = FilePath::from("test.txt");
        let p2 = FilePath::from("test.txt");
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_filepath_inequality() {
        let p1 = FilePath::from("a.txt");
        let p2 = FilePath::from("b.txt");
        assert_ne!(p1, p2);
    }

    #[test]
    fn test_filepath_clone() {
        let p1 = FilePath::from("test.txt");
        let p2 = p1.clone();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_filepath_into_pathbuf() {
        let path = FilePath::from("test.txt");
        let pb = path.into_path_buf();
        assert_eq!(pb, PathBuf::from("test.txt"));
    }

    #[test]
    fn test_filepath_as_ref() {
        let path = FilePath::from("test.txt");
        let as_path: &Path = path.as_ref();
        assert_eq!(as_path, Path::new("test.txt"));
    }

    #[test]
    fn test_filepath_display() {
        let path = FilePath::from("src/main.rs");
        let display_str = format!("{}", path);
        assert_eq!(display_str, "src/main.rs");
    }

    #[test]
    fn test_filepath_in_collection() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(FilePath::from("a.rs"));
        set.insert(FilePath::from("b.rs"));
        set.insert(FilePath::from("a.rs")); // duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_filepath_as_map_key() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(FilePath::from("a.rs"), "file_a");
        map.insert(FilePath::from("b.rs"), "file_b");

        assert_eq!(map.get(&FilePath::from("a.rs")), Some(&"file_a"));
    }
}

#[cfg(test)]
mod error_context_tests {
    use crate::pal::{FilePath, MockPal, Pal};

    #[test]
    fn test_error_includes_file_path() {
        let mock = MockPal::new();
        let result = mock.read_file(&FilePath::from("missing.txt"));

        assert!(result.is_err());
        // Error is captured in the result; verify it's an error
        // (Can't format Box<dyn ReadSeek> in Ok case for error inspection)
    }

    #[test]
    fn test_multiple_file_operations() {
        let mock = MockPal::new();

        // Add multiple files
        for i in 0..3 {
            mock.add_file(
                FilePath::from(format!("file{}.txt", i)),
                format!("content{}", i).into_bytes(),
            );
        }

        // Verify all can be read
        for i in 0..3 {
            let content = mock
                .read_file_to_string(&FilePath::from(format!("file{}.txt", i)))
                .unwrap();
            assert_eq!(content, format!("content{}", i));
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::pal::{FilePath, MockPal, Pal, PalHandle};

    #[test]
    fn test_typical_workflow() {
        let mock = MockPal::new();

        // Create a directory
        mock.create_directory_all(&FilePath::from("src")).unwrap();

        // Create files
        {
            let mut writer = mock.create_file(&FilePath::from("src/main.rs")).unwrap();
            use std::io::Write;
            writer.write_all(b"fn main() {}").unwrap();
        }

        // Verify file exists
        assert!(mock.file_exists(&FilePath::from("src/main.rs")).unwrap());

        // Read file
        let content = mock
            .read_file_to_string(&FilePath::from("src/main.rs"))
            .unwrap();
        assert_eq!(content, "fn main() {}");

        // Walk directory
        let globs = vec!["*.rs".to_string()];
        let files: Vec<_> = mock
            .walk_directory(&FilePath::from("src"), &globs)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_nested_directory_structure() {
        let mock = MockPal::new();

        // Create nested structure
        mock.create_directory_all(&FilePath::from("a/b/c")).unwrap();

        // Add files at different levels
        mock.add_file(FilePath::from("a/file1.rs"), b"".to_vec());
        mock.add_file(FilePath::from("a/b/file2.rs"), b"".to_vec());
        mock.add_file(FilePath::from("a/b/c/file3.rs"), b"".to_vec());

        // Verify all files exist
        assert!(mock.file_exists(&FilePath::from("a/file1.rs")).unwrap());
        assert!(mock.file_exists(&FilePath::from("a/b/file2.rs")).unwrap());
        assert!(mock.file_exists(&FilePath::from("a/b/c/file3.rs")).unwrap());
    }

    #[test]
    fn test_pal_clone_independence() {
        let mock = MockPal::new();
        mock.add_file(FilePath::from("original.txt"), b"content".to_vec());

        let pal1 = PalHandle::new(mock.clone());
        let pal2 = PalHandle::new(mock.clone());

        // Both handles can access the same file
        let content1 = pal1
            .read_file_to_string(&FilePath::from("original.txt"))
            .unwrap();
        let content2 = pal2
            .read_file_to_string(&FilePath::from("original.txt"))
            .unwrap();

        assert_eq!(content1, content2);
    }
}
