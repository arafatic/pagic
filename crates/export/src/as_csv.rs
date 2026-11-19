//! Export a PDF's tables and text lines to RFC 4180 CSV (`.csv`).

use super::layout::{Item, Page, items};

fn csv_cell(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}

/// Export `pages` as comma-separated values.
pub fn csv(pages: &[Page]) -> String {
    let mut out = String::new();
    for it in items(pages) {
        match it {
            Item::PageBreak => {
                if !out.is_empty() && !out.ends_with("\n\n") {
                    out.push('\n');
                }
            }
            Item::Para(runs, _) => {
                let text: String = runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>().join(" ");
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    out.push_str(&csv_cell(trimmed));
                    out.push('\n');
                }
            }
            Item::Table(t) => {
                for row in &t.rows {
                    let line: Vec<String> = row.iter().map(|cell| csv_cell(cell.text.trim())).collect();
                    out.push_str(&line.join(","));
                    out.push('\n');
                }
            }
            Item::Img(_) => {}
        }
    }
    out
}
