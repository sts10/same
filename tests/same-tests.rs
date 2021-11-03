mod same_tests {
    use same::*;
    use std::path::Path;

    #[test]
    fn can_determine_same_directory_is_same() {
        let test_path = Path::new("tests/test-files/bar");
        assert_eq!(
            hash_dir(&test_path, 1, false),
            hash_dir(&test_path, 1, false)
        );
        assert_eq!(
            hash_dir(&test_path, 2, false),
            hash_dir(&test_path, 2, false)
        );
        assert_eq!(
            hash_dir(&test_path, 4, false),
            hash_dir(&test_path, 4, false)
        );
    }

    #[test]
    fn can_determine_different_directories_are_different() {
        let path1 = Path::new("/home/sschlinkert/code/tic-tac-go");
        let path2 = Path::new("/home/sschlinkert/code/tidy");
        assert_ne!(hash_dir(&path1, 1, false), hash_dir(&path2, 1, false));
        assert_ne!(hash_dir(&path1, 2, false), hash_dir(&path2, 2, false));
        assert_ne!(hash_dir(&path1, 4, false), hash_dir(&path2, 4, false));
    }

    #[test]
    fn can_determine_copied_directory_is_same_from_paths_even_have_have_different_paths_and_path_lengths(
    ) {
        let test_path1 = Path::new("tests/test-files/bar");
        let test_path2 = Path::new("tests/test-files/back-ups/bar");
        assert_eq!(
            hash_dir(&test_path1, 2, false),
            hash_dir(&test_path2, 2, false)
        );
        assert_eq!(
            hash_dir(&test_path1, 4, false),
            hash_dir(&test_path2, 4, false)
        );
    }

    #[test]
    fn can_detect_files_differing_solely_based_on_file_content() {
        let path1 = Path::new("tests/test-files/bar");
        let path2 = Path::new("tests/test-files/corrupted_back_up/bar");
        // t=1 is too dumb for this test...
        assert_eq!(hash_dir(&path1, 1, false), hash_dir(&path2, 1, false));
        // but t=3 and t-4 should spot the difference
        assert_ne!(hash_dir(&path1, 3, false), hash_dir(&path2, 3, false));
        assert_ne!(hash_dir(&path1, 4, false), hash_dir(&path2, 4, false));
    }

    #[test]
    fn can_determine_copied_directory_is_same_from_paths_even_have_have_different_dir_name() {
        let test_path1 = Path::new("tests/test-files/bar");
        let test_path2 = Path::new("tests/test-files/baz");
        assert_eq!(
            hash_dir(&test_path1, 2, false),
            hash_dir(&test_path2, 2, false)
        );
        assert_eq!(
            hash_dir(&test_path1, 4, false),
            hash_dir(&test_path2, 4, false)
        );
    }
}
