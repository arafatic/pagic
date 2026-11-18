//! Pagic workspace: document actions, pinned folders and local recent files.

use std::collections::BTreeSet;

use egui::{Align2, CornerRadius, Rect, Sense, Shape, Stroke, vec2};

use crate::theme::{self, Tokens};
use crate::{PagicApp, icons, panels::human_size, widgets};

pub mod folders_ui;
mod recent_panel;
mod recent_thumbs;

/// Draw the workspace and recent documents. Actions reuse the existing workflows.
pub fn show(app: &mut PagicApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let sel_id = ui.scope_id().with("recent_selected_paths");
    let mut selected = ui.data(|d| d.get_temp::<BTreeSet<String>>(sel_id)).unwrap_or_default();
    selected.retain(|p| app.recent.iter().any(|r| &r.path == p));

    let thumb_cache_id = ui.scope_id().with("recent_thumb_cache");
    let thumb_cache = ui.data_mut(|d| d.get_temp_mut_or_insert_with::<recent_thumbs::RecentThumbCache>(thumb_cache_id, Default::default).clone());

    if let Some(panel_action) = recent_panel::show(ui, &t, &app.recent, &selected, &thumb_cache) {
        recent_panel::apply_action(app, panel_action, &mut selected);
    }

    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        egui::Frame::NONE.inner_margin(egui::Margin::symmetric(28, 28)).show(ui, |ui| {
            ui.label(egui::RichText::new(tl!("What would you like to do?")).font(theme::semibold(28.0)).color(t.text));
            ui.add_space(6.0);
            ui.label(egui::RichText::new(tl!("Open a file or pick a tool to get started.")).font(theme::regular(14.0)).color(t.text_muted));
            ui.add_space(28.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(12.0, 12.0);
                for (label, icon, detail) in [
                    ("Open", "folder-open", "Browse your files"),
                    ("Edit", "type", "Edit text and images"),
                    ("Merge", "files", "Combine documents"),
                    ("Export", "file-output", "Word, images and more"),
                    ("Compress", "file-down", "Reduce file size"),
                ] {
                    let width = ui.max_rect().width().clamp(1.0, 204.0);
                    let (rect, resp) = ui.allocate_exact_size(vec2(width, 116.0), Sense::click());
                    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
                    let press = theme::Press::track(ui, &resp);
                    let hovered = resp.hovered();
                    let stroke = if resp.has_focus() {
                        Stroke::new(2.0, t.accent)
                    } else if hovered {
                        Stroke::new(
                            1.0,
                            if t.dark() {
                                egui::Color32::from_white_alpha(35)
                            } else {
                                egui::Color32::from_black_alpha(25)
                            },
                        )
                    } else {
                        Stroke::NONE
                    };
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(6),
                        press.fill(t.card, t.hover, t.pressed),
                        stroke,
                        egui::StrokeKind::Inside,
                    );
                    let painter = ui.painter().with_clip_rect(rect);
                    icons::paint(ui, Rect::from_min_size(rect.min + vec2(16.0, 16.0), vec2(22.0, 22.0)), icon, 21.0, t.icon);
                    icons::paint(ui, Rect::from_min_size(rect.right_top() + vec2(-30.0, 14.0), vec2(16.0, 16.0)), "arrow-up-right", 14.0, t.text_faint);
                    painter.text(rect.min + vec2(16.0, 60.0), Align2::LEFT_CENTER, label, theme::semibold(14.0), t.text);
                    painter.text(rect.min + vec2(16.0, 84.0), Align2::LEFT_CENTER, detail, theme::regular(12.0), t.text_muted);
                    let resp = theme::hand(resp).on_hover_text(detail);
                    if label == "Export" {
                        egui::Popup::menu(&resp).show(|ui| {
                            for (format, command) in [
                                ("Word (.docx)", "export.docx"),
                                ("PowerPoint (.pptx)", "export.pptx"),
                                ("Images", "export.image"),
                                ("Plain text", "export.text"),
                                ("HTML", "export.html"),
                                ("Rich text (.rtf)", "export.rtf"),
                            ] {
                                if ui.button(format).clicked() {
                                    app.run_command(command);
                                    ui.close();
                                }
                            }
                        });
                    }
                    if label != "Export" && resp.clicked() {
                        match label {
                            "Open" => app.open_dialog(),
                            "Merge" => {
                                app.execute("page.combine");
                            }
                            "Edit" => app.run_command("edit.edit_text"),
                            "Compress" => app.run_command("optimize.reduce"),
                            _ => {}
                        }
                    }
                }
            });

            #[cfg(not(target_arch = "wasm32"))]
            {
                ui.add_space(26.0);
                crate::folders_ui::section(app, ui);
            }

            ui.add_space(26.0);
            let mut clear = false;
            let recent_view_id = ui.scope_id().with("recent_grid_view");
            let mut grid_view = ui.data(|d| d.get_temp::<bool>(recent_view_id)).unwrap_or(false);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tl!("Recent")).font(theme::semibold(17.0)));
                if !app.recent.is_empty() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if icons::button(ui, "list", 26.0, !grid_view, tl!("List view")).clicked() {
                            grid_view = false;
                            ui.data_mut(|d| d.insert_temp(recent_view_id, grid_view));
                        }
                        if icons::button(ui, "layout-grid", 26.0, grid_view, tl!("Thumbnail view")).clicked() {
                            grid_view = true;
                            ui.data_mut(|d| d.insert_temp(recent_view_id, grid_view));
                        }
                    });
                }
            });
            ui.add_space(8.0);
            if app.recent.is_empty() {
                let drop_zone_h = 164.0;
                let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), drop_zone_h), Sense::hover());
                let empty_label = tl!("No recent files yet. Drop a PDF here or");
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, false, &empty_label));
                let hovered = resp.hovered();
                ui.painter().rect_filled(
                    rect,
                    CornerRadius::same(6),
                    if hovered { t.hover } else { t.card },
                );
                let dash_color = if hovered {
                    t.accent
                } else if t.dark() {
                    egui::Color32::from_white_alpha(30)
                } else {
                    egui::Color32::from_black_alpha(25)
                };
                let dash_stroke = Stroke::new(1.0, dash_color);
                let p = [
                    rect.left_top(),
                    rect.right_top(),
                    rect.right_bottom(),
                    rect.left_bottom(),
                    rect.left_top(),
                ];
                for w in p.windows(2) {
                    ui.painter().extend(Shape::dashed_line(w, dash_stroke, 6.0, 5.0));
                }
                let center_y = rect.center().y;
                icons::paint(ui, Rect::from_center_size(egui::pos2(rect.center().x, center_y - 40.0), vec2(28.0, 28.0)), "file-text", 24.0, t.text_muted);
                ui.painter().text(
                    egui::pos2(rect.center().x, center_y - 6.0),
                    Align2::CENTER_CENTER,
                    &empty_label,
                    theme::regular(13.0),
                    t.text_muted,
                );
                let btn_rect = Rect::from_center_size(egui::pos2(rect.center().x, center_y + 34.0), vec2(116.0, 32.0));
                let btn_resp = ui.interact(btn_rect, ui.scope_id().with("empty_open_pdf"), Sense::click());
                btn_resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tl!("Open a PDF")));
                let btn_hovered = btn_resp.hovered();
                ui.painter().rect_filled(
                    btn_rect,
                    CornerRadius::same(5),
                    if btn_hovered { t.accent.linear_multiply(0.85) } else { t.accent },
                );
                ui.painter().text(
                    btn_rect.center(),
                    Align2::CENTER_CENTER,
                    tl!("Open a PDF"),
                    theme::semibold(13.0),
                    if t.dark() { egui::Color32::BLACK } else { egui::Color32::WHITE },
                );
                if theme::hand(btn_resp).clicked() {
                    app.open_dialog();
                }
            }
            let mut open = None;
            let mut remove = None;
            let mut toggle_select: Option<(String, bool)> = None;
            if !app.recent.is_empty() {
                // Select-all / deselect-all bar above the Recent list/grid when any items are selected.
                if !selected.is_empty() {
                    ui.horizontal(|ui| {
                        let all_selected = app.recent.iter().all(|r| selected.contains(&r.path));
                        let (cb_rect, cb_resp) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::click());
                        recent_panel::draw_checkbox(ui, &t, cb_rect, all_selected, !all_selected);
                        if theme::hand(cb_resp).on_hover_text(if all_selected { tl!("Deselect all") } else { tl!("Select all") }).clicked() {
                            if all_selected {
                                selected.clear();
                            } else {
                                for r in &app.recent {
                                    selected.insert(r.path.clone());
                                }
                            }
                        }
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(format!("{} {}", selected.len(), tl!("Selected"))).font(theme::medium(12.0)).color(t.text_muted));
                    });
                    ui.add_space(6.0);
                }
            }
            if grid_view && !app.recent.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(12.0, 12.0);
                    for r in &app.recent {
                        let is_selected = selected.contains(&r.path);
                        let card_w = ui.max_rect().width().clamp(1.0, 184.0);
                        let (rect, resp) = ui.allocate_exact_size(vec2(card_w, 196.0), Sense::click());
                        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &r.name));
                        let press = theme::Press::track(ui, &resp);
                        let hovered = ui.rect_contains_pointer(rect);
                        let focused = resp.has_focus();
                        let fill = if is_selected {
                            t.accent_soft
                        } else if hovered && press.hover < 0.04 && press.down < 0.04 {
                            t.hover
                        } else {
                            press.fill(t.card, t.hover, t.pressed)
                        };
                        let border = if is_selected { Stroke::new(1.5, t.accent) } else { Stroke::new(1.0, t.border) };
                        ui.painter().rect(rect, CornerRadius::same(6), fill, border, egui::StrokeKind::Inside);
                        if focused {
                            ui.painter().rect_stroke(rect, CornerRadius::same(6), Stroke::new(2.0, t.accent), egui::StrokeKind::Inside);
                        }
                        if focused && ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
                            remove = Some(r.path.clone());
                        }
                        let body = rect.translate(press.offset());
                        // Thumbnail preview area at the top of the card.
                        let preview_rect = Rect::from_min_max(body.min + vec2(10.0, 10.0), egui::pos2(body.max.x - 10.0, body.min.y + 126.0));
                        ui.painter().rect(
                            preview_rect,
                            CornerRadius::same(4),
                            t.panel,
                            Stroke::new(1.0, t.divider),
                            egui::StrokeKind::Inside,
                        );
                        if let Some(tex) = thumb_cache.get_or_request(ui.ctx(), &r.path) {
                            recent_thumbs::paint_fitted(ui, preview_rect, &tex, t.border);
                        } else {
                            // Fallback page sheet silhouette while rendering or if unavailable.
                            let sheet = Rect::from_center_size(preview_rect.center(), vec2(52.0, 68.0));
                            ui.painter().rect(
                                sheet,
                                CornerRadius::same(3),
                                t.card,
                                Stroke::new(1.0, t.border),
                                egui::StrokeKind::Inside,
                            );
                            icons::paint(ui, Rect::from_center_size(sheet.center(), vec2(24.0, 24.0)), "file-text", 22.0, t.icon);
                        }
                        if hovered || focused || is_selected {
                            let cb_rect = Rect::from_center_size(preview_rect.left_top() + vec2(14.0, 14.0), vec2(16.0, 16.0));
                            let cb = ui.interact(cb_rect, ui.scope_id().with(("select-recent-grid", &r.path)), Sense::click());
                            recent_panel::draw_checkbox(ui, &t, cb_rect, is_selected, false);
                            if theme::hand(cb).clicked() {
                                toggle_select = Some((r.path.clone(), true));
                            }
                        }
                        if hovered || focused {
                            let x_rect = Rect::from_center_size(preview_rect.right_top() + vec2(-16.0, 16.0), vec2(24.0, 24.0));
                            let x = ui.interact(x_rect, ui.scope_id().with(("remove-recent-grid", &r.path)), Sense::click());
                            let label = crate::i18n::fmt(tl!("Remove {name} from Recent"), &[("name", &r.name)]);
                            x.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &label));
                            if x.hovered() || x.has_focus() {
                                ui.painter().rect_filled(x_rect, CornerRadius::same(4), t.pressed);
                            }
                            icons::paint(ui, x_rect.shrink(5.0), "x", 14.0, t.icon);
                            if x.on_hover_text(tl!("Remove from Recent")).clicked() {
                                remove = Some(r.path.clone());
                            }
                        }
                        let text_w = (body.width() - 24.0).max(1.0);
                        widgets::row_text(ui, egui::pos2(body.left() + 12.0, preview_rect.bottom() + 18.0), r.name.as_str(), theme::medium(13.0), t.text, text_w);
                        let meta = format!("{}  ·  {}", crate::i18n::trn(crate::i18n::current(), r.pages as u64, "{n} page", "{n} pages"), human_size(r.size));
                        widgets::row_text(ui, egui::pos2(body.left() + 12.0, preview_rect.bottom() + 38.0), &meta, theme::regular(11.5), t.text_muted, text_w);
                        recent_context_menu(&resp, &r.path, &mut open, &mut remove);
                        let resp = theme::hand(resp).on_hover_text(&r.path);
                        if resp.double_clicked() {
                            open = Some(r.path.clone());
                        } else if resp.clicked() && toggle_select.is_none() {
                            let multi = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
                            toggle_select = Some((r.path.clone(), multi));
                        }
                    }
                });
            } else {
                for (idx, r) in app.recent.iter().enumerate() {
                    let is_selected = selected.contains(&r.path);
                    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
                    if idx > 0 {
                        ui.painter().hline(rect.x_range(), rect.top(), Stroke::new(0.5, t.divider.gamma_multiply(0.6)));
                    }
                    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &r.name));
                    let press = theme::Press::track(ui, &resp);
                    // The row, not its response: the pointer stays "in" the row over its remove button.
                    let hovered = ui.rect_contains_pointer(rect);
                    let focused = resp.has_focus();
                    if is_selected {
                        ui.painter().rect_filled(rect, CornerRadius::same(3), t.accent_soft);
                    } else if hovered && press.hover < 0.04 && press.down < 0.04 {
                        // The remove button sits in the row, so the row response may not be hovered.
                        ui.painter().rect_filled(rect, CornerRadius::same(3), t.hover);
                    } else {
                        press.wash(ui, rect, 3, false);
                    }
                    if focused {
                        ui.painter().rect_stroke(rect, CornerRadius::same(3), Stroke::new(2.0, t.accent), egui::StrokeKind::Inside);
                    }
                    if focused && ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
                        remove = Some(r.path.clone());
                    }
                    let show_cb = !selected.is_empty() || hovered || focused;
                    if show_cb {
                        let cb_rect = Rect::from_center_size(egui::pos2(rect.left() + 10.0, rect.center().y), vec2(15.0, 15.0));
                        let cb = ui.interact(cb_rect, ui.scope_id().with(("select-recent", &r.path)), Sense::click());
                        recent_panel::draw_checkbox(ui, &t, cb_rect, is_selected, false);
                        if theme::hand(cb).clicked() {
                            toggle_select = Some((r.path.clone(), true));
                        }
                    }
                    if hovered || focused {
                        // Remove just this file from the list (#430); the file itself is untouched.
                        let x_rect = Rect::from_center_size(rect.right_center() - vec2(26.0, 0.0), vec2(28.0, 28.0));
                        let x = ui.interact(x_rect, ui.scope_id().with(("remove-recent", &r.path)), Sense::click());
                        let label = crate::i18n::fmt(tl!("Remove {name} from Recent"), &[("name", &r.name)]);
                        x.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &label));
                        if x.hovered() || x.has_focus() {
                            ui.painter().rect_filled(x_rect, CornerRadius::same(3), t.pressed);
                        }
                        icons::paint(ui, x_rect.shrink(6.0), "x", 16.0, t.icon);
                        if x.on_hover_text(tl!("Remove from Recent")).clicked() {
                            remove = Some(r.path.clone());
                        }
                    }
                    let body = rect.translate(press.offset());
                    let icon_x = if show_cb { 26.0 } else { 0.0 };
                    let text_x = if show_cb { 56.0 } else { 34.0 };
                    icons::paint(ui, Rect::from_min_size(body.min + vec2(icon_x, 11.0), vec2(24.0, 24.0)), "file-text", 22.0, t.icon);
                    // Room on the right for the remove button.
                    let detail = ui.painter().text(
                        body.right_center() - vec2(48.0, 0.0),
                        Align2::RIGHT_CENTER,
                        format!("{}  ·  {}", crate::i18n::trn(crate::i18n::current(), r.pages as u64, "{n} page", "{n} pages"), human_size(r.size)),
                        theme::regular(12.0),
                        t.text_muted,
                    );
                    // The name and path stop short of the page count; hovering shows the whole path.
                    let width = detail.left() - 16.0 - (body.left() + text_x);
                    widgets::row_text(ui, body.min + vec2(text_x, 15.0), r.name.as_str(), theme::medium(13.5), t.text, width);
                    widgets::row_text(ui, body.min + vec2(text_x, 32.0), r.path.as_str(), theme::regular(11.0), t.text_faint, width);

                    recent_context_menu(&resp, &r.path, &mut open, &mut remove);

                    let resp = theme::hand(resp).on_hover_text(&r.path);
                    if resp.double_clicked() {
                        open = Some(r.path.clone());
                    } else if resp.clicked() && toggle_select.is_none() {
                        let multi = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
                        toggle_select = Some((r.path.clone(), multi));
                    }
                }
            }
            if !app.recent.is_empty() && recent_panel::clear_recent_footer(ui, &t) {
                clear = true;
            }
            if let Some((path, multi)) = toggle_select {
                if multi {
                    if !selected.remove(&path) {
                        selected.insert(path);
                    }
                } else if selected.len() == 1 && selected.contains(&path) {
                    selected.clear();
                } else {
                    selected.clear();
                    selected.insert(path);
                }
            }
            if let Some(p) = open {
                app.open_recent(&p);
            }
            if let Some(p) = remove {
                app.recent.retain(|r| r.path != p);
                selected.remove(&p);
            }
            if clear {
                selected.clear();
                app.execute("file.clear_recent");
            }
            ui.data_mut(|d| d.insert_temp(sel_id, selected));
        });
    });
}

fn recent_context_menu(resp: &egui::Response, path: &str, open: &mut Option<String>, remove: &mut Option<String>) {
    resp.context_menu(|ui| {
        if ui.button(tl!("Open")).clicked() {
            *open = Some(path.to_string());
            ui.close();
        }
        if ui.button(tl!("Remove from Recent")).clicked() {
            *remove = Some(path.to_string());
            ui.close();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let reveal_label = if cfg!(target_os = "macos") { tl!("Reveal in Finder") } else { tl!("Reveal in File Explorer") };
            if ui.button(reveal_label).clicked() {
                let _ = crate::menus::tab_context::reveal(std::path::Path::new(path));
                ui.close();
            }
        }
    });
}
