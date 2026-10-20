#![forbid(unsafe_code)]

#[path = "../lsp_server.rs"]
mod lsp_server;

fn main() {
    if let Err(error) = lsp_server::run_stdio() {
        eprintln!("Sublemon language service failed: {error}");
        std::process::exit(1);
    }
}
