use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

pub struct LspClient {
    child: Child,
    input: BufWriter<ChildStdin>,
    incoming: Receiver<Result<Value, String>>,
    next_id: i64,
    ready: bool,
    versions: HashMap<String, i32>,
    workspace_roots: Vec<std::path::PathBuf>,
}

impl LspClient {
    pub fn start(command: &str, root: &Path) -> io::Result<Self> {
        let (program, args) = parse_command(command)?;
        let mut child = Command::new(program)
            .args(args)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = BufWriter::new(child.stdin.take().expect("LSP stdin is piped"));
        let output = child.stdout.take().expect("LSP stdout is piped");
        let (sender, incoming) = mpsc::channel();
        thread::Builder::new()
            .name("sublemon-lsp-reader".into())
            .spawn(move || read_messages(output, sender))?;

        let root_uri = path_to_uri(root);
        let mut client = Self {
            child,
            input,
            incoming,
            next_id: 1,
            ready: false,
            versions: HashMap::new(),
            workspace_roots: vec![root.to_path_buf()],
        };
        client.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": root_uri,
                "capabilities": {
                    "textDocument": {
                        "synchronization": { "dynamicRegistration": false, "willSave": false, "didSave": false },
                        "publishDiagnostics": { "relatedInformation": true },
                        "completion": { "completionItem": { "snippetSupport": false } }
                    },
                    "workspace": { "workspaceFolders": { "supported": true, "changeNotifications": true } }
                },
                "workspaceFolders": [{ "uri": root_uri, "name": root.file_name().unwrap_or_default().to_string_lossy() }]
            }),
        )?;
        Ok(client)
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn poll(&mut self) -> Vec<Value> {
        let mut messages = Vec::new();
        loop {
            match self.incoming.try_recv() {
                Ok(Ok(message)) => {
                    if message.get("id") == Some(&json!(1))
                        && message.get("method").is_none()
                        && message.get("error").is_none()
                    {
                        if self.notify("initialized", json!({})).is_ok() {
                            self.ready = true;
                        }
                    }
                    if message.get("method").is_some()
                        && let Some(id) = message.get("id").cloned()
                    {
                        self.respond_to_server_request(&message, id);
                    }
                    messages.push(message);
                }
                Ok(Err(error)) => {
                    messages.push(json!({ "method": "sublemon/error", "message": error }))
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        if self.child.try_wait().ok().flatten().is_some() {
            messages.push(json!({ "method": "sublemon/exit" }));
        }
        messages
    }

    pub fn did_open(&mut self, path: &Path, text: &str) -> io::Result<()> {
        if !self.ready {
            return Ok(());
        }
        let uri = path_to_uri(path);
        self.versions.insert(uri.clone(), 1);
        self.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": language_id(path),
                    "version": 1,
                    "text": text
                }
            }),
        )
    }

    pub fn did_change(&mut self, path: &Path, text: &str) -> io::Result<()> {
        if !self.ready {
            return Ok(());
        }
        let uri = path_to_uri(path);
        let version = {
            let version = self.versions.entry(uri.clone()).or_insert(0);
            *version += 1;
            *version
        };
        self.notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": version },
                "contentChanges": [{ "text": text }]
            }),
        )
    }

    pub fn request_at(
        &mut self,
        method: &str,
        path: &Path,
        text: &str,
        character_offset: usize,
        extra: Value,
    ) -> io::Result<i64> {
        let (line, character) = position_from_offset(text, character_offset);
        let mut params = json!({
            "textDocument": { "uri": path_to_uri(path) },
            "position": { "line": line, "character": character }
        });
        if let (Some(params), Some(extra)) = (params.as_object_mut(), extra.as_object()) {
            params.extend(extra.clone());
        }
        self.request(method, params)
    }

    pub fn did_close(&mut self, path: &Path) -> io::Result<()> {
        if !self.ready {
            return Ok(());
        }
        let uri = path_to_uri(path);
        self.versions.remove(&uri);
        self.notify(
            "textDocument/didClose",
            json!({ "textDocument": { "uri": uri } }),
        )
    }

    pub fn sync_workspace_folders(&mut self, paths: &[&Path]) -> io::Result<()> {
        if !self.ready {
            return Ok(());
        }
        let next: Vec<_> = paths.iter().map(|path| path.to_path_buf()).collect();
        let added: Vec<_> = next
            .iter()
            .filter(|path| !self.workspace_roots.contains(path))
            .map(|path| {
                json!({
                    "uri": path_to_uri(path),
                    "name": path.file_name().unwrap_or_default().to_string_lossy()
                })
            })
            .collect();
        let removed: Vec<_> = self
            .workspace_roots
            .iter()
            .filter(|path| !next.contains(path))
            .map(|path| {
                json!({
                    "uri": path_to_uri(path),
                    "name": path.file_name().unwrap_or_default().to_string_lossy()
                })
            })
            .collect();
        self.notify(
            "workspace/didChangeWorkspaceFolders",
            json!({ "event": { "added": added, "removed": removed } }),
        )?;
        self.workspace_roots = next;
        Ok(())
    }

    pub fn stop(self) {
        drop(self);
    }

    fn shutdown(&mut self) {
        if self.ready {
            let _ = self.request("shutdown", Value::Null);
            let _ = self.notify("exit", Value::Null);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    fn request(&mut self, method: &str, params: Value) -> io::Result<i64> {
        let id = self.next_id;
        self.next_id += 1;
        write_message(
            &mut self.input,
            &json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
        )?;
        Ok(id)
    }

    fn notify(&mut self, method: &str, params: Value) -> io::Result<()> {
        write_message(
            &mut self.input,
            &json!({ "jsonrpc": "2.0", "method": method, "params": params }),
        )
    }

    fn respond_to_server_request(&mut self, message: &Value, id: Value) {
        let method = message["method"].as_str().unwrap_or_default();
        let result = match method {
            "workspace/configuration" => {
                let count = message["params"]["items"].as_array().map_or(0, Vec::len);
                Value::Array(vec![Value::Null; count])
            }
            "workspace/workspaceFolders" => Value::Array(
                self.workspace_roots
                    .iter()
                    .map(|path| json!({ "uri": path_to_uri(path), "name": path.file_name().unwrap_or_default().to_string_lossy() }))
                    .collect(),
            ),
            "workspace/applyEdit" => json!({ "applied": false }),
            "client/registerCapability"
            | "client/unregisterCapability"
            | "window/workDoneProgress/create"
            | "workspace/semanticTokens/refresh"
            | "workspace/codeLens/refresh"
            | "workspace/inlayHint/refresh" => Value::Null,
            _ => {
                let _ = write_message(
                    &mut self.input,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32601, "message": format!("unsupported client request: {method}") }
                    }),
                );
                return;
            }
        };
        let _ = write_message(
            &mut self.input,
            &json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        );
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn parse_command(command: &str) -> io::Result<(String, Vec<String>)> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut characters = command.trim().chars().peekable();
    while let Some(character) = characters.next() {
        if escaped {
            word.push(character);
            escaped = false;
        } else if character == '\\' && quote != Some('\'') {
            if characters
                .peek()
                .is_some_and(|next| next.is_whitespace() || matches!(next, '\\' | '\'' | '"'))
            {
                escaped = true;
            } else {
                word.push(character);
            }
        } else if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            } else {
                word.push(character);
            }
        } else if character.is_whitespace() && quote.is_none() {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
        } else {
            word.push(character);
        }
    }
    if escaped || quote.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unclosed quote or escape in server command",
        ));
    }
    if !word.is_empty() {
        words.push(word);
    }
    if words.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "enter a language server command",
        ));
    }
    let program = words.remove(0);
    Ok((program, words))
}

fn write_message(writer: &mut impl Write, value: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

fn read_messages(output: impl Read, sender: mpsc::Sender<Result<Value, String>>) {
    let mut reader = BufReader::new(output);
    loop {
        let mut content_length = None;
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => return,
                Ok(_) if line == "\r\n" || line == "\n" => break,
                Ok(_) => {
                    if let Some((header, value)) = line.trim().split_once(':') {
                        if header.eq_ignore_ascii_case("content-length") {
                            content_length = value.trim().parse::<usize>().ok();
                        }
                    }
                }
                Err(error) => {
                    let _ = sender.send(Err(error.to_string()));
                    return;
                }
            }
        }
        let Some(length) = content_length else {
            let _ = sender.send(Err("LSP message has no Content-Length header".into()));
            return;
        };
        let mut body = vec![0; length];
        if let Err(error) = reader.read_exact(&mut body) {
            let _ = sender.send(Err(error.to_string()));
            return;
        }
        match serde_json::from_slice(&body) {
            Ok(message) => {
                if sender.send(Ok(message)).is_err() {
                    return;
                }
            }
            Err(error) => {
                let _ = sender.send(Err(error.to_string()));
                return;
            }
        }
    }
}

fn path_to_uri(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    let mut encoded = String::new();
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b':' | b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    if encoded.starts_with('/') {
        format!("file://{encoded}")
    } else {
        format!("file:///{encoded}")
    }
}

pub fn uri_to_path(uri: &str) -> Option<std::path::PathBuf> {
    let encoded = uri.strip_prefix("file://")?;
    let mut decoded = Vec::with_capacity(encoded.len());
    let bytes = encoded.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    let decoded = String::from_utf8(decoded).ok()?;
    #[cfg(windows)]
    let decoded = decoded
        .strip_prefix('/')
        .unwrap_or(&decoded)
        .replace('/', "\\");
    Some(std::path::PathBuf::from(decoded))
}

fn position_from_offset(text: &str, character_offset: usize) -> (u32, u32) {
    let prefix: String = text.chars().take(character_offset).collect();
    let line = prefix.matches('\n').count() as u32;
    let current_line = prefix.rsplit('\n').next().unwrap_or_default();
    (line, current_line.encode_utf16().count() as u32)
}

pub fn offset_from_position(text: &str, line: u32, character: u32) -> usize {
    let mut offset = 0usize;
    for (index, text_line) in text.split('\n').enumerate() {
        if index == line as usize {
            let mut utf16_offset = 0;
            for (char_offset, ch) in text_line.chars().enumerate() {
                let units = ch.len_utf16() as u32;
                if utf16_offset + units > character {
                    return offset + char_offset;
                }
                utf16_offset += units;
            }
            return offset + text_line.chars().count();
        }
        offset += text_line.chars().count() + 1;
    }
    text.chars().count()
}

fn language_id(path: &Path) -> &str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "rs" => "rust",
        "py" | "pyi" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "javascriptreact",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" => "typescriptreact",
        "json" => "json",
        "jsonc" => "jsonc",
        "md" | "markdown" => "markdown",
        "html" | "htm" => "html",
        "css" => "css",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cc" | "cpp" | "cxx" | "hpp" => "cpp",
        "cs" => "csharp",
        "lua" => "lua",
        "rb" => "ruby",
        "php" => "php",
        "sh" | "bash" => "shellscript",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        _ => "plaintext",
    }
}
