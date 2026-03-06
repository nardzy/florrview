
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

mod florr;
use eframe::NativeOptions;
use egui::FontFamily;
use florr::core::Florr;
// ptr 300

fn main() -> eframe::Result<()> {
    let icon_data = eframe::icon_data::from_png_bytes(include_bytes!("../data/favicon.png"))
        .expect("invalid icon data");
    let mut options = NativeOptions::default();
    options.viewport.icon = Some(Arc::new(icon_data));
    eframe::run_native(
        "FlorrView",
        options,
        Box::new(|ctx| {
            let mut fonts = egui::FontDefinitions::default();

            fonts.font_data.insert(
                "noto_sans_jp".to_owned(),
                egui::FontData::from_static(include_bytes!("../data/font/NotoSansJP-Regular.ttf"))
                    .into(),
            );

            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "noto_sans_jp".to_owned());

            ctx.egui_ctx.set_fonts(fonts);
            Ok(Box::new(Florr::new()))
        }),
    )
}