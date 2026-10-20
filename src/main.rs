#![forbid(unsafe_code)]

mod app;
mod app_state;
mod document;
mod editor_metrics;
mod lsp;
mod lsp_server;
mod syntax;

use eframe::egui;

fn main() -> eframe::Result<()> {
    if std::env::args().nth(1).as_deref() == Some("--lsp-server") {
        return match lsp_server::run_stdio() {
            Ok(()) => Ok(()),
            Err(error) => {
                eprintln!("Sublemon language service failed: {error}");
                Err(eframe::Error::AppCreation(Box::new(error)))
            }
        };
    }
    let image = image::load_from_memory(include_bytes!("../icon.png"))
        .expect("the bundled Sublemon icon must be a valid PNG")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let icon = egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    };
    let viewport = egui::ViewportBuilder::default()
        .with_title("Sublemon")
        .with_inner_size([627.0, 692.0])
        .with_min_inner_size([480.0, 360.0])
        .with_icon(icon);
    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Sublemon",
        options,
        Box::new(|_creation_context| Ok(Box::new(app::SublemonApp::default()))),
    )
}
