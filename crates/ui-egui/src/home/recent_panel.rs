//! Right-side inspector panel for selected Recent files on Home.

use std::collections::BTreeSet;

use egui::{Align2, CornerRadius, Rect, Sense, Stroke, vec2};

use super::recent_thumbs::{self, RecentThumbCache};
use crate::theme::{self, Tokens};
use crate::{PagicApp, RecentFile, icons, panels::human_size, widgets};

pub(super) enum PanelAction {
    ClearSelection,
    Open(Vec<String>),
    Combine(Vec<String>),
    OpenWithTool { path: String, tool: &'static str },
    OpenAndRun { path: String, command: &'static str },
    Remove(Vec<String>),
}

/// Draw the right-side inspector bar when one or more Recent files are selected.
pub(super) fn show(
    ui: &mut egui::Ui,
    t: &Tokens,
    recent: &[RecentFile],
    selected: &BTreeSet<String>,
    thumb_cache: &RecentThumbCache,
) -> Option<PanelAction> {
    let items: Vec<&RecentFile> = recent.iter().filter(|r| selected.contains(&r.path)).collect();
    if items.is_empty() {
        return None;
    }

    let mut action = None;
    let frame = egui::Frame::NONE
        .fill(t.panel)
        .inner_margin(egui::Margin::symmetric(16, 16))
        .stroke(Stroke::new(0.5, t.divider));

    egui::Panel::right("home_recent_inspector")
        .exact_size(264.0)
        .frame(frame)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if items.len() == 1 {
                let r = items[0];
                // Top-right close button to deselect.
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if icons::button(ui, "x", 24.0, false, tl!("Close")).clicked() {
                            action = Some(PanelAction::ClearSelection);
                        }
                    });
                });
                ui.add_space(4.0);

                // Large first-page thumbnail preview.
                let preview_w = ui.available_width();
                let preview_h = 160.0;
                let (preview_rect, preview_resp) = ui.allocate_exact_size(vec2(preview_w, preview_h), Sense::click());
                ui.painter().rect(
                    preview_rect,
                    CornerRadius::same(6),
                    t.card,
                    Stroke::new(1.0, t.border),
                    egui::StrokeKind::Inside,
                );
                if let Some(tex) = thumb_cache.get_or_request(ui.ctx(), &r.path) {
                    recent_thumbs::paint_fitted(ui, preview_rect, &tex, t.border);
                } else {
                    let sheet = Rect::from_center_size(preview_rect.center(), vec2(60.0, 78.0));
                    ui.painter().rect(sheet, CornerRadius::same(3), t.panel, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
                    icons::paint(ui, Rect::from_center_size(sheet.center(), vec2(26.0, 26.0)), "file-text", 24.0, t.icon);
                }
                if theme::hand(preview_resp).on_hover_text(tl!("Open")).clicked() {
                    action = Some(PanelAction::Open(vec![r.path.clone()]));
                }

                ui.add_space(12.0);
                ui.add(egui::Label::new(egui::RichText::new(&r.name).font(theme::semibold(13.5)).color(t.text)).truncate())
                    .on_hover_text(&r.name);
                ui.add_space(2.0);
                let meta = format!(
                    "PDF  ·  {}  ·  {}",
                    crate::i18n::trn(crate::i18n::current(), r.pages as u64, "{n} page", "{n} pages"),
                    human_size(r.size)
                );
                ui.label(egui::RichText::new(meta).font(theme::regular(11.5)).color(t.text_muted));
                ui.add(egui::Label::new(egui::RichText::new(&r.path).font(theme::regular(11.0)).color(t.text_faint)).truncate())
                    .on_hover_text(&r.path);

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                if action_row(ui, t, "folder-open", tl!("Open")).clicked() {
                    action = Some(PanelAction::Open(vec![r.path.clone()]));
                }
                if action_row(ui, t, "type", tl!("Edit PDF")).clicked() {
                    action = Some(PanelAction::OpenAndRun { path: r.path.clone(), command: "edit.edit_text" });
                }
                if action_row(ui, t, "file-output", tl!("Export PDF")).clicked() {
                    action = Some(PanelAction::OpenWithTool { path: r.path.clone(), tool: "export" });
                }
                if action_row(ui, t, "layout-grid", tl!("Organize Pages")).clicked() {
                    action = Some(PanelAction::OpenAndRun { path: r.path.clone(), command: "page.organize" });
                }
                if action_row(ui, t, "message-square", tl!("Comment")).clicked() {
                    action = Some(PanelAction::OpenWithTool { path: r.path.clone(), tool: "comment" });
                }
                if action_row(ui, t, "pen-tool", tl!("Fill & Sign")).clicked() {
                    action = Some(PanelAction::OpenWithTool { path: r.path.clone(), tool: "fill_sign" });
                }
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
                if action_row(ui, t, "circle-x", tl!("Remove From Recent")).clicked() {
                    action = Some(PanelAction::Remove(vec![r.path.clone()]));
                }
            } else {
                // Multi-selection header: icon + "{n} Selected" + close button.
                ui.horizontal(|ui| {
                    icons::paint(ui, Rect::from_min_size(ui.cursor().min, vec2(22.0, 22.0)), "files", 20.0, t.icon);
                    ui.add_space(26.0);
                    let title = format!("{} {}", items.len(), tl!("Selected"));
                    ui.label(egui::RichText::new(title).font(theme::semibold(14.0)).color(t.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if icons::button(ui, "x", 24.0, false, tl!("Close")).clicked() {
                            action = Some(PanelAction::ClearSelection);
                        }
                    });
                });
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                let paths: Vec<String> = items.iter().map(|r| r.path.clone()).collect();
                if action_row(ui, t, "folder-open", tl!("Open")).clicked() {
                    action = Some(PanelAction::Open(paths.clone()));
                }
                if action_row(ui, t, "files", tl!("Combine Files")).clicked() {
                    action = Some(PanelAction::Combine(paths.clone()));
                }
                if action_row(ui, t, "circle-x", tl!("Remove From Recent")).clicked() {
                    action = Some(PanelAction::Remove(paths));
                }
            }
        });

    action
}

fn action_row(ui: &mut egui::Ui, t: &Tokens, icon: &str, label: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let press = theme::Press::track(ui, &resp);
    press.wash(ui, rect, 6, false);
    let body = rect.translate(press.offset());
    icons::paint(ui, Rect::from_min_size(body.min + vec2(6.0, 8.0), vec2(18.0, 18.0)), icon, 17.0, t.icon);
    let text_w = (body.width() - 38.0).max(1.0);
    widgets::row_text(ui, body.left_center() + vec2(32.0, 0.0), label, theme::regular(13.0), t.text, text_w);
    theme::hand(resp)
}

/// Apply an action emitted by the right-side Recent inspector panel.
pub(super) fn apply_action(app: &mut PagicApp, action: PanelAction, selected: &mut BTreeSet<String>) {
    match action {
        PanelAction::ClearSelection => selected.clear(),
        PanelAction::Open(paths) => {
            for p in paths {
                app.open_recent(&p);
            }
        }
        PanelAction::Combine(paths) => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let path_bufs: Vec<std::path::PathBuf> = paths.into_iter().map(std::path::PathBuf::from).collect();
                app.use_paths(crate::files::FilePurpose::Combine, &path_bufs);
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = paths;
                app.open_combine_tab();
            }
        }
        PanelAction::OpenWithTool { path, tool } => {
            app.open_recent(&path);
            app.select_tool(tool);
        }
        PanelAction::OpenAndRun { path, command } => {
            app.open_recent(&path);
            app.run_command(command);
        }
        PanelAction::Remove(paths) => {
            let remove_set: BTreeSet<String> = paths.into_iter().collect();
            app.recent.retain(|r| !remove_set.contains(&r.path));
            for p in remove_set {
                selected.remove(&p);
            }
        }
    }
}

/// Draw a small selection checkbox (unchecked, checked, or indeterminate `-`).
pub(super) fn draw_checkbox(ui: &egui::Ui, t: &Tokens, rect: Rect, checked: bool, indeterminate: bool) {
    let fill = if checked || indeterminate { t.accent } else { egui::Color32::TRANSPARENT };
    let stroke = if checked || indeterminate {
        Stroke::new(1.0, t.accent)
    } else {
        Stroke::new(1.2, t.text_muted)
    };
    ui.painter().rect(rect, CornerRadius::same(3), fill, stroke, egui::StrokeKind::Inside);
    let fg = if t.dark() { egui::Color32::BLACK } else { egui::Color32::WHITE };
    if checked {
        let c = rect.center();
        let s = Stroke::new(1.8, fg);
        ui.painter().line_segment([c + vec2(-3.5, 0.2), c + vec2(-1.0, 2.8)], s);
        ui.painter().line_segment([c + vec2(-1.0, 2.8), c + vec2(3.8, -2.5)], s);
    } else if indeterminate {
        let c = rect.center();
        ui.painter().line_segment([c + vec2(-3.5, 0.0), c + vec2(3.5, 0.0)], Stroke::new(1.8, fg));
    }
}

/// Draw the "Clear Recent" action at the bottom right of the Recent section.
pub(super) fn clear_recent_footer(ui: &mut egui::Ui, t: &Tokens) -> bool {
    let mut clear = false;
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let label = tl!("Clear Recent");
            let font = theme::medium(12.5);
            let w = widgets::text_width(ui, label, &font) + 12.0;
            let (rect, resp) = ui.allocate_exact_size(vec2(w, 26.0), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Clear Recent Files")));
            if resp.hovered() {
                ui.painter().rect_filled(rect, CornerRadius::same(4), t.hover);
            }
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, font, t.accent_text);
            clear = theme::hand(resp).on_hover_text(tl!("Clear Recent Files")).clicked();
        });
    });
    clear
}
