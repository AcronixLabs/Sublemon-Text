use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, BufWriter, Write},
};

const MAX_ANALYSIS_BYTES: usize = 512 * 1024;
const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Default)]
struct Server {
    documents: HashMap<String, String>,
}

pub fn run_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());
    let mut server = Server::default();

    while let Some(message) = read_message(&mut reader)? {
        let method = message["method"].as_str().unwrap_or_default();
        let id = message.get("id").cloned();
        match method {
            "exit" => break,
            "initialize" => {
                if let Some(id) = id {
                    write_message(
                        &mut writer,
                        &json!({
                            "jsonrpc":"2.0", "id":id,
                            "result": {"capabilities": {
                                "textDocumentSync": 1,
                                "completionProvider": {"resolveProvider": false, "triggerCharacters": [".", "::"]},
                                "hoverProvider": true,
                                "definitionProvider": true,
                                "declarationProvider": true,
                                "typeDefinitionProvider": true,
                                "implementationProvider": true,
                                "referencesProvider": true,
                                "documentSymbolProvider": true
                            }, "serverInfo": {"name":"Sublemon Language Service", "version":"0.1"}}
                        }),
                    )?;
                }
            }
            "initialized" | "$/cancelRequest" | "$/setTrace" => {}
            "shutdown" => {
                if let Some(id) = id {
                    write_message(
                        &mut writer,
                        &json!({"jsonrpc":"2.0", "id":id, "result":null}),
                    )?;
                }
            }
            "textDocument/didOpen" => {
                let uri = message["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or_default();
                let text = message["params"]["textDocument"]["text"]
                    .as_str()
                    .unwrap_or_default();
                server.documents.insert(uri.to_owned(), text.to_owned());
                server.publish_diagnostics(&mut writer, uri)?;
            }
            "textDocument/didChange" => {
                let uri = message["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or_default();
                let text = message["params"]["contentChanges"]
                    .as_array()
                    .and_then(|changes| changes.last())
                    .and_then(|change| change["text"].as_str())
                    .unwrap_or_default();
                server.documents.insert(uri.to_owned(), text.to_owned());
                server.publish_diagnostics(&mut writer, uri)?;
            }
            "textDocument/didClose" => {
                let uri = message["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or_default();
                server.documents.remove(uri);
                write_message(
                    &mut writer,
                    &json!({"jsonrpc":"2.0", "method":"textDocument/publishDiagnostics", "params":{"uri":uri,"diagnostics":[]}}),
                )?;
            }
            "workspace/didChangeWorkspaceFolders" => {}
            "textDocument/completion" => {
                if let Some(id) = id {
                    write_message(
                        &mut writer,
                        &json!({"jsonrpc":"2.0", "id":id, "result":server.completion(&message["params"])}),
                    )?;
                }
            }
            "textDocument/hover" => {
                if let Some(id) = id {
                    write_message(
                        &mut writer,
                        &json!({"jsonrpc":"2.0", "id":id, "result":server.hover(&message["params"])}),
                    )?;
                }
            }
            "textDocument/definition"
            | "textDocument/declaration"
            | "textDocument/typeDefinition"
            | "textDocument/implementation" => {
                if let Some(id) = id {
                    write_message(
                        &mut writer,
                        &json!({"jsonrpc":"2.0", "id":id, "result":server.definition(&message["params"])}),
                    )?;
                }
            }
            "textDocument/references" => {
                if let Some(id) = id {
                    write_message(
                        &mut writer,
                        &json!({"jsonrpc":"2.0", "id":id, "result":server.references(&message["params"])}),
                    )?;
                }
            }
            "textDocument/documentSymbol" => {
                if let Some(id) = id {
                    write_message(
                        &mut writer,
                        &json!({"jsonrpc":"2.0", "id":id, "result":server.document_symbols(&message["params"])}),
                    )?;
                }
            }
            _ => {
                if let Some(id) = id {
                    write_message(
                        &mut writer,
                        &json!({
                            "jsonrpc":"2.0", "id":id,
                            "error":{"code":-32601,"message":format!("method not supported: {method}")}
                        }),
                    )?;
                }
            }
        }
    }
    Ok(())
}

impl Server {
    fn document_text<'a>(&'a self, params: &Value) -> Option<(String, &'a str)> {
        let uri = params["textDocument"]["uri"].as_str()?.to_owned();
        Some((uri.clone(), self.documents.get(&uri)?.as_str()))
    }

    fn completion(&self, params: &Value) -> Value {
        let Some((_, text)) = self.document_text(params) else {
            return json!([]);
        };
        if text.len() > MAX_ANALYSIS_BYTES {
            return json!([]);
        }
        let position = &params["position"];
        let offset = offset_at(
            text,
            position["line"].as_u64().unwrap_or(0) as usize,
            position["character"].as_u64().unwrap_or(0) as usize,
        );
        let prefix = word_at(text, offset);
        let mut candidates = std::collections::BTreeSet::new();
        for token in text.split(|ch: char| !(ch.is_alphanumeric() || ch == '_')) {
            if token.len() >= 2 && token.starts_with(prefix) && token != prefix {
                candidates.insert(token.to_owned());
            }
        }
        for keyword in [
            "async", "await", "break", "const", "else", "enum", "false", "fn", "for", "if", "impl",
            "let", "loop", "match", "mut", "pub", "return", "self", "struct", "trait", "true",
            "type", "use", "where", "while",
        ] {
            if keyword.starts_with(prefix) && keyword != prefix {
                candidates.insert(keyword.to_owned());
            }
        }
        Value::Array(
            candidates
                .into_iter()
                .take(80)
                .map(|label| json!({"label":label,"kind":6}))
                .collect(),
        )
    }

    fn hover(&self, params: &Value) -> Value {
        let Some((_, text)) = self.document_text(params) else {
            return Value::Null;
        };
        let position = &params["position"];
        let offset = offset_at(
            text,
            position["line"].as_u64().unwrap_or(0) as usize,
            position["character"].as_u64().unwrap_or(0) as usize,
        );
        let word = word_at(text, offset);
        if word.is_empty() {
            Value::Null
        } else {
            json!({"contents":{"kind":"markdown","value":format!("**{word}**\n\nSublemon local symbol information.")}})
        }
    }

    fn definition(&self, params: &Value) -> Value {
        let Some((origin_uri, text)) = self.document_text(params) else {
            return Value::Null;
        };
        let position = &params["position"];
        let offset = offset_at(
            text,
            position["line"].as_u64().unwrap_or(0) as usize,
            position["character"].as_u64().unwrap_or(0) as usize,
        );
        let word = word_at(text, offset).to_owned();
        if word.is_empty() {
            return Value::Null;
        }
        for (uri, source) in &self.documents {
            for (line_number, line) in source.lines().enumerate() {
                if !line.contains(&word) || !is_declaration_line(line, &word) {
                    continue;
                }
                if let Some(character) = line.find(&word) {
                    return json!({"uri":uri,"range":{"start":{"line":line_number,"character":line[..character].encode_utf16().count()},"end":{"line":line_number,"character":line[..character + word.len()].encode_utf16().count()}}});
                }
            }
        }
        let _ = origin_uri;
        Value::Null
    }

    fn references(&self, params: &Value) -> Value {
        let Some((_, text)) = self.document_text(params) else {
            return json!([]);
        };
        let position = &params["position"];
        let offset = offset_at(
            text,
            position["line"].as_u64().unwrap_or(0) as usize,
            position["character"].as_u64().unwrap_or(0) as usize,
        );
        let word = word_at(text, offset).to_owned();
        if word.is_empty() {
            return json!([]);
        }
        let mut locations = Vec::new();
        for (uri, source) in &self.documents {
            for (line_number, line) in source.lines().enumerate() {
                for (byte, _) in line.match_indices(&word) {
                    let left_ok = line[..byte]
                        .chars()
                        .next_back()
                        .is_none_or(|ch| !(ch.is_alphanumeric() || ch == '_'));
                    let end = byte + word.len();
                    let right_ok = line[end..]
                        .chars()
                        .next()
                        .is_none_or(|ch| !(ch.is_alphanumeric() || ch == '_'));
                    if left_ok && right_ok {
                        let start = line[..byte].encode_utf16().count();
                        let finish = line[..end].encode_utf16().count();
                        locations.push(json!({"uri":uri,"range":{"start":{"line":line_number,"character":start},"end":{"line":line_number,"character":finish}}}));
                    }
                }
            }
        }
        Value::Array(locations)
    }

    fn document_symbols(&self, params: &Value) -> Value {
        let Some((_, text)) = self.document_text(params) else {
            return json!([]);
        };
        Value::Array(text.lines().enumerate().filter_map(|(line_number, line)| {
            let name = declaration_name(line)?;
            let start = line.find(name)?;
            Some(json!({"name":name,"kind":12,"range":{"start":{"line":line_number,"character":0},"end":{"line":line_number,"character":line.encode_utf16().count()}},"selectionRange":{"start":{"line":line_number,"character":line[..start].encode_utf16().count()},"end":{"line":line_number,"character":line[..start + name.len()].encode_utf16().count()}}}))
        }).collect())
    }

    fn publish_diagnostics(&self, writer: &mut impl Write, uri: &str) -> io::Result<()> {
        let diagnostics = self
            .documents
            .get(uri)
            .map(|text| lightweight_diagnostics(text))
            .unwrap_or_default();
        write_message(
            writer,
            &json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":diagnostics}}),
        )
    }
}

fn declaration_name(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    for prefix in [
        "pub fn ",
        "fn ",
        "def ",
        "function ",
        "class ",
        "struct ",
        "enum ",
        "trait ",
        "interface ",
        "type ",
        "const ",
        "let ",
    ] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let end = rest
                .find(|ch: char| !(ch.is_alphanumeric() || ch == '_'))
                .unwrap_or(rest.len());
            if end > 0 {
                return Some(&rest[..end]);
            }
        }
    }
    None
}

fn is_declaration_line(line: &str, word: &str) -> bool {
    declaration_name(line) == Some(word)
}

fn offset_at(text: &str, wanted_line: usize, wanted_utf16_column: usize) -> usize {
    let mut global_chars = 0;
    for (line_number, line) in text.split('\n').enumerate() {
        if line_number == wanted_line {
            let mut units = 0;
            for (index, ch) in line.char_indices() {
                if units >= wanted_utf16_column {
                    return global_chars + line[..index].chars().count();
                }
                units += ch.len_utf16();
            }
            return global_chars + line.chars().count();
        }
        global_chars += line.chars().count() + 1;
    }
    text.chars().count()
}

fn word_at(text: &str, character_offset: usize) -> &str {
    let byte_offset = text
        .char_indices()
        .nth(character_offset)
        .map_or(text.len(), |(byte, _)| byte);
    let before = &text[..byte_offset];
    let after = &text[byte_offset..];
    let left = before
        .char_indices()
        .rev()
        .take_while(|(_, ch)| ch.is_alphanumeric() || *ch == '_')
        .last()
        .map_or(byte_offset, |(byte, _)| byte);
    let right = after
        .char_indices()
        .take_while(|(_, ch)| ch.is_alphanumeric() || *ch == '_')
        .last()
        .map_or(byte_offset, |(byte, ch)| byte + ch.len_utf8());
    &text[left..right]
}

fn lightweight_diagnostics(text: &str) -> Vec<Value> {
    if text.len() > MAX_ANALYSIS_BYTES {
        return Vec::new();
    }
    let mut stack: Vec<(char, usize, usize)> = Vec::new();
    let mut diagnostics = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    for (line_number, line) in text.lines().enumerate() {
        let mut chars = line.chars().enumerate().peekable();
        while let Some((column, ch)) = chars.next() {
            if let Some(delimiter) = quote {
                if !escaped && ch == delimiter {
                    quote = None;
                }
                escaped = !escaped && ch == '\\';
                if ch != '\\' {
                    escaped = false;
                }
                continue;
            }
            if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '/') {
                break;
            }
            if matches!(ch, '"' | '\'' | '`') {
                quote = Some(ch);
                continue;
            }
            if matches!(ch, '(' | '[' | '{') {
                stack.push((ch, line_number, column));
            }
            if matches!(ch, ')' | ']' | '}') {
                let expected = match ch {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                };
                if stack.last().is_some_and(|(open, _, _)| *open == expected) {
                    stack.pop();
                } else {
                    diagnostics.push(json!({"range":{"start":{"line":line_number,"character":column},"end":{"line":line_number,"character":column+1}},"severity":2,"source":"Sublemon","message":format!("Unmatched closing delimiter `{ch}`")}));
                }
            }
        }
    }
    for (open, line, column) in stack.into_iter().take(200) {
        diagnostics.push(json!({"range":{"start":{"line":line,"character":column},"end":{"line":line,"character":column+1}},"severity":2,"source":"Sublemon","message":format!("Unclosed delimiter `{open}`")}));
    }
    diagnostics
}

fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut length = None;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((header, value)) = line.trim().split_once(':')
            && header.eq_ignore_ascii_case("content-length")
        {
            length = value.trim().parse::<usize>().ok();
        }
    }
    let length = length
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length"))?;
    if length > MAX_MESSAGE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "message too large",
        ));
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(io::Error::other)
}

fn write_message(writer: &mut impl Write, value: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}
