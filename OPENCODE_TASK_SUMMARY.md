# Mark Recent Files Feature Report

## Summary

Implemented a new CLI flag: `--mark=<DURATION>`. When provided, files that were created or modified within the supplied duration will have an appended asterisk (` *`) in listings. This works across views (lines, grid, details, grid-details, and tree).

Usage example: `eza --tree --mark 5s` will mark files created/modified within the last 5 seconds.

## Changes made

- options:
  - Added new flag `--mark` to `src/options/flags.rs` (takes a value).
  - Documented the flag in `src/options/help.rs` and `man/eza.1.md`.
  - Implemented `Options::mark_duration` parser in `src/options/file_name.rs` which parses the provided duration using humantime (supports values like `5s`, `2m`, `1h`, `1d`).

- output:
  - Added an optional `mark_duration: Option<Duration>` field to `output::file_name::Options` and threaded it through when deducing file name options.
  - Appended an asterisk (` *`) to the printed filename when a file’s modified or created time is within the provided duration.

- dependencies:
  - Added `humantime` (for parsing human durations) and `parse_duration` (was considered but we use humantime in the final impl).

- tests:
  - Ran unit and integration tests. All existing tests pass.

## Implementation notes

- Parsing the duration is performed with `humantime::parse_duration`, which accepts strings like `5s`, `2m`, `1h`, `1d`, `1w`, and also ISO durations like `PT5S`.

- The feature checks both the file’s `modified_time()` and `created_time()` (if available). If either timestamp exists and is within the time window from 'now', the file is marked.

- For symbolic links, the existing `File` behavior for dereferencing is respected when determining timestamps — i.e., `File::modified_time()` accounts for `--dereference` behavior when the file name object was created with deref enabled.

- The asterisk is appended using the same style as the filename colour — i.e., `bits.push(self.style().paint(" *"));` so it follows theme styling.

## Limitations and open questions

- Time precision and time zones:
  - We compare the file time with `chrono::Local::now()` and `NaiveDateTime`. This is consistent with the existing time handling but could be impacted by unusual system clocks or timezones.

- Granularity:
  - Humantime supports many duration formats, but the choice of library may impact user expectations about accepted units and parsing edge cases. We intentionally chose `humantime` for wide, friendly coverage.

- Performance:
  - Checking the timestamp for each file is cheap relative to the I/O involved in reading directories. No special caching was added.

- Interaction with views:
  - The asterisk is appended directly to the printed filename. In grid views, this increases the cell width and might impact grid layout slightly. This is acceptable; if desired, we could consider placing a separate column (like the git status column) for marks.

- Configuration and environment variables:
  - Currently, the `--mark` flag must be supplied on each invocation. Consider adding an environment variable (e.g., `EZA_MARK_DURATION`) in future to allow a persistent default.

- Tests:
  - No new unit tests were added specifically for the marking behavior. Creating deterministic tests requires generating files with controlled timestamps (see `devtools/generate-timestamp-test-dir.sh`). If you want, I can add integration tests that create files with modified timestamps and assert the presence of a trailing `*` in output.

## Future improvements

- Add tests that explicitly exercise `--mark` behavior with a generated timestamped test directory.
- Consider adding a `--mark-symbol` option or theme config for the marker character and its style (e.g., make it configurable or use a dedicated style code).
- Add environment variable `EZA_MARK_DURATION` to configure default marking behavior.

---

If you'd like, I can:

- Add automated tests for the feature (create files with specific timestamps and verify output). If you want that I will add a small integration test and update the test data using `devtools/generate-timestamp-test-dir.sh`.
- Implement an environment variable for a default mark duration.

Would you like me to add tests for this feature now?