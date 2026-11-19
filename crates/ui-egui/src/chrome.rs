//! Window chrome: tab strip (with the integrated macOS title bar), mode bar, right rail.

use craft_ui::panes::{Edge, Rail, RailItem, RailRequest};
use egui::{Align, CornerRadius, Layout, Rect, Sense, Stroke, vec2};

use crate::canvas::{DocView, Fit, PageLayout};
use crate::theme::{self, ThemePreference, Tokens};
use crate::{Dialog, PagicApp, PropsTab, RightPanel, icons, widgets, window_controls};

/// Height of the tab strip when it doubles as the macOS title bar.
pub const MAC_TITLE_BAR: f32 = 38.0;
/// The traffic lights as AppKit lays them out: button diameter, centre-to-centre pitch and the
/// first button's left edge (8–10 pt depending on the macOS version; the larger value keeps the
/// gap after them from shrinking).
const MAC_LIGHT: f32 = 12.0;
const MAC_LIGHT_PITCH: f32 = 20.0;
const MAC_LIGHTS_LEFT: f32 = 10.0;
/// Where the strip's content starts to the right of the traffic lights: the end of the three
/// lights plus the same gap the lights keep from the top of the window.
pub const MAC_CONTENT_LEFT: f32 = MAC_LIGHTS_LEFT + 2.0 * MAC_LIGHT_PITCH + MAC_LIGHT + (MAC_TITLE_BAR - MAC_LIGHT) / 2.0;
/// The strip and its controls everywhere else.
const TAB_STRIP: f32 = 38.0;
const TAB_HEIGHT: f32 = 30.0;
const STRIP_BUTTON: f32 = 28.0;

/// Draw the window's tab strip: on macOS, a single unified bar with Home, tabs, search and actions;
/// on Windows/Linux, a 2-bar layout with a black top menu+window-controls bar (Bar 1) above the
/// unified Home + tabs + search + actions bar (Bar 2).
pub fn tab_strip(app: &mut PagicApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let mac = cfg!(target_os = "macos") && app.integrated_titlebar;
    let fullscreen = ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
    let controls = app
        .window_controls
        .clone()
        .filter(|_| app.draws_window_frame() && !fullscreen)
        .unwrap_or_else(|| window_controls::Layout { left: Vec::new(), right: Vec::new() });

    // Bar 1 (Windows/Linux): black title/menu bar with Pagic logo + File..Help menus on the left,
    // and Minimize/Maximize/Close window controls on the right.
    if !mac && (app.native_menu.is_none() || !controls.right.is_empty() || !controls.left.is_empty()) {
        let bar_bg = egui::Color32::from_rgb(16, 16, 18);
        let bar_fg = egui::Color32::from_gray(235);
        let bar_frame = egui::Frame::NONE.fill(bar_bg).inner_margin(egui::Margin { left: 10, right: 10, top: 0, bottom: 0 });
        egui::Panel::top("win_menu_titlebar")
            .exact_size(34.0)
            .show_separator_line(false)
            .frame(bar_frame)
            .show(ui, |ui| {
                let full = ui.max_rect();
                let drag = ui.interact(full, ui.scope_id().with("win_titledrag"), Sense::click_and_drag());
                if drag.drag_started() {
                    window_controls::desktop_grab(ui.ctx(), egui::ViewportCommand::StartDrag);
                }
                if drag.double_clicked() {
                    let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
                }
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    if !controls.left.is_empty() {
                        ui.scope(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            window_controls::buttons_tinted(ui, &controls.left, bar_fg);
                        });
                        ui.add_space(4.0);
                    }
                    // Pagic app logo on the far left of Bar 1.
                    let logo_bytes: &[u8] = include_bytes!("../../../assets/app-icon/pagic.svg");
                    let logo = egui::Image::from_bytes("bytes://app-icon/pagic.svg", logo_bytes)
                        .fit_to_exact_size(vec2(18.0, 18.0));
                    ui.add(logo);
                    ui.add_space(2.0);
                    if app.native_menu.is_none() {
                        ui.scope(|ui| {
                            let v = ui.visuals_mut();
                            let hover_bg = egui::Color32::from_rgb(46, 50, 58);
                            let open_bg = egui::Color32::from_rgb(60, 65, 75);
                            v.override_text_color = Some(bar_fg);
                            v.widgets.inactive.fg_stroke.color = bar_fg;
                            v.widgets.hovered.weak_bg_fill = hover_bg;
                            v.widgets.hovered.bg_fill = hover_bg;
                            v.widgets.hovered.fg_stroke.color = egui::Color32::WHITE;
                            v.widgets.active.weak_bg_fill = open_bg;
                            v.widgets.active.bg_fill = open_bg;
                            v.widgets.active.fg_stroke.color = egui::Color32::WHITE;
                            v.widgets.open.weak_bg_fill = open_bg;
                            v.widgets.open.bg_fill = open_bg;
                            v.widgets.open.fg_stroke.color = egui::Color32::WHITE;
                            app.menu_bar_ui(ui);
                        });
                    }
                    if !controls.right.is_empty() {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let size = egui::vec2(window_controls::width(&controls.right) - window_controls::SIDE_GAP, window_controls::BUTTON);
                            ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                window_controls::buttons_tinted(ui, &controls.right, bar_fg);
                            });
                        });
                    }
                });
            });
    }

    let left = if mac && !fullscreen {
        MAC_CONTENT_LEFT as i8
    } else if mac {
        10
    } else {
        10
    };

    let (tab_h, button_size) = (TAB_HEIGHT, STRIP_BUTTON);
    let strip_height = if mac { MAC_TITLE_BAR } else { TAB_STRIP };
    let frame = egui::Frame::NONE.fill(t.titlebar).inner_margin(egui::Margin { left, right: 10, top: 0, bottom: 0 });
    egui::Panel::top("tab_strip")
        .exact_size(strip_height)
        .show_separator_line(false)
        .frame(frame)
        .show(ui, |ui| {
            let full = ui.max_rect();
            ui.painter().hline(full.x_range(), full.bottom(), Stroke::new(0.5, t.divider.gamma_multiply(0.6)));
            if mac {
                let drag = ui.interact(full, ui.scope_id().with("titledrag"), Sense::click_and_drag());
                if drag.drag_started() {
                    window_controls::desktop_grab(ui.ctx(), egui::ViewportCommand::StartDrag);
                }
                if drag.double_clicked() {
                    let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
                }
            }
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                // Home button on the far left of Bar 2 (all platforms).
                let is_home = app.active.is_none() && !app.combine_showing();
                let tip = tl!("Home");
                let home_btn = icons::button(ui, "house", button_size, is_home, tip);
                if home_btn.clicked() {
                    app.active = None;
                    app.combine_tab.focused = false;
                }
                ui.add_space(4.0);
                let right_actions_budget: f32 = 440.0;
                let controls_width = 18.0 + right_actions_budget;
                let mut tabs_width = (ui.available_width() - controls_width).max(0.0);
                // Tabs shrink to share the strip (names end in "…") and scroll only once they reach
                // their narrowest; then arrows either side say so and step through them.
                let mut natural: Vec<f32> = Vec::with_capacity(app.views.len() + 1);
                for i in 0..app.views.len() {
                    if let Some(name) = crate::canvas::tab_label(app, i) {
                        natural.push(tab_natural_width(ui, &t, &name));
                    }
                }
                if app.combine_tab.open {
                    natural.push(tab_natural_width(ui, &t, tl!("Combine files")));
                }
                let gap = ui.spacing().item_spacing.x;
                let active_tab_index = if app.combine_showing() {
                    Some(app.views.len())
                } else {
                    app.active
                };
                let (mut tab_widths, overflow) = compute_tab_widths(&natural, active_tab_index, tabs_width, gap);
                if overflow {
                    tabs_width = (tabs_width - 2.0 * (TAB_ARROW + gap)).max(0.0);
                    tab_widths = compute_tab_widths(&natural, active_tab_index, tabs_width, gap).0;
                }
                // Where the tabs were scrolled to last frame, and how far they can go.
                let scroll_id = ui.scope_id().with("tab_scroll");
                let (offset, max_offset) = ui.data(|d| d.get_temp::<(f32, f32)>(scroll_id)).unwrap_or((0.0, 0.0));
                let mut scroll_to = None;
                if overflow && arrow(ui, "chevron-left", offset > 0.5, tl!("Scroll tabs left")) {
                    scroll_to = Some((offset - (TAB_ICON_WIDTH + gap)).max(0.0));
                }
                // Keyed by document, not index: a tab dragged past the active one moves it without
                // the strip jumping to it.
                let state = (app.active_ids().map(|(_, id)| id), app.views.len(), app.combine_showing());
                let changed = ui.data_mut(|data| {
                    let id = ui.scope_id().with("active_tab");
                    let changed = data.get_temp::<(Option<pagic_engine::DocId>, usize, bool)>(id) != Some(state);
                    data.insert_temp(id, state);
                    changed
                });
                let mut action = None;
                let mut close = None;
                // The pointer's x while a tab is dragged, for scrolling at the strip's edges.
                let mut drag_x = None;
                ui.scope(|ui| {
                    ui.style_mut().always_scroll_the_only_direction = true;
                    let mut area = egui::ScrollArea::horizontal().id_salt("document_tabs").max_width(tabs_width).auto_shrink([true, true]);
                    if let Some(x) = ui.data_mut(|d| d.remove_temp::<f32>(scroll_id.with("to"))) {
                        area = area.horizontal_scroll_offset(x);
                    }
                    let out = area.show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            // Each document tab's index and rect, and the one being dragged
                            // (with whether it was just released).
                            let mut slots = Vec::with_capacity(app.views.len());
                            let mut dragged = None;
                            for i in 0..app.views.len() {
                                let Some(doc) = app.session.get(app.views[i].id) else { continue };
                                let Some(name) = crate::canvas::tab_label(app, i) else { continue };
                                let dirty = doc.dirty;
                                // A tab showing the document's title names its file on hover.
                                let file = (name != doc.name).then_some(doc.name.as_str());
                                let width = tab_widths.get(i).copied().unwrap_or(TAB_ICON_WIDTH);
                                let response = tab(
                                    ui,
                                    TabSpec {
                                        id: ui.scope_id().with(("document-tab", app.views[i].id)),
                                        theme: &t,
                                        icon: "file-text",
                                        name: &name,
                                        file,
                                        dirty,
                                        active: app.active == Some(i),
                                        index: i,
                                        width,
                                        height: tab_h,
                                        draggable: true,
                                    },
                                    &mut close,
                                );
                                slots.push((i, response.rect));
                                if response.dragged() || response.drag_stopped() {
                                    dragged = Some((i, response.drag_stopped()));
                                }
                                if changed && app.active == Some(i) && !app.combine_showing() {
                                    response.scroll_to_me(Some(Align::Center));
                                }
                                let id = app.views[i].id;
                                let reveal = cfg!(not(target_arch = "wasm32")) && doc.path.is_some();
                                response.context_menu(|ui| {
                                    if ui.button(tl!("Close Tab")).clicked() {
                                        action = Some(crate::tab_context::Action::Close(id));
                                        ui.close();
                                    }
                                    if ui.add_enabled(!app.closed_tabs.is_empty(), egui::Button::new(tl!("Reopen Closed Tab"))).clicked() {
                                        action = Some(crate::tab_context::Action::Reopen);
                                        ui.close();
                                    }
                                    let label = if cfg!(target_os = "macos") { tl!("Reveal in Finder") } else { tl!("Reveal in File Explorer") };
                                    if ui.add_enabled(reveal, egui::Button::new(label)).clicked() {
                                        action = Some(crate::tab_context::Action::Reveal(id));
                                        ui.close();
                                    }
                                });
                                if response.clicked() {
                                    app.active = Some(i);
                                }
                                // A middle click closes the tab, as in Acrobat and browsers (#1853);
                                // unsaved changes ask first, as the close button does.
                                if response.middle_clicked() {
                                    close = Some(i);
                                }
                            }
                            // Dragging a tab moves it (#1537): a line marks the gap it will land
                            // in, and releasing the button drops it there.
                            if let Some((from, released)) = dragged
                                && let Some(x) = ui.ctx().pointer_latest_pos().map(|p| p.x)
                            {
                                let to = drop_index(&slots, from, x);
                                if released {
                                    if close.is_none() && action.is_none() {
                                        app.move_tab(from, to);
                                    }
                                } else {
                                    drag_x = Some(x);
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                                    if let Some(marker) = drop_marker(&slots, from, to, ui.spacing().item_spacing.x) {
                                        let span = slots.first().map_or(ui.max_rect().y_range(), |(_, r)| r.y_range());
                                        // In front of the first tab the gap lies outside the scroll area's
                                        // clip rect (#2225): keep the whole line inside it.
                                        let clip = ui.clip_rect();
                                        let marker = marker.clamp(clip.left() + 1.0, (clip.right() - 1.0).max(clip.left() + 1.0));
                                        ui.painter().vline(marker, span.shrink(4.0), Stroke::new(2.0, t.accent));
                                    }
                                }
                            }
                            if let Some(i) = close
                                && let Some(view) = app.views.get(i)
                            {
                                action.get_or_insert(crate::tab_context::Action::Close(view.id));
                            }
                            if let Some(action) = action {
                                app.tab_action(action);
                            }
                            if app.combine_tab.open {
                                // After the document tabs; its index can't clash with theirs.
                                let mut close = None;
                                let width = tab_widths.get(app.views.len()).copied().unwrap_or(TAB_ICON_WIDTH);
                                let response = tab(
                                    ui,
                                    TabSpec {
                                        id: ui.scope_id().with("combine-tab"),
                                        theme: &t,
                                        icon: "files",
                                        name: tl!("Combine files"),
                                        file: None,
                                        dirty: false,
                                        active: app.combine_showing(),
                                        index: usize::MAX,
                                        width,
                                        height: tab_h,
                                        draggable: false,
                                    },
                                    &mut close,
                                );
                                if changed && app.combine_showing() {
                                    response.scroll_to_me(Some(Align::Center));
                                }
                                if response.clicked() {
                                    app.open_combine_tab();
                                }
                                if close.is_some() || response.middle_clicked() {
                                    app.close_combine_tab();
                                }
                            }
                        });
                    });
                    let max = (out.content_size.x - out.inner_rect.width()).max(0.0);
                    ui.data_mut(|d| d.insert_temp(scroll_id, (out.state.clamped_offset().x, max)));
                    // A tab dragged to either edge of a strip that scrolls brings the hidden tabs in.
                    if let Some(x) = drag_x
                        && max > 0.0
                    {
                        let step = if x < out.inner_rect.left() + TAB_DRAG_EDGE {
                            -TAB_DRAG_STEP
                        } else if x > out.inner_rect.right() - TAB_DRAG_EDGE {
                            TAB_DRAG_STEP
                        } else {
                            0.0
                        };
                        if step != 0.0 {
                            ui.data_mut(|d| d.insert_temp(scroll_id.with("to"), (out.state.clamped_offset().x + step).clamp(0.0, max)));
                            ui.ctx().request_repaint();
                        }
                    }
                });
                if overflow && arrow(ui, "chevron-right", offset < max_offset - 0.5, tl!("Scroll tabs right")) {
                    scroll_to = Some((offset + TAB_ICON_WIDTH + gap).min(max_offset));
                }
                if let Some(x) = scroll_to {
                    ui.data_mut(|d| d.insert_temp(scroll_id.with("to"), x));
                    ui.ctx().request_repaint();
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    render_right_actions(app, ui, &t, button_size);
                });
            });
        });
}

/// The right-click menu on the title bar's theme button.
fn appearance_menu(app: &mut PagicApp, ui: &mut egui::Ui) {
    if ui.button(tl!("Appearance settings…")).clicked() {
        app.dialog = Some(Dialog::Preferences);
        ui.close();
    }
}

/// A tab's chrome around its name: the icon on the left, the close button on the right.
const TAB_CHROME: f32 = 64.0;
/// The minimum width of an inactive tab in icon-only mode (Safari style).
pub const TAB_ICON_WIDTH: f32 = 38.0;
/// The narrowest an inactive tab shrinks before collapsing into icon-only mode.
pub const TAB_MIN_TEXT_WIDTH: f32 = 56.0;
/// The minimum width of an active tab so its title stays legible.
pub const TAB_ACTIVE_MIN_WIDTH: f32 = 150.0;
/// The arrows either side of tabs that don't fit.
const TAB_ARROW: f32 = 24.0;

/// Compute tab widths using the Safari smart tab sizing model:
/// - When space allows, tabs keep natural widths.
/// - As space tightens, inactive tabs shrink while the active tab keeps its title.
/// - When crowded, inactive tabs collapse to compact 38px icon-only pills, leaving maximum room for the active tab.
/// - When even icon-only pills don't fit, horizontal scroll arrows appear.
fn compute_tab_widths(
    natural: &[f32],
    active_idx: Option<usize>,
    budget: f32,
    gap: f32,
) -> (Vec<f32>, bool) {
    let n = natural.len();
    if n == 0 {
        return (Vec::new(), false);
    }
    let total_gap = gap * (n - 1) as f32;
    let sum_natural: f32 = natural.iter().sum::<f32>() + total_gap;

    if sum_natural <= budget {
        return (natural.to_vec(), false);
    }

    if n == 1 {
        return (vec![budget.max(TAB_ICON_WIDTH).min(natural[0])], false);
    }

    let active = active_idx.unwrap_or(0).min(n - 1);
    let active_nat = natural[active].min(240.0);

    // 1. Keep active tab wide (up to 240px, flexing down to TAB_ACTIVE_MIN_WIDTH if needed)
    //    so inactive tabs shrink gradually with ellipsized titles down to TAB_MIN_TEXT_WIDTH.
    let inactive_count = (n - 1) as f32;
    let active_max = active_nat.max(TAB_ACTIVE_MIN_WIDTH.min(natural[active]));
    let active_min = TAB_ACTIVE_MIN_WIDTH.min(natural[active]);
    let needed_for_inactive_text = inactive_count * TAB_MIN_TEXT_WIDTH;
    let available_for_active = budget - total_gap - needed_for_inactive_text;
    let active_w = available_for_active.clamp(active_min, active_max);
    let share = (budget - active_w - total_gap) / inactive_count;

    if share >= TAB_MIN_TEXT_WIDTH {
        let mut widths = Vec::with_capacity(n);
        for (i, &nat) in natural.iter().enumerate() {
            if i == active {
                widths.push(active_w.min(nat));
            } else {
                widths.push(share.min(nat));
            }
        }
        return (widths, false);
    }

    // 2. Inactive tabs cannot fit text comfortably: collapse inactive tabs to icon-only pills (38px)!
    let inactive_total = inactive_count * TAB_ICON_WIDTH + total_gap;
    let rem_for_active = budget - inactive_total;

    if rem_for_active >= TAB_ACTIVE_MIN_WIDTH {
        let active_w = rem_for_active.min(natural[active]);
        let mut widths = Vec::with_capacity(n);
        for i in 0..n {
            if i == active {
                widths.push(active_w);
            } else {
                widths.push(TAB_ICON_WIDTH);
            }
        }
        return (widths, false);
    }

    // 3. Very tight space: active tab at TAB_ACTIVE_MIN_WIDTH
    let min_needed = inactive_total + TAB_ACTIVE_MIN_WIDTH;
    if min_needed <= budget {
        let active_w = (budget - inactive_total).max(TAB_ACTIVE_MIN_WIDTH);
        let mut widths = Vec::with_capacity(n);
        for i in 0..n {
            if i == active {
                widths.push(active_w);
            } else {
                widths.push(TAB_ICON_WIDTH);
            }
        }
        return (widths, false);
    }

    // 4. Overflow: needs scroll area and scroll arrows
    let mut widths = Vec::with_capacity(n);
    for i in 0..n {
        if i == active {
            widths.push(TAB_ACTIVE_MIN_WIDTH.min(natural[active]));
        } else {
            widths.push(TAB_ICON_WIDTH);
        }
    }
    (widths, true)
}

/// How close to either end of a scrolling strip a dragged tab scrolls it, and how far per frame.
const TAB_DRAG_EDGE: f32 = 24.0;
const TAB_DRAG_STEP: f32 = 8.0;

/// Where the tab at `from` lands when dropped with the pointer at `x`: past every other tab
/// whose centre is left of the pointer. `slots` are the tabs' indices and rects, in order.
fn drop_index(slots: &[(usize, Rect)], from: usize, x: f32) -> usize {
    slots.iter().filter(|(i, r)| *i != from && r.center().x < x).count()
}

/// The x of the line marking where the tab at `from` drops at `to`: in the gap before the other
/// tab it lands in front of, or after the last. `None` when it would stay where it is.
fn drop_marker(slots: &[(usize, Rect)], from: usize, to: usize, gap: f32) -> Option<f32> {
    if slots.iter().position(|(i, _)| *i == from) == Some(to) {
        return None;
    }
    let mut others = slots.iter().filter(|(i, _)| *i != from).map(|(_, r)| r);
    match to.checked_sub(1) {
        None => others.next().map(|r| r.left() - gap / 2.0),
        Some(before) => others.nth(before).map(|r| r.right() + gap / 2.0),
    }
}

/// A scroll arrow beside the tabs; dimmed (and inert) at its end. True when clicked.
fn arrow(ui: &mut egui::Ui, icon: &str, enabled: bool, tip: &str) -> bool {
    ui.add_enabled_ui(enabled, |ui| icons::button(ui, icon, TAB_ARROW, false, tip)).inner.clicked()
}

/// A tab name, at most 28 characters (longer names end in "…").
fn tab_name(name: &str) -> String {
    if name.chars().count() > 28 { format!("{}…", name.chars().take(27).collect::<String>()) } else { name.to_string() }
}

/// The width a tab takes when nothing has to shrink.
fn tab_natural_width(ui: &egui::Ui, t: &Tokens, name: &str) -> f32 {
    let label = tab_name(name);
    ui.fonts_mut(|f| f.layout_no_wrap(label, theme::regular(13.0), t.text).size().x) + TAB_CHROME
}

/// Draw one tab with its icon, elided label, close button and unsaved dot; records a close request and
/// returns the tab response for drag handling.
struct TabSpec<'a> {
    id: egui::Id,
    theme: &'a Tokens,
    icon: &'a str,
    name: &'a str,
    file: Option<&'a str>,
    dirty: bool,
    active: bool,
    index: usize,
    width: f32,
    height: f32,
    draggable: bool,
}

fn tab(ui: &mut egui::Ui, spec: TabSpec<'_>, close: &mut Option<usize>) -> egui::Response {
    let TabSpec { id, theme: t, icon, name, file, dirty, active, index, width, height, draggable } = spec;
    let font = theme::regular(13.0);
    let mut label = tab_name(name);
    let is_icon_only = width <= 48.0 && !active;
    let compact_close = !active && width < 96.0;

    // Full strip height so the active tab sits on the mode bar instead of floating in the title bar.
    let height = ui.available_height().max(height);
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let a11y = if dirty { format!("{name} (edited)") } else { name.to_string() };
    let sense = if draggable { Sense::click_and_drag() } else { Sense::click() };
    let resp = craft_ui::tabs::Tab::new(id, &a11y, active).sense(sense).focus_stroke(Stroke::new(1.0, t.accent)).show_at(ui, rect, |ui, resp| {
        let press = theme::Press::track(ui, resp);
        let inactive_bg = t.chrome.lerp_to_gamma(t.titlebar, 0.45);
        let bg = if active {
            press.fill(t.chrome, t.chrome.lerp_to_gamma(t.hover, 0.65), t.hover)
        } else {
            press.fill(inactive_bg, t.hover, t.pressed)
        };
        let tab_radius = CornerRadius { nw: 7, ne: 7, sw: 0, se: 0 };
        let tab_stroke = if active {
            Stroke::new(0.75, t.divider)
        } else {
            Stroke::new(0.5, t.divider.gamma_multiply(0.65))
        };
        ui.painter().rect(rect, tab_radius, bg, tab_stroke, egui::StrokeKind::Inside);

        if is_icon_only {
            // Safari-style icon-only compact pill:
            // Centered icon when idle
            let icon_rect = Rect::from_center_size(rect.center(), vec2(16.0, 16.0));
            icons::paint(
                ui,
                icon_rect,
                icon,
                15.0,
                if active { t.accent } else { t.text_muted },
            );
            if dirty {
                let dot_pos = egui::pos2(rect.right() - 6.0, rect.top() + 6.0);
                ui.painter().circle_filled(dot_pos, 3.0, if active { t.accent } else { t.text_muted });
            }
        } else {
            icons::paint(
                ui,
                Rect::from_min_size(rect.min + vec2(8.0, (height - 16.0) / 2.0), vec2(16.0, 16.0)),
                icon,
                15.0,
                if active { t.accent } else { t.text_muted },
            );

            let chrome = if compact_close {
                if dirty { 44.0 } else { 36.0 }
            } else {
                TAB_CHROME
            };
            let room = (width - chrome).max(0.0);
            label = crate::panels::ellipsized_prefix(&label, |s| crate::bidi::layout_ltr(ui, s.to_owned(), font.clone(), t.text).size().x <= room);

            let label_color = if active { t.text } else { t.text_muted };
            let label_galley = crate::bidi::layout_ltr(ui, label.clone(), font.clone(), label_color);
            let text_w = label_galley.size().x;
            let title_pos = rect.min + vec2(28.0, (rect.height() - label_galley.size().y) / 2.0);
            ui.painter().galley(title_pos, label_galley.clone(), label_color);

            if dirty {
                let dot_x = (title_pos.x + text_w + 6.0).min(rect.right() - 6.0);
                let dot_pos = egui::pos2(dot_x, rect.center().y);
                ui.painter().circle_filled(dot_pos, 3.0, if active { t.accent } else { t.text_muted });
            }
        }
    });

    let x_clicked = if is_icon_only {
        if resp.hovered() {
            let x_rect = Rect::from_center_size(rect.right_center() - vec2(10.0, 0.0), vec2(14.0, 14.0));
            let x = ui.interact(x_rect, id.with("close"), Sense::click());
            let x_press = theme::Press::track(ui, &x);
            x_press.wash(ui, x_rect, 3, false);
            ui.painter().rect_filled(x_rect, CornerRadius::same(3), t.hover);
            icons::paint(ui, x_rect.translate(x_press.offset()), "x", 11.0, t.text);
            theme::hand(x).clicked()
        } else {
            false
        }
    } else if compact_close {
        if resp.hovered() {
            let x_rect = Rect::from_center_size(rect.right_center() - vec2(12.0, 0.0), vec2(16.0, 16.0));
            let x = ui.interact(x_rect, id.with("close"), Sense::click());
            let x_press = theme::Press::track(ui, &x);
            x_press.wash(ui, x_rect, 3, false);
            ui.painter().rect_filled(x_rect, CornerRadius::same(3), t.hover);
            icons::paint(ui, x_rect.translate(x_press.offset()), "x", 11.0, t.text);
            theme::hand(x).clicked()
        } else {
            false
        }
    } else {
        let x_rect = Rect::from_center_size(rect.right_center() - vec2(16.0, 0.0), vec2(20.0, 20.0));
        let x = ui.interact(x_rect, id.with("close"), Sense::click());
        let x_press = theme::Press::track(ui, &x);
        x_press.wash(ui, x_rect, 4, false);
        if active || resp.hovered() || x.hovered() || !dirty {
            icons::paint(ui, x_rect.translate(x_press.offset()), "x", 13.0, t.text_muted);
        } else if dirty {
            ui.painter().circle_filled(x_rect.center(), 4.0, if active { t.text } else { t.text_muted });
        }
        theme::hand(x).clicked()
    };

    if x_clicked {
        *close = Some(index);
    }
    let shown = match file {
        Some(file) => format!("{name}\n{file}"),
        None => name.to_string(),
    };
    theme::hand(resp).on_hover_text(if dirty { crate::i18n::fmt(tl!("{name} — unsaved changes"), &[("name", &shown)]) } else { shown })
}

fn render_right_actions(app: &mut PagicApp, ui: &mut egui::Ui, t: &Tokens, button_size: f32) {
    ui.spacing_mut().item_spacing.x = 4.0;

    // Theme toggle on the far right.
    let (icon, label) = match app.theme_preference {
        ThemePreference::System => ("monitor", tl!("Use system setting")),
        ThemePreference::Light => ("sun", tl!("Light gray")),
        ThemePreference::Dark => ("moon", tl!("Dark gray")),
    };
    let tip = format!("{}: {label}", tl!("Display theme"));
    let theme_btn = icons::button(ui, icon, button_size, false, &tip);
    if theme_btn.clicked() {
        app.execute(match app.theme_preference {
            ThemePreference::System => "view.theme.light",
            ThemePreference::Light => "view.theme.dark",
            ThemePreference::Dark => "view.theme.system",
        });
    }
    egui::Popup::context_menu(&theme_btn).id(theme_btn.id.with("theme_menu_mode")).show(|ui| appearance_menu(app, ui));

    // Help / About Pagic.
    let tip = crate::commands::command_tip(ui.ctx(), tl!("About Pagic ({key})"), "help.about");
    if icons::button(ui, "circle-help", button_size, false, &tip).clicked() {
        app.dialog = Some(Dialog::About);
    }

    // Divider before help & theme.
    ui.add_space(4.0);
    ui.painter().vline(ui.cursor().left(), ui.max_rect().y_range().shrink(8.0), Stroke::new(1.0, t.divider));
    ui.add_space(4.0);

    // Document action icons: Print, Save, Info.
    let has_doc = app.active.is_some();
    ui.add_enabled_ui(has_doc, |ui| {
        let tip = crate::commands::command_tip(ui.ctx(), tl!("Print ({key})"), "print.dialog");
        let may_print = !has_doc || pagic_engine::commands::command("print.dialog").is_some_and(|c| app.command_enabled(c));
        let mut print = ui.add_enabled_ui(may_print, |ui| icons::button(ui, "printer", button_size, false, &tip)).inner;
        if !may_print {
            print = print.on_disabled_hover_text(crate::print_refused());
        }
        if print.clicked() {
            app.run_command("print.dialog");
        }
        let tip = crate::commands::command_tip(ui.ctx(), tl!("Save ({key})"), "file.save");
        if icons::button(ui, "save", button_size, false, &tip).clicked() {
            app.run_command("file.save");
        }
        let tip = crate::commands::command_tip(ui.ctx(), tl!("Document properties ({key})"), "file.properties");
        if icons::button(ui, "info", button_size, false, &tip).clicked() {
            app.dialog = Some(Dialog::Properties(PropsTab::Description));
        }
    });

    // Divider between search and document actions.
    ui.add_space(4.0);
    ui.painter().vline(ui.cursor().left(), ui.max_rect().y_range().shrink(8.0), Stroke::new(1.0, t.divider));
    ui.add_space(4.0);

    // Search box with enlarged clamp range and keycap shortcut badge.
    let avail = ui.available_width();
    if avail >= 110.0 {
        if widgets::search_box(ui, tl!("Find tools and commands"), avail.clamp(40.0, 280.0)).clicked() {
            app.palette_open = true;
        }
    } else if avail >= 32.0 {
        let tip = crate::commands::command_tip(ui.ctx(), tl!("Find tools and commands ({key})"), "view.palette");
        if icons::button(ui, "search", button_size, false, &tip).clicked() {
            app.palette_open = true;
        }
    }
}

pub fn mode_bar(_app: &mut PagicApp, _ui: &mut egui::Ui) {
    // Menus, Home, tabs, search and actions are now handled cleanly in Bar 1 + Bar 2 (or 1 bar on macOS).
}

/// Draw the right tool rail with page navigation, history, zoom, rotation and page-display controls for the
/// active view.
pub fn right_rail(app: &mut PagicApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let Some((index, id)) = app.active_ids() else { return };
    let Some(doc) = app.session.get(id) else { return };
    let has_signatures = doc.is_signed();
    let has_check = app.a11y.report.as_ref().is_some_and(|(d, _)| *d == id);
    let (has_comments, has_outline, has_fields, has_layers, has_files) = (
        !doc.info.annotations.is_empty(),
        !doc.info.outline.is_empty(),
        !doc.info.fields.is_empty(),
        !doc.info.layers.is_empty(),
        !doc.info.attachments.is_empty(),
    );
    let page_count = doc.info.pages.len();
    let labels: Vec<String> = doc.info.pages.iter().map(|p| p.label.clone()).collect();
    let item = |panel: RightPanel, icon: &str, label: &str, marker: Option<&str>| {
        let item = RailItem::new(panel, icons::source(icon), tl!(label)).marker(marker.is_some());
        match marker {
            Some(marker) => item.marker_label(tl!(marker)),
            None => item,
        }
    };
    let has = |has: bool, label: &'static str| has.then_some(label);
    let mut items = vec![
        item(RightPanel::Comments, "message-square-text", "Comments", has(has_comments, "Has comments")),
        item(RightPanel::Bookmarks, "bookmark", "Bookmarks", has(has_outline, "Has bookmarks")),
        item(RightPanel::Pages, "files", "Page thumbnails", None),
        item(RightPanel::Fields, "text-cursor-input", "Form fields", has(has_fields, "Has form fields")),
        item(RightPanel::Layers, "layers", "Layers", has(has_layers, "Has layers")),
        item(RightPanel::Attachments, "paperclip", "Attachments", has(has_files, "Has attachments")),
        item(RightPanel::Signatures, "signature", "Signatures", has(has_signatures, "Signed")),
    ];
    if has_check {
        items.push(item(RightPanel::Accessibility, "accessibility", "Accessibility Checker", None));
    }
    items.push(item(RightPanel::Tags, "tag", "Tags", None));
    let view = &mut app.views[index];
    let mut page_display = None;
    let rail = Rail::new(egui::Id::unique("rail"))
        .edge(Edge::End)
        .frame(egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin::symmetric(6, 8)).stroke(Stroke::new(1.0, t.divider)))
        .show(ui, &items, app.right.as_ref(), |ui| {
            // Page navigation cluster at the bottom, a panel of its own so that in a short
            // window the buttons above scroll instead of running into it (#3282).
            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                let tip = crate::commands::key_tip(ui.ctx(), tl!("Zoom out ({key})"), crate::commands::ZOOM_OUT);
                if icons::button(ui, "zoom-out", 32.0, false, &tip).clicked() {
                    view.zoom_step(false);
                }
                let tip = crate::commands::key_tip(ui.ctx(), tl!("Zoom in ({key})"), crate::commands::ZOOM_IN);
                if icons::button(ui, "zoom-in", 32.0, false, &tip).clicked() {
                    view.zoom_step(true);
                }
                // Between rotate and zoom, as in Acrobat. Its command runs once `view` is free.
                let tip = crate::i18n::fmt(tl!("Page display: {layout}"), &[("layout", tl!(view.layout.label()))]);
                let resp = icons::button(ui, view.layout.icon(), 32.0, false, &tip);
                page_display = egui::Popup::menu(&resp)
                    .show(|ui| {
                        ui.set_min_width(220.0);
                        rail_view_menu(ui, view)
                    })
                    .and_then(|r| r.inner);
                let tip = crate::commands::key_tip(ui.ctx(), tl!("Rotate view clockwise ({key})"), crate::commands::ROTATE_CW);
                if icons::button(ui, "rotate-cw", 32.0, false, &tip).clicked() {
                    view.rotate_view(true);
                }
                // Not `columns-2` while fitting the page: that is the two-page view's icon.
                let fit_icon = if view.fit == Fit::Width { "maximize" } else { "maximize-2" };
                if icons::button(ui, fit_icon, 32.0, false, tl!("Toggle fit page / fit width")).clicked() {
                    view.fit = if view.fit == Fit::Width { Fit::Page } else { Fit::Width };
                    view.goto = Some((view.current, 0.0));
                }
                let id = egui::Id::unique("zoom-input");
                let draft = id.with("draft");
                let mut typed = ui.data(|d| d.get_temp::<String>(draft)).unwrap_or_else(|| format!("{:.0}%", view.zoom * 100.0));
                let edit = egui::TextEdit::singleline(&mut typed)
                    .id(id)
                    .desired_width(40.0)
                    .align(egui::Align2::CENTER_TOP)
                    .font(theme::medium(11.0))
                    .text_color(t.text_muted)
                    .frame(egui::Frame::NONE)
                    .margin(vec2(2.0, 2.0));
                let r = ui.add(edit).on_hover_text(tl!("Zoom — type a percentage and press Enter"));
                ui.ctx().accesskit_node_builder(id, |node| node.set_label(tl!("Zoom")));
                if r.gained_focus() {
                    let mut state = egui::TextEdit::load_state(ui.ctx(), id).unwrap_or_default();
                    let all = egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(typed.chars().count()));
                    state.cursor.set_char_range(Some(all));
                    state.store(ui.ctx(), id);
                }
                if r.has_focus() {
                    ui.data_mut(|d| d.insert_temp(draft, typed.clone()));
                } else {
                    ui.data_mut(|d| d.remove::<String>(draft));
                    if r.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && let Some(zoom) = DocView::parse_zoom_percent(&typed)
                    {
                        view.set_zoom(zoom);
                    }
                }
                ui.add_space(6.0);
                if icons::button(ui, "chevron-down", 30.0, false, tl!("Next page")).clicked() {
                    view.step_page(true);
                }
                if icons::button(ui, "chevron-up", 30.0, false, tl!("Previous page")).clicked() {
                    view.step_page(false);
                }
                // History is distinct from stepping to the adjacent page, and belongs to this tab.
                for (forward, icon, label, shortcut, enabled) in [
                    (true, "chevron-right", tl!("Next view"), crate::commands::NEXT_VIEW, !view.forward.is_empty()),
                    (false, "chevron-left", tl!("Previous view"), crate::commands::PREV_VIEW, !view.back.is_empty()),
                ] {
                    let tip = format!("{label} ({})", crate::commands::shortcut_label(ui.ctx(), shortcut));
                    if ui.add_enabled_ui(enabled, |ui| icons::button(ui, icon, 30.0, false, &tip)).inner.clicked() {
                        view.view_history(forward);
                    }
                }
                let font = theme::regular(11.0);
                let mut position = page_position(view.current, page_count, labels.get(view.current).map_or("", String::as_str));
                if widgets::text_width(ui, &position, &font) > ui.available_width() {
                    position = page_count.to_string();
                }
                // Out of range (an empty document), like `page_position`: "Page 0 of 0", never past the end.
                let shown = if view.current < page_count { view.current + 1 } else { page_count };
                let of = crate::i18n::fmt(tl!("Page {p} of {n}"), &[("p", &shown.to_string()), ("n", &page_count.to_string())]);
                ui.label(egui::RichText::new(position).font(font).color(t.text_muted)).on_hover_text(of);
                let page_input_id = view.page_input_id();
                // Consume replacement selection once, including when the editor has lost focus.
                let reselect = std::mem::take(&mut view.page_input_reselect);
                // Navigation updates the idle display, never the text being edited.
                if !ui.memory(|m| m.has_focus(page_input_id)) {
                    view.page_input = view.current_page_label(labels.get(view.current).map(String::as_str));
                } else if reselect {
                    // The replacement changed the label while egui retained this editor's cursor.
                    let mut state = egui::TextEdit::load_state(ui.ctx(), page_input_id).unwrap_or_default();
                    let end = egui::text::CCursor::new(view.page_input.chars().count());
                    state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(0), end)));
                    state.store(ui.ctx(), page_input_id);
                }
                let edit = egui::TextEdit::singleline(&mut view.page_input)
                    .id(page_input_id)
                    .desired_width(34.0)
                    .align(egui::Align2::CENTER_TOP)
                    .font(theme::medium(12.0))
                    .margin(vec2(2.0, 4.0));
                let r = egui::Frame::NONE
                    .fill(t.field)
                    .stroke(Stroke::new(1.0, t.border))
                    .corner_radius(CornerRadius::same(5))
                    .show(ui, |ui| ui.add(edit))
                    .inner
                    .on_hover_text(tl!("Current page — type a page number or label (such as iv) and press Enter"))
                    .accessible_name(tl!("Current page"));
                if r.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // A page label first (logical page numbers, as Acrobat), then a number.
                        let typed = view.page_input.clone();
                        view.go_to_typed(&typed, &labels);
                    }
                    view.page_input = view.current_page_label(labels.get(view.current).map(String::as_str));
                }
            });
        });
    match rail.request {
        Some(RailRequest::Open(panel)) => app.choose_right_panel(Some(panel)),
        Some(RailRequest::Close(_)) => app.choose_right_panel(None),
        _ => {}
    }
    if let Some(id) = page_display {
        app.execute(id);
    }
}

/// What the rail shows under the page box: the page count, or, when the current page's label is
/// not its physical number (front matter numbered i, ii, …), its position as "3/180" (#2834).
fn page_position(current: usize, count: usize, label: &str) -> String {
    let number = current.saturating_add(1).to_string();
    if label.is_empty() || label == number || current >= count { count.to_string() } else { format!("{number}/{count}") }
}

/// The rail's Page display menu: the layouts and the cover page (also View ▸ Page display).
/// Returns the command a click asks for, for the caller to run.
fn page_display_menu(ui: &mut egui::Ui, view: &DocView) -> Option<&'static str> {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new(tl!("Page display")).color(t.text_faint).small());
    let mut picked = None;
    for l in PageLayout::ORDER {
        if ui.add(widgets::radio(view.layout == l, tl!(l.label()))).clicked() {
            picked = Some(l.command());
        }
    }
    ui.separator();
    // Continuous scrolling is an independent checkbox over the base layout, as in Acrobat (#752).
    let mut continuous = view.continuous();
    if ui.add(widgets::checkbox(&mut continuous, tl!("Continuous scrolling"))).changed() {
        picked = Some("view.continuous");
    }
    let mut gaps = view.show_page_gaps;
    if ui.add(widgets::checkbox(&mut gaps, tl!("Show gaps between pages"))).changed() {
        picked = Some("view.page_gaps");
    }
    let mut cover = view.cover;
    if ui.add_enabled(view.cover_applies(), widgets::checkbox(&mut cover, tl!("Show cover page in two-page view"))).changed() {
        picked = Some("view.layout.cover");
    }
    if picked.is_some() {
        ui.close();
    }
    picked
}

/// The rail's Page display menu: Acrobat's view choices around the page display. Zoom changes
/// apply at once; the command a click asks for is returned for the caller to run.
fn rail_view_menu(ui: &mut egui::Ui, view: &mut DocView) -> Option<&'static str> {
    let mut picked = None;
    for (id, label) in [("view.fit_width_scrolling", "Fit to width scrolling"), ("view.fit_one_page", "Fit one full page")] {
        if ui.button(tl!(label)).clicked() {
            picked = Some(id);
        }
    }
    ui.separator();
    if ui.button(tl!("Actual size")).clicked() {
        view.set_zoom(1.0);
        ui.close();
    }
    if ui.button(tl!("Zoom to page level")).clicked() {
        view.set_fit(Fit::Page);
        ui.close();
    }
    if ui.button(tl!("Fit visible")).clicked() {
        picked = Some("view.fit_visible");
    }
    ui.separator();
    picked = page_display_menu(ui, view).or(picked);
    ui.separator();
    for (id, label) in [("view.read_mode", "Read mode"), ("view.full_screen", "Full screen mode")] {
        if ui.button(tl!(label)).clicked() {
            picked = Some(id);
        }
    }
    if picked.is_some() {
        ui.close();
    }
    picked
}

#[cfg(test)]
mod page_position_tests {
    use super::page_position;

    #[test]
    fn position_shows_only_when_the_label_is_not_the_page_number() {
        assert_eq!(page_position(0, 180, "i"), "1/180");
        assert_eq!(page_position(2, 180, "1"), "3/180");
        assert_eq!(page_position(2, 180, "3"), "180");
        assert_eq!(page_position(2, 180, ""), "180");
        assert_eq!(page_position(0, 0, "i"), "0");
        assert_eq!(page_position(usize::MAX, 3, "x"), "3");
    }
}

#[cfg(test)]
fn tab_cap(natural: &[f32], budget: f32, gap: f32, min: f32) -> Option<f32> {
    let n = natural.len();
    if n == 0 {
        return None;
    }
    let mut left = budget - gap * (n - 1) as f32;
    if natural.iter().sum::<f32>() <= left {
        return None;
    }
    let mut sorted = natural.to_vec();
    sorted.sort_by(f32::total_cmp);
    for (k, w) in sorted.iter().enumerate() {
        let share = left / (n - k) as f32;
        if *w > share {
            return Some(share.max(min));
        }
        left -= w;
    }
    None
}

#[cfg(test)]
fn tabs_overflow(natural: &[f32], budget: f32, gap: f32, min: f32) -> bool {
    let narrowest: f32 = natural.iter().map(|&w| w.min(min)).sum();
    narrowest + gap * natural.len().saturating_sub(1) as f32 > budget
}

#[cfg(test)]
mod tab_widths {
    use super::{MAC_CONTENT_LEFT, MAC_LIGHT, MAC_LIGHT_PITCH, MAC_LIGHTS_LEFT, MAC_TITLE_BAR, tab_cap};

    /// Tabs that fit keep their own widths.
    #[test]
    fn tabs_that_fit_do_not_shrink() {
        assert_eq!(tab_cap(&[200.0, 150.0], 400.0, 4.0, 110.0), None);
        assert_eq!(tab_cap(&[], 0.0, 4.0, 110.0), None);
    }

    /// Too wide: narrow tabs keep their width, the wide ones share the rest equally, and
    /// together they fill the strip exactly.
    #[test]
    fn wide_tabs_share_what_narrow_ones_leave() {
        let natural = [120.0, 300.0, 300.0];
        let cap = tab_cap(&natural, 500.0, 4.0, 110.0).expect("must shrink");
        let used: f32 = natural.iter().map(|&w| w.min(cap)).sum::<f32>() + 2.0 * 4.0;
        assert!((used - 500.0).abs() < 0.01, "{cap} uses {used}");
        assert!((cap - 186.0).abs() < 0.01, "{cap}");
    }

    /// Arrows appear only when the tabs can't fit even at their narrowest.
    #[test]
    fn arrows_only_when_the_narrowest_tabs_overflow() {
        use super::tabs_overflow;
        assert!(!tabs_overflow(&[250.0; 3], 600.0, 4.0, 180.0), "three shrunk tabs fit");
        assert!(tabs_overflow(&[250.0; 4], 600.0, 4.0, 180.0), "four don't, even at 180");
        assert!(!tabs_overflow(&[100.0; 5], 600.0, 4.0, 180.0), "narrow tabs fit as they are");
        assert!(!tabs_overflow(&[], 0.0, 4.0, 180.0));
    }

    /// Tabs never get narrower than the minimum: past it, the strip scrolls instead.
    #[test]
    fn tabs_stop_at_their_minimum() {
        assert_eq!(tab_cap(&[250.0; 12], 600.0, 4.0, 110.0), Some(110.0));
        assert_eq!(tab_cap(&[250.0; 3], 0.0, 4.0, 110.0), Some(110.0));
    }

    /// Three 100-wide tabs, 4 apart, starting at x = 0.
    fn three_slots() -> Vec<(usize, egui::Rect)> {
        (0..3).map(|i| (i, egui::Rect::from_min_size(egui::pos2(i as f32 * 104.0, 0.0), egui::vec2(100.0, 30.0)))).collect()
    }

    /// A dragged tab lands past every other tab whose centre the pointer has crossed.
    #[test]
    fn a_dragged_tab_drops_past_the_centres_it_crossed() {
        use super::drop_index;
        let slots = three_slots();
        // Centres at 50, 154 and 258.
        assert_eq!(drop_index(&slots, 0, 10.0), 0, "still over itself");
        assert_eq!(drop_index(&slots, 0, 160.0), 1, "past the middle tab's centre");
        assert_eq!(drop_index(&slots, 0, 900.0), 2, "past the end");
        assert_eq!(drop_index(&slots, 2, -50.0), 0, "before the start");
        assert_eq!(drop_index(&slots, 2, 100.0), 1, "between the first and middle centres");
        assert_eq!(drop_index(&[], 0, 100.0), 0);
    }

    /// The drop line sits in the gap the tab will land in, and not at all where it already is.
    #[test]
    fn the_drop_line_marks_the_gap_and_hides_when_nothing_moves() {
        use super::drop_marker;
        let slots = three_slots();
        assert_eq!(drop_marker(&slots, 0, 0, 4.0), None);
        assert_eq!(drop_marker(&slots, 1, 1, 4.0), None);
        // The first tab dropped at the end: after the last tab.
        assert_eq!(drop_marker(&slots, 0, 2, 4.0), Some(310.0));
        // The first tab dropped between the middle and the last: in the gap between them.
        assert_eq!(drop_marker(&slots, 0, 1, 4.0), Some(206.0));
        // The last tab dropped at the start: before the first tab.
        assert_eq!(drop_marker(&slots, 2, 0, 4.0), Some(-2.0));
        assert_eq!(drop_marker(&[], 0, 1, 4.0), None);
    }

    #[test]
    fn the_mac_title_bar_centres_and_clears_the_traffic_lights() {
        // The lights are centred in the bar, and content keeps the same gap after them (#789).
        let margin = (MAC_TITLE_BAR - MAC_LIGHT) / 2.0;
        let lights_end = MAC_LIGHTS_LEFT + 2.0 * MAC_LIGHT_PITCH + MAC_LIGHT;
        assert_eq!(MAC_CONTENT_LEFT - lights_end, margin);
        // `egui::Margin` is in `i8`.
        assert!(MAC_CONTENT_LEFT <= f32::from(i8::MAX));
    }
}
