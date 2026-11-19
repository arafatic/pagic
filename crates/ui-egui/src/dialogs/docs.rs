//! Built-in, keyword-searchable Q&A documentation covering every tool and menu in Pagic.

use egui::{CornerRadius, Stroke, vec2};

use crate::theme::{self, Tokens};
use crate::{PagicApp, icons};

pub struct DocEntry {
    pub question: &'static str,
    pub answer: &'static str,
    pub keywords: &'static str,
    /// Optional tool group id in `pagic_engine::catalog::TOOL_GROUPS` to jump directly to the tool.
    pub group: Option<&'static str>,
}

pub const ENTRIES: &[DocEntry] = &[
    DocEntry {
        question: "1. How can I combine or merge multiple PDFs into one file?",
        answer: "Open Tools ▸ Combine files (menu bar), or click Merge PDFs under Organize in the left sidebar, or Merge on the Home screen. Add your PDFs (or drag and drop files), reorder pages or files as needed, and click Combine.",
        keywords: "combine merge join multiple files append pdfs",
        group: Some("combine"),
    },
    DocEntry {
        question: "2. How can I convert or export a PDF to Excel (.xlsx), Word (.docx), PowerPoint (.pptx), CSV, HTML, RTF, XML, or Images?",
        answer: "Open a PDF and go to File ▸ Save As (pick your target format in Save as type), or File ▸ Export To, or Tools ▸ Export a PDF (left sidebar Convert ▸ Export PDF). For Excel (.xlsx), Pagic uses its Hybrid Lattice + Stream engine to preserve bordered tables, merged cells, and multi-line wrapped cells.",
        keywords: "export convert xlsx excel docx word pptx powerpoint csv html xml rtf image png jpeg tiff save as table",
        group: Some("export"),
    },
    DocEntry {
        question: "3. How can I create a new PDF from scratch, from images, or from text?",
        answer: "Go to File ▸ Create, or Tools ▸ Create a PDF (left sidebar Create & Edit ▸ Create PDF). You can create a Blank Page, convert Images to PDF, create from Clipboard, or create a PDF from plain text.",
        keywords: "create new blank page image text clipboard",
        group: Some("create"),
    },
    DocEntry {
        question: "4. How can I edit existing text or images inside a PDF?",
        answer: "Open Tools ▸ Edit a PDF (left sidebar Create & Edit ▸ Edit PDF, or Home ▸ Edit). Turn on 'Edit text & images active', then click any text block to edit its wording, font, size, or color, or drag images and objects. You can also Add text or Add image from the same panel.",
        keywords: "edit text image modify font color move delete add",
        group: Some("edit"),
    },
    DocEntry {
        question: "5. How can I rotate, reorder, delete, insert, extract, or split PDF pages?",
        answer: "Open Tools ▸ Organize pages (or left sidebar Organize ▸ Organize pages). Select pages manually or by filter (All, Even, Odd, Landscape, Portrait), then Rotate, Delete, Extract, Insert from file, Replace pages, Split document, or Number pages.",
        keywords: "organize pages rotate reorder delete insert extract split replace number",
        group: Some("organize"),
    },
    DocEntry {
        question: "6. How can I compress or reduce the file size of a PDF?",
        answer: "Go to File ▸ Compress PDF, Tools ▸ Compress a PDF, left sidebar Organize ▸ Compress PDF, or Home ▸ Compress. Choose a compression level and save the optimized PDF.",
        keywords: "compress reduce file size optimize shrink",
        group: Some("compress"),
    },
    DocEntry {
        question: "7. How can I compare two PDF files to find differences?",
        answer: "Go to Tools ▸ Compare files (or left sidebar Organize ▸ Compare files, or the Compare icon on the right panel rail). Select the older and newer PDF versions to inspect text and layout changes.",
        keywords: "compare diff differences versions older newer",
        group: Some("compare"),
    },
    DocEntry {
        question: "8. How can I sign a PDF (handwritten/drawn signature or Digital ID certificate)?",
        answer: "For quick signing, open Sign ▸ Fill & Sign (or left sidebar Protect & Sign ▸ Fill & Sign) to draw/upload a signature, place initials, text, checkmarks, or dates. For cryptographic Digital ID signing or certifying, use Sign ▸ Sign with Digital ID or Sign ▸ Certify.",
        keywords: "sign signature fill digital id certificate certify initials date",
        group: Some("fill_sign"),
    },
    DocEntry {
        question: "9. How can I create or edit interactive PDF form fields?",
        answer: "Open Tools ▸ Prepare a form (or left sidebar Create & Edit ▸ Prepare a form). Add Text fields, Checkboxes, Radio buttons, Dropdowns, List boxes, Push buttons, or Signature fields, and configure Field Properties or Calculation order.",
        keywords: "form prepare field checkbox radio dropdown list button calculation order",
        group: Some("form"),
    },
    DocEntry {
        question: "10. How can I add comments, sticky notes, highlights, or drawings?",
        answer: "Use the floating Quick Tools bar on the left of the canvas (Sticky note, Highlight, Draw) or open Tools ▸ Add comments (left sidebar Create & Edit ▸ Add comments). Open the Comments panel on the right rail to filter, reply, or review annotations.",
        keywords: "comment annotate sticky note highlight underline strikethrough draw markup",
        group: Some("comment"),
    },
    DocEntry {
        question: "11. How can I add stamps, watermarks, headers & footers, backgrounds, or Bates numbers?",
        answer: "For stamps, open left sidebar Create & Edit ▸ Add a stamp. For Watermarks, Headers & Footers, Backgrounds, or Bates numbering, open Tools ▸ Edit a PDF and use Page marks (or Edit ▸ Watermark / Header & Footer).",
        keywords: "stamp watermark header footer background bates numbering",
        group: Some("stamp"),
    },
    DocEntry {
        question: "12. How can I password-protect a PDF or restrict printing/editing?",
        answer: "Open Tools ▸ Protect a PDF (or File ▸ Protect Using Password). Set an open password and/or permissions password (to restrict printing, copying, or modifying) and save.",
        keywords: "protect password encrypt security permissions lock restrict",
        group: Some("protect"),
    },
    DocEntry {
        question: "13. How can I permanently redact sensitive text or pages?",
        answer: "Open Tools ▸ Redact a PDF. Mark text, areas, or whole pages (or use Find text and redact), then click Apply Redactions to permanently remove the underlying text and graphics.",
        keywords: "redact remove sensitive hide black out permanent",
        group: Some("redact"),
    },
    DocEntry {
        question: "14. How can I measure distances, perimeters, or areas on a PDF?",
        answer: "Open Tools ▸ Measure (or Tools ▸ All tools ▸ Measure). Calibrate the drawing scale if needed, then pick Distance, Perimeter, or Area.",
        keywords: "measure distance perimeter area scale ruler",
        group: Some("measure"),
    },
    DocEntry {
        question: "15. How can I check PDF accessibility or inspect document tags?",
        answer: "Open Accessibility Checker or Tags from the right-hand panel rail, or via View ▸ Side panels ▸ Accessibility Checker / Tags.",
        keywords: "accessibility tags screen reader wcag check structure",
        group: Some("accessibility"),
    },
    DocEntry {
        question: "16. How do I search text inside a PDF or quickly find any tool/documentation?",
        answer: "Press Ctrl+F (Cmd+F on macOS) to search text inside the active PDF. Press Ctrl+K (Cmd+K) or click the 'Find tools and commands' search pill in the top bar to jump to any tool, command, or documentation topic.",
        keywords: "search find ctrl+f ctrl+k palette command help docs",
        group: None,
    },
    DocEntry {
        question: "17. How do I use Scan & OCR?",
        answer: "Open Tools ▸ Scan & OCR (left sidebar Convert ▸ Scan & OCR) to run text recognition on a file or multiple files, or enhance camera images.",
        keywords: "scan ocr recognize text camera image",
        group: Some("scan"),
    },
];

/// Return documentation entries matching all whitespace-separated keywords in `query`.
pub fn matches(query: &str) -> Vec<&'static DocEntry> {
    let q = pagic_render::text::fold_case(query.trim());
    if q.is_empty() {
        return ENTRIES.iter().collect();
    }
    let words: Vec<&str> = q.split_whitespace().collect();
    ENTRIES
        .iter()
        .filter(|e| {
            let hay = pagic_render::text::fold_case(&format!("{} {} {}", e.question, e.answer, e.keywords));
            words.iter().all(|w| hay.contains(w))
        })
        .collect()
}

/// Draw `Dialog::Documentation` with a live keyword search pill and Q&A cards.
pub fn show(ui: &mut egui::Ui, app: &mut PagicApp) -> bool {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new(tl!("Documentation & Help")).font(theme::semibold(18.0)).color(t.text));
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(tl!("Search by keyword (e.g. combine, excel, sign, edit, compress) to find where each tool lives and how to use it."))
            .font(theme::regular(12.5))
            .color(t.text_muted),
    );
    ui.add_space(8.0);

    // Search pill
    let mut open_tool: Option<&'static str> = None;
    egui::Frame::NONE
        .fill(t.panel)
        .stroke(Stroke::new(1.0, t.border))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icons::paint(ui, egui::Rect::from_min_size(ui.cursor().min + vec2(0.0, 2.0), vec2(15.0, 15.0)), "search", 15.0, t.text_muted);
                ui.add_space(20.0);
                let _resp = ui.add_sized(
                    [ui.available_width() - 26.0, 20.0],
                    egui::TextEdit::singleline(&mut app.docs_query)
                        .hint_text(tl!("Search documentation by keyword..."))
                        .frame(egui::Frame::NONE),
                );
                if !app.docs_query.is_empty() && icons::button(ui, "x", 20.0, false, tl!("Clear")).clicked() {
                    app.docs_query.clear();
                }
            });
        });
    ui.add_space(10.0);

    let hits = matches(&app.docs_query);
    egui::ScrollArea::vertical().max_height(420.0).auto_shrink([false, false]).show(ui, |ui| {
        if hits.is_empty() {
            ui.add_space(12.0);
            ui.label(egui::RichText::new(tl!("No matching documentation found. Try another keyword like combine, export, edit, or sign.")).color(t.text_muted));
            return;
        }
        for entry in hits {
            egui::Frame::NONE
                .fill(t.panel)
                .stroke(Stroke::new(1.0, t.border))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(entry.question).font(theme::semibold(13.5)).color(t.text));
                    });
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(entry.answer).font(theme::regular(12.5)).color(t.text_muted));
                    if let Some(group) = entry.group {
                        ui.add_space(6.0);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if crate::widgets::pill_button(ui, tl!("Open tool"), false).clicked() {
                                open_tool = Some(group);
                            }
                        });
                    }
                });
            ui.add_space(6.0);
        }
    });

    if let Some(group) = open_tool {
        app.dialog = None;
        app.select_tool(group);
    }

    true
}
