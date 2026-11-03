//! Existing notices embedded verbatim for offline, selectable license access.

pub(super) const NOTICES: &[(&str, &str)] = &[
    ("Pagic / Recto — MIT", include_str!("../../../../LICENSE-MIT")),
    ("Apache License 2.0", include_str!("../../../../LICENSE-APACHE")),
    ("egui — MIT", include_str!("../../../../licenses/egui-MIT.txt")),
    ("hayro — MIT", include_str!("../../../../vendor/hayro/LICENSE-MIT")),
    ("Lucide — ISC", include_str!("../../../../assets/icons/LICENSE-lucide.txt")),
    ("Historical Recto artwork notice", include_str!("../../../../assets/app-icon/LICENSE-recto.txt")),
    ("Historical Recto trademark guidance", include_str!("../../../../docs/brand/LICENSE-brand.txt")),
    ("Inter — OFL", include_str!("../../../../assets/fonts/OFL-Inter.txt")),
    ("JetBrains Mono — OFL", include_str!("../../../../assets/fonts/OFL-JetBrainsMono.txt")),
    ("Dancing Script — OFL", include_str!("../../../../assets/fonts/OFL-DancingScript.txt")),
    ("biz-ud-mincho — OFL", include_str!("../../../../crates/fonts/assets/fonts/biz-ud-mincho/OFL.txt")),
    ("biz-ud-pgothic — OFL", include_str!("../../../../crates/fonts/assets/fonts/biz-ud-pgothic/OFL.txt")),
    ("lxgw-wenkai-lite — OFL", include_str!("../../../../crates/fonts/assets/fonts/lxgw-wenkai-lite/OFL.txt")),
    ("nanum-gothic — OFL", include_str!("../../../../crates/fonts/assets/fonts/nanum-gothic/OFL.txt")),
    ("noto-sans-arabic — OFL", include_str!("../../../../crates/fonts/assets/fonts/noto-sans-arabic/OFL.txt")),
    ("noto-sans-cjk-sc — OFL", include_str!("../../../../crates/fonts/assets/fonts/noto-sans-cjk-sc/OFL.txt")),
    ("noto-sans-devanagari — OFL", include_str!("../../../../crates/fonts/assets/fonts/noto-sans-devanagari/OFL.txt")),
    ("noto-sans-georgian — OFL", include_str!("../../../../crates/fonts/assets/fonts/noto-sans-georgian/OFL.txt")),
    ("noto-sans-hebrew — OFL", include_str!("../../../../crates/fonts/assets/fonts/noto-sans-hebrew/OFL.txt")),
    ("noto-sans-symbols — OFL", include_str!("../../../../crates/fonts/assets/fonts/noto-sans-symbols/OFL.txt")),
    ("noto-sans-symbols-2 — OFL", include_str!("../../../../crates/fonts/assets/fonts/noto-sans-symbols-2/OFL.txt")),
    ("noto-sans-thai — OFL", include_str!("../../../../crates/fonts/assets/fonts/noto-sans-thai/OFL.txt")),
    ("shippori-mincho — OFL", include_str!("../../../../crates/fonts/assets/fonts/shippori-mincho/OFL.txt")),
];

pub(super) fn show(ui: &mut egui::Ui) {
    let id = ui.make_persistent_id("about-license-selection");
    let mut selected = ui.data_mut(|data| data.get_temp::<usize>(id)).unwrap_or(0).min(NOTICES.len() - 1);
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(220.0);
            ui.label(crate::i18n::t("License notice"));
            egui::ScrollArea::vertical().id_salt("about-license-list").max_height(300.0).auto_shrink([false, false]).show(ui, |ui| {
                for (index, &(name, _)) in NOTICES.iter().enumerate() {
                    ui.selectable_value(&mut selected, index, name);
                }
            });
        });
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical().id_salt(("about-license-text", selected)).max_height(300.0).auto_shrink([false, false]).show(ui, |ui| {
                ui.add(egui::Label::new(NOTICES[selected].1).selectable(true).wrap());
            });
        });
    });
    ui.data_mut(|data| data.insert_temp(id, selected));
}
