#![forbid(unsafe_code)]

mod app;
mod document;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let icon = load_icon();
    let viewport = egui::ViewportBuilder::default()
        .with_title("Sublemon")
        .with_inner_size([1120.0, 760.0])
        .with_min_inner_size([720.0, 480.0])
        .with_icon(icon);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Sublemon",
        options,
        Box::new(|_creation_context| Ok(Box::new(app::SublemonApp::default()))),
    )
}

fn load_icon() -> egui::IconData {
    let image = image::load_from_memory(include_bytes!("../icon.png"))
        .expect("the bundled Sublemon icon must be a valid PNG")
        .into_rgba8();
    let (width, height) = image.dimensions();

    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}
