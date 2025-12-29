use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};
use filetime::{set_file_mtime, FileTime};
use tempfile::TempDir;

fn get_eza_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("eza");
    path
}

fn create_file_with_mtime(dir: &TempDir, filename: &str, age_secs: u64) -> PathBuf {
    let file_path = dir.path().join(filename);
    fs::File::create(&file_path).expect("Failed to create file");
    
    // Set the modification time to (now - age_secs)
    let now = SystemTime::now();
    let file_time = now
        .checked_sub(Duration::from_secs(age_secs))
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| FileTime::from_unix_time(d.as_secs() as i64, 0))
        .expect("Failed to calculate file time");
    
    set_file_mtime(&file_path, file_time).expect("Failed to set mtime");
    file_path
}

#[test]
fn test_since_filter_basic() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create files with different ages
    create_file_with_mtime(&temp_dir, "old_file.txt", 7200); // 2 hours old
    create_file_with_mtime(&temp_dir, "recent_file.txt", 60); // 1 minute old
    
    let eza = get_eza_binary();
    
    // Test with --since 1h (should show only recent_file.txt)
    let output = Command::new(&eza)
        .arg("--since")
        .arg("1h")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute eza");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("recent_file.txt"), "Should show recent_file.txt");
    assert!(!stdout.contains("old_file.txt"), "Should not show old_file.txt");
}

#[test]
fn test_since_filter_shows_all_recent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create files that are all recent
    create_file_with_mtime(&temp_dir, "file1.txt", 30); // 30 seconds old
    create_file_with_mtime(&temp_dir, "file2.txt", 60); // 1 minute old
    
    let eza = get_eza_binary();
    
    // Test with --since 5m (should show both files)
    let output = Command::new(&eza)
        .arg("--since")
        .arg("5m")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute eza");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file1.txt"), "Should show file1.txt");
    assert!(stdout.contains("file2.txt"), "Should show file2.txt");
}

#[test]
fn test_since_filter_hides_all_old() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create files that are all old
    create_file_with_mtime(&temp_dir, "old1.txt", 3600); // 1 hour old
    create_file_with_mtime(&temp_dir, "old2.txt", 7200); // 2 hours old
    
    let eza = get_eza_binary();
    
    // Test with --since 30m (should show nothing)
    let output = Command::new(&eza)
        .arg("--since")
        .arg("30m")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute eza");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("old1.txt"), "Should not show old1.txt");
    assert!(!stdout.contains("old2.txt"), "Should not show old2.txt");
}

#[test]
fn test_since_filter_long_view() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    create_file_with_mtime(&temp_dir, "old_file.txt", 7200); // 2 hours old
    create_file_with_mtime(&temp_dir, "recent_file.txt", 60); // 1 minute old
    
    let eza = get_eza_binary();
    
    // Test with --since and -l (long view)
    let output = Command::new(&eza)
        .arg("--since")
        .arg("1h")
        .arg("-l")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute eza");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("recent_file.txt"), "Should show recent_file.txt in long view");
    assert!(!stdout.contains("old_file.txt"), "Should not show old_file.txt in long view");
}

#[test]
fn test_since_filter_tree_view() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create subdirectory
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).expect("Failed to create subdir");
    
    // Create files
    create_file_with_mtime(&temp_dir, "old_root.txt", 7200); // 2 hours old in root
    create_file_with_mtime(&temp_dir, "recent_root.txt", 60); // 1 minute old in root
    
    // Create file in subdirectory
    let sub_file = subdir.join("recent_sub.txt");
    fs::File::create(&sub_file).expect("Failed to create sub file");
    let file_time = SystemTime::now()
        .checked_sub(Duration::from_secs(60))
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| FileTime::from_unix_time(d.as_secs() as i64, 0))
        .expect("Failed to calculate file time");
    set_file_mtime(&sub_file, file_time).expect("Failed to set mtime");
    
    let eza = get_eza_binary();
    
    // Test with --since and --tree
    let output = Command::new(&eza)
        .arg("--since")
        .arg("1h")
        .arg("--tree")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute eza");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("recent_root.txt"), "Should show recent_root.txt in tree view");
    assert!(!stdout.contains("old_root.txt"), "Should not show old_root.txt in tree view");
    assert!(stdout.contains("subdir"), "Should show subdir");
    assert!(stdout.contains("recent_sub.txt"), "Should show recent_sub.txt in tree view");
}

#[test]
fn test_since_filter_with_various_durations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    create_file_with_mtime(&temp_dir, "test_file.txt", 45); // 45 seconds old
    
    let eza = get_eza_binary();
    
    // Test with 30s - should not show
    let output = Command::new(&eza)
        .arg("--since")
        .arg("30s")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute eza");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("test_file.txt"), "Should not show with --since 30s");
    
    // Test with 1m - should show
    let output = Command::new(&eza)
        .arg("--since")
        .arg("1m")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute eza");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test_file.txt"), "Should show with --since 1m");
    
    // Test with 1h - should show
    let output = Command::new(&eza)
        .arg("--since")
        .arg("1h")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute eza");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test_file.txt"), "Should show with --since 1h");
}

#[test]
fn test_since_filter_invalid_duration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    create_file_with_mtime(&temp_dir, "test_file.txt", 60);
    
    let eza = get_eza_binary();
    
    // Test with invalid duration format
    let output = Command::new(&eza)
        .arg("--since")
        .arg("invalid")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute eza");
    
    // Should fail with error
    assert!(!output.status.success(), "Should fail with invalid duration");
}
