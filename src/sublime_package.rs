use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use zip::ZipArchive;

const MAX_ENTRIES: usize = 10_000;
const MAX_ENTRY_SIZE: u64 = 64 * 1024 * 1024;
const MAX_PACKAGE_SIZE: u64 = 256 * 1024 * 1024;

/// Extracts a `.sublime-package` ZIP archive into a fresh destination directory.
/// The archive is treated as data only: package code is never loaded or executed.
pub fn extract_package(archive_path: &Path, packages_dir: &Path) -> io::Result<PathBuf> {
    if !archive_path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("sublime-package"))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a .sublime-package file",
        ));
    }

    let package_name = archive_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|name| {
            !name.is_empty()
                && *name != "."
                && *name != ".."
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid package filename"))?;

    fs::create_dir_all(packages_dir)?;
    let destination = packages_dir.join(package_name);
    if destination.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("package folder already exists: {}", destination.display()),
        ));
    }

    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file).map_err(invalid_archive)?;
    if archive.len() > MAX_ENTRIES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("package contains more than {MAX_ENTRIES} entries"),
        ));
    }

    let mut unpacked_total = 0_u64;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(invalid_archive)?;
        if entry.size() > MAX_ENTRY_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "package entry '{}' exceeds the 64 MiB limit",
                    entry_label(&entry)
                ),
            ));
        }
        unpacked_total = unpacked_total.saturating_add(entry.size());
        if unpacked_total > MAX_PACKAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "package exceeds the 256 MiB unpacked size limit",
            ));
        }
        validated_entry_path(&entry)?;
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("package entry '{}' is a symbolic link", entry_label(&entry)),
            ));
        }
    }

    let stage = unique_stage_path(packages_dir, package_name);
    fs::create_dir(&stage)?;
    let mut stage_guard = StagingDirectory(Some(stage.clone()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(invalid_archive)?;
        let relative_path = validated_entry_path(&entry)?;
        let output_path = stage.join(relative_path);
        if entry.is_dir() {
            fs::create_dir_all(&output_path)?;
            continue;
        }
        let parent = output_path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "archive entry has no parent directory",
            )
        })?;
        fs::create_dir_all(parent)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output_path)?;
        let copied = io::copy(&mut entry.by_ref().take(MAX_ENTRY_SIZE + 1), &mut output)?;
        if copied != entry.size() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "package entry '{}' had an unexpected size",
                    entry_label(&entry)
                ),
            ));
        }
    }

    fs::rename(&stage, &destination)?;
    stage_guard.0 = None;
    Ok(destination)
}

fn validated_entry_path<R: Read>(entry: &zip::read::ZipFile<'_, R>) -> io::Result<PathBuf> {
    let relative_path = entry.enclosed_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsafe package path: {}", entry_label(entry)),
        )
    })?;
    if relative_path.as_os_str().is_empty()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsafe package path: {}", entry_label(entry)),
        ));
    }
    Ok(relative_path)
}

fn entry_label<R: Read>(entry: &zip::read::ZipFile<'_, R>) -> String {
    entry
        .name()
        .map(|name| name.into_owned())
        .unwrap_or_else(|_| "<invalid name>".to_owned())
}

fn unique_stage_path(packages_dir: &Path, package_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    packages_dir.join(format!(
        ".{package_name}.installing-{}-{timestamp}",
        std::process::id()
    ))
}

fn invalid_archive(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("invalid ZIP package: {error}"),
    )
}

struct StagingDirectory(Option<PathBuf>);

impl Drop for StagingDirectory {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let _ = fs::remove_dir_all(path);
        }
    }
}
