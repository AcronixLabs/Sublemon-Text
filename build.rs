use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let icon_dir = manifest_dir.join("assets/material-icon-theme/png");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("output dir"));
    let mut icons: Vec<_> = fs::read_dir(&icon_dir)
        .expect("bundled Material Icon Theme PNG directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "png"))
        .collect();
    icons.sort();

    let mut generated = String::from("static MATERIAL_ICON_PNGS: &[(&str, &[u8])] = &[\n");
    for path in icons {
        let name = path.file_stem().expect("icon stem").to_string_lossy();
        println!("cargo:rerun-if-changed={}", path.display());
        generated.push_str(&format!(
            "    ({name:?}, include_bytes!({:?})),\n",
            path.to_string_lossy()
        ));
    }
    generated.push_str("];\n");
    fs::write(out_dir.join("material_icon_pngs.rs"), generated).expect("write icon table");
    println!("cargo:rerun-if-changed=assets/material-icon-theme/associations.tsv");
    println!("cargo:rerun-if-changed=assets/sublemon-syntaxes.packdump");
}
