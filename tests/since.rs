use std::process::Command;
use tempfile::tempdir;
use filetime::{FileTime, set_file_times};

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // skip until 'm' or end
            while let Some(cc) = chars.next() {
                if cc == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn since_marks_recent_files() {
    let dir = tempdir().expect("create tempdir");
    let old_path = dir.path().join("old.txt");
    let recent_path = dir.path().join("recent.txt");

    std::fs::write(&old_path, "old\n").expect("write old file");
    std::fs::write(&recent_path, "recent\n").expect("write recent file");

    // Set old file to a timestamp far in the past (2000-01-01)
    let old_ft = FileTime::from_unix_time(946684800, 0);
    set_file_times(&old_path, old_ft, old_ft).expect("set old file time");

    // Ensure recent file has a current timestamp (30 minutes ago) so it's
    // deterministically within the 1 hour window used by this test.
    let recent_time = std::time::SystemTime::now() - std::time::Duration::from_secs(30 * 60);
    let recent_ft = FileTime::from_system_time(recent_time);
    set_file_times(&recent_path, recent_ft, recent_ft).expect("set recent file time");

    let eza_bin = std::env::var("CARGO_BIN_EXE_eza").unwrap_or_else(|_| {
        // Fallback to target/debug/eza if Cargo didn't provide the env var
        format!("{}/target/debug/eza", std::env::var("CARGO_MANIFEST_DIR").unwrap())
    });

    let out = Command::new(eza_bin)
        .arg("--oneline")
        .arg("--since")
        .arg("1d")
        .arg(dir.path())
        .output()
        .expect("failed to run eza");

    assert!(out.status.success(), "eza exited with failure: {}", String::from_utf8_lossy(&out.stderr));

    let stdout = String::from_utf8_lossy(&out.stdout);
    let clean = strip_ansi(&stdout);

    // recent should be marked with an asterisk, old should not
    assert!(clean.contains("recent.txt *"), "recent not marked as expected: {}", clean);
    assert!(clean.contains("old.txt") && !clean.contains("old.txt *"), "old was incorrectly marked: {}", clean);
}
