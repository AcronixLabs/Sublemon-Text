use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    Crlf,
    Cr,
}

impl LineEnding {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lf => "LF",
            Self::Crlf => "CRLF",
            Self::Cr => "CR",
        }
    }
}

#[derive(Clone, Debug)]
struct FileStamp {
    modified: Option<SystemTime>,
    len: u64,
    hash: u64,
}

impl FileStamp {
    fn read(path: &Path) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        let modified = fs::metadata(path)
            .ok()
            .and_then(|meta| meta.modified().ok());
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        Ok(Self {
            modified,
            len: bytes.len() as u64,
            hash: hasher.finish(),
        })
    }
}

pub struct Document {
    pub path: Option<PathBuf>,
    pub text: String,
    saved_text: String,
    pub line_ending: LineEnding,
    baseline: Option<FileStamp>,
    pub read_only: bool,
}

impl Document {
    pub fn untitled() -> Self {
        Self {
            path: None,
            text: String::new(),
            saved_text: String::new(),
            line_ending: LineEnding::Lf,
            baseline: None,
            read_only: false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.text != self.saved_text
    }

    pub fn discard_changes(&mut self) {
        self.text.clone_from(&self.saved_text);
    }

    pub fn title(&self, number: usize) -> String {
        self.path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                if number == 1 {
                    "untitled".into()
                } else {
                    format!("untitled {number}")
                }
            })
    }

    pub fn open(path: &Path) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        let source = String::from_utf8(bytes).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("file is not valid UTF-8: {error}"),
            )
        })?;
        let line_ending = detect_line_ending(&source);
        let text = source.replace("\r\n", "\n").replace('\r', "\n");
        let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let baseline = Some(FileStamp::read(&canonical)?);
        Ok(Self {
            path: Some(canonical),
            saved_text: text.clone(),
            text,
            line_ending,
            baseline,
            read_only: fs::metadata(path)
                .map(|meta| meta.permissions().readonly())
                .unwrap_or(false),
        })
    }

    pub fn save(&mut self) -> io::Result<()> {
        let path = self.path.clone().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "untitled document needs Save As",
            )
        })?;
        self.save_to(&path, false)
    }

    pub fn save_as(&mut self, path: &Path) -> io::Result<()> {
        self.save_to(path, true)
    }

    fn save_to(&mut self, path: &Path, save_as: bool) -> io::Result<()> {
        if self.read_only && self.path.as_deref() == Some(path) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "file is read-only",
            ));
        }
        if save_as && self.path.as_deref() != Some(path) && path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Save As will not overwrite an existing file",
            ));
        }
        if !save_as {
            let current = FileStamp::read(path)?;
            if self
                .baseline
                .as_ref()
                .is_some_and(|baseline| !same_stamp(baseline, &current))
            {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "file changed on disk since it was opened; save cancelled",
                ));
            }
        }
        let serialized = match self.line_ending {
            LineEnding::Lf => self.text.clone(),
            LineEnding::Crlf => self.text.replace('\n', "\r\n"),
            LineEnding::Cr => self.text.replace('\n', "\r"),
        };
        fs::write(path, serialized.as_bytes())?;
        let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        self.path = Some(canonical.clone());
        self.baseline = Some(FileStamp::read(&canonical)?);
        self.saved_text.clone_from(&self.text);
        self.read_only = false;
        Ok(())
    }

    pub fn revert(&mut self) -> io::Result<()> {
        let path = self.path.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "untitled document cannot be reverted",
            )
        })?;
        let replacement = Self::open(path)?;
        *self = replacement;
        Ok(())
    }
}

fn detect_line_ending(text: &str) -> LineEnding {
    if text.contains("\r\n") {
        LineEnding::Crlf
    } else if text.contains('\r') {
        LineEnding::Cr
    } else {
        LineEnding::Lf
    }
}

fn same_stamp(left: &FileStamp, right: &FileStamp) -> bool {
    left.len == right.len && left.hash == right.hash && left.modified == right.modified
}
