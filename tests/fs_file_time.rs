#[cfg(test)]
mod file_time_tests {
    use tempfile::tempdir;
    use filetime::{FileTime, set_file_times};
    use crate::fs::File;

    #[test]
    fn modified_time_recent_and_old() {
        let dir = tempdir().expect("tempdir");
        let recent_path = dir.path().join("recent_test.txt");
        let old_path = dir.path().join("old_test.txt");

        std::fs::write(&recent_path, "recent\n").expect("write recent");
        std::fs::write(&old_path, "old\n").expect("write old");

        // Set old file time to 2000-01-01
        let old_ft = FileTime::from_unix_time(946684800, 0);
        set_file_times(&old_path, old_ft, old_ft).expect("set old time");

        // Create File objects
        let recent_file = File::from_args(recent_path.clone(), None::<&crate::fs::dir::Dir>, None::<String>, false, false, None);
        let old_file = File::from_args(old_path.clone(), None::<&crate::fs::dir::Dir>, None::<String>, false, false, None);

        let now = chrono::Local::now().naive_local();
        let one_hour = std::time::Duration::from_secs(3600);
        let one_hour_chrono = chrono::Duration::from_std(one_hour).unwrap();

        if let Some(m) = recent_file.modified_time() {
            assert!(now.signed_duration_since(m) <= one_hour_chrono, "recent file not within 1h");
        } else {
            panic!("recent file missing modified_time");
        }

        if let Some(m) = old_file.modified_time() {
            assert!(now.signed_duration_since(m) > one_hour_chrono, "old file unexpectedly within 1h");
        } else {
            panic!("old file missing modified_time");
        }
    }
}
