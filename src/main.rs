#![forbid(unsafe_code)]

mod app;
mod document;
mod lsp;
mod sublime_package;
mod syntax;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let icon = load_icon();
    let viewport = egui::ViewportBuilder::default()
        .with_title("Sublemon")
        .with_inner_size([627.0, 692.0])
        .with_min_inner_size([480.0, 360.0])
        .with_icon(icon);

    let options = eframe::NativeOptions {
        viewport,
        // Render egui's UI through wgpu so supported systems can use hardware acceleration.
        renderer: eframe::Renderer::Wgpu,
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
