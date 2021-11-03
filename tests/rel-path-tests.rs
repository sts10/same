mod rel_path_tests {
    use same::*;
    use std::path::Path;
    #[test]
    fn can_get_relative_path() {
        let dir_path = Path::new("home/username/Pictures");
        let this_sub_folder = Path::new("home/username/Pictures/holidays/maine");
        assert_eq!(
            get_path_relative_to_dir(&dir_path, &this_sub_folder),
            Path::new("holidays/maine"),
        );
    }

    #[test]
    fn can_get_relative_path_with_filename() {
        let dir_path = Path::new("home/username/Pictures");
        let absolute_path = Path::new("home/username/Pictures/holidays/maine/sunset.jpg");
        assert_eq!(
            get_path_relative_to_dir(&dir_path, &absolute_path),
            Path::new("holidays/maine/sunset.jpg"),
        );
    }
}
