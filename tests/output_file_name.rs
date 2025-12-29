use tempfile::tempdir;
use filetime::{FileTime, set_file_times};
use crate::fs::File;
use crate::output::file_name::{Options as FileOptions, Classify, QuoteStyle, ShowIcons, EmbedHyperlinks, Absolute};
use crate::output::render::filetype as filetype_mod;
use nu_ansi_term::Style;

struct TestColours;
impl filetype_mod::Colours for TestColours {
    fn normal(&self) -> Style { Style::default() }
    fn directory(&self) -> Style { Style::default() }
    fn pipe(&self) -> Style { Style::default() }
    fn symlink(&self) -> Style { Style::default() }
    fn block_device(&self) -> Style { Style::default() }
    fn char_device(&self) -> Style { Style::default() }
    fn socket(&self) -> Style { Style::default() }
    fn special(&self) -> Style { Style::default() }
}

impl crate::output::file_name::Colours for TestColours {
    fn symlink_path(&self) -> Style { Style::default() }
    fn normal_arrow(&self) -> Style { Style::default() }
    fn broken_symlink(&self) -> Style { Style::default() }
    fn broken_filename(&self) -> Style { Style::default() }
    fn control_char(&self) -> Style { Style::default() }
    fn broken_control_char(&self) -> Style { Style::default() }
    fn executable_file(&self) -> Style { Style::default() }
    fn mount_point(&self) -> Style { Style::default() }
    fn colour_file(&self, _file: &File<'_>) -> Style { Style::default() }
    fn style_override(&self, _file: &File<'_>) -> Option<crate::theme::FileNameStyle> { None }
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // skip until 'm' or end
            while let Some(cc) = chars.next() {
                if cc == 'm' { break; }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn file_name_marks_recent() {
    let dir = tempdir().expect("create tempdir");
    let recent_path = dir.path().join("recent_test.txt");
    let old_path = dir.path().join("old_test.txt");

    std::fs::write(&recent_path, "recent\n").expect("write recent");
    std::fs::write(&old_path, "old\n").expect("write old");

    // Set old file time to 2000-01-01
    let old_ft = FileTime::from_unix_time(946684800, 0);
    set_file_times(&old_path, old_ft, old_ft).expect("set old time");

    let recent_file = File::from_args(recent_path.clone(), None::<&crate::fs::dir::Dir>, None::<String>, false, false, None);
    let old_file = File::from_args(old_path.clone(), None::<&crate::fs::dir::Dir>, None::<String>, false, false, None);

    let file_style = FileOptions {
        classify: Classify::JustFilenames,
        quote_style: QuoteStyle::QuoteSpaces,
        show_icons: ShowIcons::Never,
        embed_hyperlinks: EmbedHyperlinks::Off,
        is_a_tty: true,
        absolute: Absolute::Off,
        since_duration: Some(std::time::Duration::from_secs(3600)),
    };

    let colours = TestColours;

    let recent_cell = file_style.for_file(&recent_file, &colours).paint();
    let old_cell = file_style.for_file(&old_file, &colours).paint();

    let recent_str = format!("{}", nu_ansi_term::AnsiStrings(&recent_cell));
    let old_str = format!("{}", nu_ansi_term::AnsiStrings(&old_cell));

    let clean_recent = strip_ansi(&recent_str);
    let clean_old = strip_ansi(&old_str);

    assert!(clean_recent.contains("recent_test.txt *"), "recent not marked: {}", clean_recent);
    assert!(clean_old.contains("old_test.txt") && !clean_old.contains("old_test.txt *"), "old incorrectly marked: {}", clean_old);
}
