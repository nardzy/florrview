#![windows_subsystem = "windows"]

mod florr;

use eframe::NativeOptions;
use egui::{FontFamily, ViewportBuilder};
use florr::core::Florr;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_app_id("florrview")
            .with_icon(eframe::icon_data::from_png_bytes(include_bytes!("../data/florrview.png")).expect("whaa?")),
        ..Default::default()
    };
    let version = env!("CARGO_PKG_VERSION");
    let app_name = format!("FlorrView v{version}");
    eframe::run_native(
        &app_name,
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