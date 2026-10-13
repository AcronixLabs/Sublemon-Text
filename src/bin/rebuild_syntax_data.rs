use std::{
    fs,
    path::{Path, PathBuf},
};

fn add_syntax_files(
    builder: &mut syntect::parsing::SyntaxSetBuilder,
    folder: &Path,
    loaded: &mut usize,
    skipped: &mut usize,
) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            add_syntax_files(builder, &path, loaded, skipped);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "sublime-syntax")
        {
            let parsed = fs::read_to_string(&path)
                .map_err(|error| error.to_string())
                .and_then(|source| {
                    let fallback = path.file_stem().and_then(|stem| stem.to_str());
                    syntect::parsing::SyntaxDefinition::load_from_str(&source, true, fallback)
                        .map_err(|error| error.to_string())
                });
            match parsed {
                Ok(syntax) => {
                    builder.add(syntax);
                    *loaded += 1;
                }
                Err(_) => *skipped += 1,
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let package_dir = root.join("assets/sublime-syntax-packages");
    let mut builder = syntect::parsing::SyntaxSet::load_defaults_newlines().into_builder();
    let mut loaded = 0;
    let mut skipped = 0;
    add_syntax_files(&mut builder, &package_dir, &mut loaded, &mut skipped);

    let dump = syntect::dumps::dump_binary(&builder.build());
    fs::write(root.join("assets/sublemon-syntaxes.packdump"), dump)?;
    println!(
        "Compiled {loaded} upstream syntax definitions; {skipped} need syntax features the current parser does not support."
    );
    Ok(())
}
