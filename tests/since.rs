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

    let eza_bin = std::env::var("CARGO_BIN_EXE_eza").expect("CARGO_BIN_EXE_eza not set");

    let out = Command::new(eza_bin)
        .arg("--oneline")
        .arg("--since")
        .arg("1h")
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
