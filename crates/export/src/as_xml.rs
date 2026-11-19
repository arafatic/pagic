//! Export a PDF's pages, headings, paragraphs, and tables as XML 1.0 (`.xml`).

use std::fmt::Write;

use super::as_html::esc;
use super::layout::{Item, Page, items};

/// Export `pages` as structured XML 1.0.
pub fn xml(pages: &[Page], title: &str) -> String {
    let mut out = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<document title=\"{}\">\n", esc(title));
    let mut page_num = 1usize;
    let mut in_page = true;
    let _ = writeln!(out, "  <page number=\"{page_num}\">");

    for it in items(pages) {
        match it {
            Item::PageBreak => {
                if in_page {
                    out.push_str("  </page>\n");
                }
                page_num += 1;
                let _ = writeln!(out, "  <page number=\"{page_num}\">");
                in_page = true;
            }
            Item::Para(runs, lvl) => {
                let text: String = runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>().join(" ");
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    if lvl > 0 {
                        let _ = writeln!(out, "    <heading level=\"{lvl}\">{}</heading>", esc(trimmed));
                    } else {
                        let _ = writeln!(out, "    <paragraph>{}</paragraph>", esc(trimmed));
                    }
                }
            }
            Item::Table(t) => {
                out.push_str("    <table>\n");
                for row in &t.rows {
                    out.push_str("      <row>\n");
                    for cell in row {
                        let _ = writeln!(out, "        <cell>{}</cell>", esc(cell.text.trim()));
                    }
                    out.push_str("      </row>\n");
                }
                out.push_str("    </table>\n");
            }
            Item::Img(im) => {
                let _ = writeln!(
                    out,
                    "    <image format=\"{}\" width=\"{:.1}\" height=\"{:.1}\"/>",
                    esc(im.ext),
                    im.size[0],
                    im.size[1]
                );
            }
        }
    }

    if in_page {
        out.push_str("  </page>\n");
    }
    out.push_str("</document>\n");
    out
}
