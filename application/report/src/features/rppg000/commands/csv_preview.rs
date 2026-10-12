//! Reads the first rows out of the first bytes of a CSV file. Pure function (unit-tested in
//! `application/report/tests/rppg000_test.rs`).

/// `(columns, rows)` from `bytes`, the beginning of a CSV file (UTF-8, optional byte order mark).
/// At most `max_rows` rows. When `cut` is set the bytes end mid-file, so the last record may be
/// incomplete and is dropped.
pub fn preview_rows(bytes: &[u8], max_rows: usize, cut: bool) -> (Vec<String>, Vec<Vec<String>>) {
    let bytes = bytes.strip_prefix("\u{feff}".as_bytes()).unwrap_or(bytes);
    let mut reader = csv::ReaderBuilder::new().has_headers(true).flexible(true).from_reader(bytes);
    let columns = reader
        .headers()
        .map(|h| h.iter().map(str::to_string).collect())
        .unwrap_or_default();
    let mut rows: Vec<Vec<String>> = reader
        .records()
        .map_while(Result::ok)
        .map(|r| r.iter().map(str::to_string).collect())
        .collect();
    if cut {
        rows.pop();
    }
    rows.truncate(max_rows);
    (columns, rows)
}
