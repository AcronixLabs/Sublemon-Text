use crate::app_state::AppState;
use crate::document::Document;
use crate::editor_metrics::LineMetrics;
use crate::lsp::{LspClient, offset_from_position, uri_to_path};
use crate::syntax::FAST_TEXT_PATH_LIMIT;
use eframe::egui;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Instant,
};

#[derive(Clone, Copy)]
struct ThemeColors {
    ui: egui::Color32,
    sidebar: egui::Color32,
    status: egui::Color32,
    editor: egui::Color32,
    gutter: egui::Color32,
    tab_active: egui::Color32,
    tab_inactive: egui::Color32,
    foreground: egui::Color32,
    is_light: bool,
}

impl ThemeColors {
    fn new(light: bool) -> Self {
        if light {
            Self {
                ui: egui::Color32::from_rgb(235, 236, 239),
                sidebar: egui::Color32::from_rgb(244, 245, 247),
                status: egui::Color32::from_rgb(225, 227, 231),
                editor: egui::Color32::from_rgb(250, 250, 252),
                gutter: egui::Color32::from_rgb(242, 243, 246),
                tab_active: egui::Color32::from_rgb(250, 250, 252),
                tab_inactive: egui::Color32::from_rgb(222, 224, 229),
                foreground: egui::Color32::from_rgb(45, 48, 55),
                is_light: true,
            }
        } else {
            Self {
                ui: egui::Color32::from_rgb(52, 51, 52),
                sidebar: egui::Color32::from_rgb(34, 37, 41),
                status: egui::Color32::from_rgb(45, 48, 54),
                editor: egui::Color32::from_rgb(49, 54, 64),
                gutter: egui::Color32::from_rgb(49, 54, 64),
                tab_active: egui::Color32::from_rgb(49, 54, 64),
                tab_inactive: egui::Color32::from_rgb(79, 83, 92),
                foreground: egui::Color32::from_rgb(204, 205, 207),
                is_light: false,
            }
        }
    }
}

#[derive(Clone, Copy)]
enum EditCommand {
    Undo,
    Redo,
    SelectAll,
}

type TreeIcon = String;

#[derive(Clone, Copy)]
enum LspNavigationKind {
    Definition,
    Declaration,
    TypeDefinition,
    Implementation,
    References,
}

impl LspNavigationKind {
    fn method(self) -> &'static str {
        match self {
            Self::Definition => "textDocument/definition",
            Self::Declaration => "textDocument/declaration",
            Self::TypeDefinition => "textDocument/typeDefinition",
            Self::Implementation => "textDocument/implementation",
            Self::References => "textDocument/references",
        }
    }
}

#[derive(Clone)]
struct EditorLocation {
    path: PathBuf,
    character_offset: usize,
}

#[derive(Clone)]
struct NavigationTarget {
    path: PathBuf,
    line: u32,
    character: u32,
}

struct PendingNavigation {
    kind: LspNavigationKind,
    origin: EditorLocation,
}

struct PendingCompletion {
    document_id: u64,
    character_offset: usize,
    text_snapshot: String,
}

#[derive(Clone)]
struct CompletionCandidate {
    label: String,
    insert_text: String,
    replace_range: Option<(usize, usize)>,
}

pub struct SublemonApp {
    welcome_screen: bool,
    app_state: AppState,
    pending_file_opens: Vec<Receiver<(PathBuf, std::io::Result<(Document, Option<LineMetrics>)>)>>,
    pending_open_paths: HashSet<PathBuf>,
    documents: Vec<Document>,
    active_document: usize,
    sidebar_visible: bool,
    status_bar_visible: bool,
    word_wrap: bool,
    markdown_preview: bool,
    light_theme: bool,
    cursor_line: usize,
    cursor_column: usize,
    cursor_offsets: HashMap<u64, usize>,
    line_metrics: HashMap<u64, LineMetrics>,
    line_metrics_pending: HashMap<u64, Instant>,
    lsp_auto_attempted_for: Option<u64>,
    pending_navigations: HashMap<i64, PendingNavigation>,
    navigation_history: Vec<EditorLocation>,
    navigation_results: Vec<NavigationTarget>,
    navigation_result_origin: Option<EditorLocation>,
    pending_completions: HashMap<i64, PendingCompletion>,
    pending_hovers: HashMap<i64, u64>,
    completion_items: Vec<CompletionCandidate>,
    completion_selection: usize,
    completion_document_id: Option<u64>,
    completion_cursor_offset: usize,
    completion_anchor: Option<egui::Pos2>,
    hover_content: Option<String>,
    path_dialog: Option<PathDialog>,
    path_input: String,
    message: Option<String>,
    close_confirmation: Option<usize>,
    pending_edit_command: Option<EditCommand>,
    workspace_roots: Vec<PathBuf>,
    workspace_entries_cache: HashMap<PathBuf, Vec<PathBuf>>,
    tree_cache_updated_at: Instant,
    expanded_folders: HashSet<PathBuf>,
    tree_icon_textures: HashMap<String, egui::TextureHandle>,
    lsp: Option<LspClient>,
    lsp_status: String,
    lsp_documents_opened: bool,
    lsp_diagnostic_count: usize,
    lsp_diagnostics: HashMap<String, Vec<String>>,
}

#[derive(Clone, Copy)]
enum PathDialog {
    SaveAs,
    ImportFolder,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AutoPairResult {
    NotHandled,
    CursorOnly,
    TextChanged,
}

impl Default for SublemonApp {
    fn default() -> Self {
        let app_state = AppState::load();
        let remembered_project = app_state.recent_projects.first().cloned();
        let (workspace_roots, settings_error) = if let Some(root) = &remembered_project {
            load_project_settings(root).unwrap_or_else(|error| {
                (
                    vec![root.clone()],
                    Some(format!("Could not read project settings: {error}")),
                )
            })
        } else {
            (Vec::new(), None)
        };
        let mut documents: Vec<Document> = remembered_project
            .iter()
            .flat_map(|root| ["README.md", "AGENTS.md"].map(|name| root.join(name)))
            .filter_map(|path| Document::open(&path).ok())
            .collect();
        if documents.is_empty() {
            documents.push(Document::untitled());
        }
        let expanded_folders = workspace_roots.iter().cloned().collect();
        Self {
            welcome_screen: workspace_roots.is_empty(),
            app_state,
            pending_file_opens: Vec::new(),
            pending_open_paths: HashSet::new(),
            documents,
            active_document: 0,
            sidebar_visible: true,
            status_bar_visible: true,
            word_wrap: true,
            markdown_preview: false,
            light_theme: false,
            cursor_line: 1,
            cursor_column: 1,
            cursor_offsets: HashMap::new(),
            line_metrics: HashMap::new(),
            line_metrics_pending: HashMap::new(),
            lsp_auto_attempted_for: None,
            pending_navigations: HashMap::new(),
            navigation_history: Vec::new(),
            navigation_results: Vec::new(),
            navigation_result_origin: None,
            pending_completions: HashMap::new(),
            pending_hovers: HashMap::new(),
            completion_items: Vec::new(),
            completion_selection: 0,
            completion_document_id: None,
            completion_cursor_offset: 0,
            completion_anchor: None,
            hover_content: None,
            path_dialog: None,
            path_input: String::new(),
            message: settings_error,
            close_confirmation: None,
            pending_edit_command: None,
            workspace_roots,
            workspace_entries_cache: HashMap::new(),
            tree_cache_updated_at: Instant::now(),
            expanded_folders,
            tree_icon_textures: HashMap::new(),
            lsp: None,
            lsp_status: "Built-in LSP starting".to_owned(),
            lsp_documents_opened: false,
            lsp_diagnostic_count: 0,
            lsp_diagnostics: HashMap::new(),
        }
    }
}

impl SublemonApp {
    fn new_document(&mut self) {
        self.welcome_screen = false;
        self.documents.push(Document::untitled());
        self.active_document = self.documents.len() - 1;
    }

    fn active_document(&self) -> &Document {
        &self.documents[self.active_document]
    }

    fn tree_icon_texture(&mut self, context: &egui::Context, icon: &str) -> egui::TextureHandle {
        if let Some(texture) = self.tree_icon_textures.get(icon) {
            return texture.clone();
        }

        let rgba = image::load_from_memory(material_icon_png(icon))
            .expect("bundled Material Icon Theme PNG must be valid")
            .into_rgba8();
        let [width, height] = [rgba.width() as usize, rgba.height() as usize];
        let color_image = egui::ColorImage::from_rgba_unmultiplied([width, height], rgba.as_raw());
        let texture = context.load_texture(
            format!("tree_icon_{icon}"),
            color_image,
            egui::TextureOptions::LINEAR,
        );
        self.tree_icon_textures
            .insert(icon.to_owned(), texture.clone());
        texture
    }

    fn show_tabs(&mut self, ui: &mut egui::Ui, colors: ThemeColors) {
        egui::Frame::new().fill(colors.tab_inactive).show(ui, |ui| {
            ui.add_space(2.0);
            egui::ScrollArea::horizontal()
                .id_salt("document_tabs")
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        let _ = ui.small_button("◀");
                        let _ = ui.small_button("▶");
                        ui.add_space(3.0);
                        let mut close_index = None;
                        for (index, document) in self.documents.iter().enumerate() {
                            let active = index == self.active_document;
                            let title = document.title(index + 1);
                            let frame = egui::Frame::new()
                                .fill(if active {
                                    colors.tab_active
                                } else {
                                    colors.tab_inactive
                                })
                                .inner_margin(egui::Margin::symmetric(12, 5));
                            frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let label = if document.is_dirty() {
                                        format!("{title}  ●")
                                    } else {
                                        title
                                    };
                                    if ui.selectable_label(active, label).clicked() {
                                        self.active_document = index;
                                    }
                                    if ui.small_button("×").clicked() {
                                        close_index = Some(index);
                                    }
                                });
                            });
                        }
                        if let Some(index) = close_index {
                            self.close_document(index);
                        }
                        if ui.small_button("+").clicked() {
                            self.new_document();
                        }
                        let _ = ui.small_button("▾");
                    });
                });
        });
        ui.separator();
    }

    fn show_folder_row(&mut self, ui: &mut egui::Ui, path: &Path, depth: usize) {
        let is_directory = path.is_dir();
        let expanded = self.expanded_folders.contains(path);
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let open_document = self
            .documents
            .iter()
            .position(|document| document.path.as_deref() == Some(path));
        let mut toggle_folder = false;
        let mut open_file = false;

        let icon = if is_directory {
            tree_folder_icon(&name, self.workspace_roots.iter().any(|root| root == path))
        } else {
            tree_file_icon(path)
        };
        let icon_texture = self.tree_icon_texture(ui.ctx(), &icon);

        ui.horizontal(|ui| {
            ui.add_space((depth * 14) as f32);
            if is_directory {
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(16.0, 22.0), egui::Sense::click());
                paint_disclosure(ui.painter(), rect, expanded);
                if response.clicked() {
                    toggle_folder = true;
                }
            } else {
                ui.add_space(16.0);
            }
            ui.add(
                egui::Image::from_texture(&icon_texture).fit_to_exact_size(egui::vec2(16.0, 16.0)),
            );
            let selected = open_document == Some(self.active_document);
            if ui.selectable_label(selected, name.as_ref()).clicked() {
                if is_directory {
                    toggle_folder = true;
                } else {
                    open_file = true;
                }
            }
        });

        if toggle_folder {
            if expanded {
                self.expanded_folders.remove(path);
            } else {
                self.expanded_folders.insert(path.to_path_buf());
            }
        }
        if open_file {
            self.open_path(path.to_path_buf());
        }
    }

    fn workspace_tree_rows(&mut self) -> Vec<(PathBuf, usize)> {
        if self.tree_cache_updated_at.elapsed().as_secs() >= 2 {
            self.workspace_entries_cache.clear();
            self.tree_cache_updated_at = Instant::now();
        }
        let mut rows = Vec::new();
        for root in self.workspace_roots.clone() {
            rows.push((root.clone(), 0));
            self.append_tree_children(&root, 1, &mut rows);
        }
        rows
    }

    fn append_tree_children(
        &mut self,
        folder: &Path,
        depth: usize,
        rows: &mut Vec<(PathBuf, usize)>,
    ) {
        if !self.expanded_folders.contains(folder) {
            return;
        }
        for child in self.workspace_entries(folder) {
            rows.push((child.clone(), depth));
            if child.is_dir() && self.expanded_folders.contains(&child) {
                self.append_tree_children(&child, depth + 1, rows);
            }
        }
    }

    fn workspace_entries(&mut self, folder: &Path) -> Vec<PathBuf> {
        if let Some(entries) = self.workspace_entries_cache.get(folder) {
            return entries.clone();
        }
        let mut entries: Vec<PathBuf> = fs::read_dir(folder)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name().is_some_and(|name| name != ".git")
                    && path.file_name().is_some_and(|name| name != "sublime_text")
                    && path
                        .file_name()
                        .is_some_and(|name| name != "sublime_text_build_4215_x64.tar.xz")
            })
            .collect();
        entries.sort_by_key(|path| {
            let is_file = path.is_file();
            (
                is_file,
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase(),
            )
        });
        self.workspace_entries_cache
            .insert(folder.to_path_buf(), entries.clone());
        entries
    }

    fn close_document(&mut self, index: usize) {
        if self.documents[index].is_dirty() {
            self.close_confirmation = Some(index);
            return;
        }

        if let (Some(lsp), Some(path)) = (&mut self.lsp, self.documents[index].path.as_deref()) {
            let _ = lsp.did_close(path);
        }

        if self.documents.len() == 1 {
            self.documents[0] = Document::untitled();
            self.active_document = 0;
            return;
        }

        self.documents.remove(index);
        if self.active_document > index {
            self.active_document -= 1;
        } else if self.active_document >= self.documents.len() {
            self.active_document = self.documents.len() - 1;
        }
    }

    fn open_path(&mut self, path: PathBuf) {
        let path = fs::canonicalize(&path).unwrap_or(path);
        if self.welcome_screen {
            if let Some(parent) = path.parent() {
                self.open_folder(parent.to_path_buf());
            }
        }
        if let Some(index) = self
            .documents
            .iter()
            .position(|item| item.path.as_deref() == Some(path.as_path()))
        {
            self.active_document = index;
            return;
        }
        if !self.pending_open_paths.insert(path.clone()) {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        let task_path = path.clone();
        let spawned = thread::Builder::new()
            .name("sublemon-file-open".into())
            .spawn(move || {
                let result = Document::open(&task_path).map(|document| {
                    let metrics = (document.text.len() > FAST_TEXT_PATH_LIMIT)
                        .then(|| LineMetrics::new(&document.text, document.revision(), usize::MAX));
                    (document, metrics)
                });
                let _ = sender.send((task_path, result));
            });
        match spawned {
            Ok(_) => {
                self.pending_file_opens.push(receiver);
                self.message = Some(format!("Opening {}…", path.display()));
            }
            Err(error) => {
                self.pending_open_paths.remove(&path);
                self.message = Some(format!("Could not start file loading: {error}"));
            }
        }
    }

    fn poll_file_opens(&mut self) {
        let mut completed = Vec::new();
        for (index, receiver) in self.pending_file_opens.iter().enumerate() {
            match receiver.try_recv() {
                Ok((path, result)) => completed.push((index, Some(path), Some(result))),
                Err(TryRecvError::Disconnected) => completed.push((index, None, None)),
                Err(TryRecvError::Empty) => {}
            }
        }
        for (index, path, result) in completed.into_iter().rev() {
            self.pending_file_opens.remove(index);
            if let Some(path) = path {
                self.pending_open_paths.remove(&path);
                match result.expect("completed open result") {
                    Ok((document, metrics)) => {
                        let document_id = document.id;
                        if self.documents.len() == 1
                            && self.documents[0].path.is_none()
                            && self.documents[0].text.is_empty()
                        {
                            self.documents[0] = document;
                            self.active_document = 0;
                        } else {
                            self.documents.push(document);
                            self.active_document = self.documents.len() - 1;
                        }
                        if let Some(metrics) = metrics {
                            self.line_metrics.insert(document_id, metrics);
                        }
                        self.message = None;
                        self.notify_active_document_open();
                    }
                    Err(error) => self.message = Some(format!("Could not open file: {error}")),
                }
            }
        }
    }

    fn open_folder(&mut self, path: PathBuf) {
        match canonical_folder(&path) {
            Ok(root) => {
                let (roots, settings_error) =
                    load_project_settings(&root).unwrap_or_else(|error| {
                        (
                            vec![root.clone()],
                            Some(format!("Could not read .sublemon.json: {error}")),
                        )
                    });
                self.workspace_roots = roots;
                self.welcome_screen = false;
                self.app_state.remember_project(root.clone());
                self.expanded_folders.clear();
                self.expanded_folders
                    .extend(self.workspace_roots.iter().cloned());
                self.workspace_entries_cache.clear();
                if let Some(lsp) = self.lsp.as_mut() {
                    let roots: Vec<_> = self.workspace_roots.iter().map(PathBuf::as_path).collect();
                    let _ = lsp.sync_workspace_folders(&roots);
                }
                self.message = settings_error;
            }
            Err(error) => self.message = Some(format!("Could not open folder: {error}")),
        }
    }

    fn import_folder(&mut self, path: PathBuf) {
        match canonical_folder(&path) {
            Ok(root) if self.workspace_roots.contains(&root) => {
                self.message = Some("That folder is already in the project.".to_owned());
            }
            Ok(root) => {
                self.expanded_folders.insert(root.clone());
                self.workspace_roots.push(root);
                self.workspace_roots[1..].sort();
                if let Some(lsp) = self.lsp.as_mut() {
                    let roots: Vec<_> = self.workspace_roots.iter().map(PathBuf::as_path).collect();
                    let _ = lsp.sync_workspace_folders(&roots);
                }
                self.message = save_project_settings(&self.workspace_roots)
                    .err()
                    .map(|error| {
                        format!("Folder added, but project settings were not saved: {error}")
                    });
            }
            Err(error) => self.message = Some(format!("Could not add folder: {error}")),
        }
    }

    fn notify_active_document_open(&mut self) {
        let Some(lsp) = self.lsp.as_mut().filter(|lsp| lsp.is_ready()) else {
            return;
        };
        if let Some(document) = self.documents.get(self.active_document) {
            if let Some(path) = document.path.as_deref() {
                if let Err(error) = lsp.did_open(path, &document.text) {
                    self.message = Some(format!("Could not notify language server: {error}"));
                }
            }
        }
    }

    fn request_lsp_navigation(&mut self, kind: LspNavigationKind) {
        if matches!(kind, LspNavigationKind::References) {
            self.navigation_results.clear();
            self.navigation_result_origin = None;
        }
        let Some(document) = self.documents.get(self.active_document) else {
            return;
        };
        let Some(path) = document.path.clone() else {
            self.message =
                Some("Save this file before asking the language server to navigate.".into());
            return;
        };
        let text = document.text.clone();
        let character_offset = self.cursor_offsets.get(&document.id).copied().unwrap_or(0);
        let Some(lsp) = self.lsp.as_mut().filter(|lsp| lsp.is_ready()) else {
            self.message = Some("Start a language server before using code navigation.".into());
            return;
        };
        let extra = if matches!(kind, LspNavigationKind::References) {
            serde_json::json!({ "context": { "includeDeclaration": true } })
        } else {
            serde_json::json!({})
        };
        match lsp.request_at(kind.method(), &path, &text, character_offset, extra) {
            Ok(id) => {
                self.pending_navigations.insert(
                    id,
                    PendingNavigation {
                        kind,
                        origin: EditorLocation {
                            path,
                            character_offset,
                        },
                    },
                );
                self.message = None;
            }
            Err(error) => self.message = Some(format!("Could not request navigation: {error}")),
        }
    }

    fn request_completion(&mut self) {
        self.request_completion_with_trigger(1);
    }

    fn request_completion_with_trigger(&mut self, trigger_kind: u8) {
        self.completion_items.clear();
        self.completion_document_id = None;
        self.completion_anchor = None;
        let Some(document) = self.documents.get(self.active_document) else {
            return;
        };
        let Some(path) = document.path.clone() else {
            self.message = Some("Save this file before requesting completions.".into());
            return;
        };
        let document_id = document.id;
        let text = document.text.clone();
        let character_offset = self.cursor_offsets.get(&document_id).copied().unwrap_or(0);
        let Some(lsp) = self.lsp.as_mut().filter(|lsp| lsp.is_ready()) else {
            self.message = Some("Start a language server before requesting completions.".into());
            return;
        };
        match lsp.request_at(
            "textDocument/completion",
            &path,
            &text,
            character_offset,
            serde_json::json!({ "context": { "triggerKind": trigger_kind } }),
        ) {
            Ok(id) => {
                self.pending_completions.insert(
                    id,
                    PendingCompletion {
                        document_id,
                        character_offset,
                        text_snapshot: text,
                    },
                );
                self.message = None;
            }
            Err(error) => self.message = Some(format!("Could not request completions: {error}")),
        }
    }

    fn request_hover(&mut self) {
        self.hover_content = None;
        let Some(document) = self.documents.get(self.active_document) else {
            return;
        };
        let Some(path) = document.path.clone() else {
            self.message = Some("Save this file before requesting hover information.".into());
            return;
        };
        let document_id = document.id;
        let text = document.text.clone();
        let character_offset = self.cursor_offsets.get(&document.id).copied().unwrap_or(0);
        let Some(lsp) = self.lsp.as_mut().filter(|lsp| lsp.is_ready()) else {
            self.message =
                Some("Start a language server before requesting hover information.".into());
            return;
        };
        match lsp.request_at(
            "textDocument/hover",
            &path,
            &text,
            character_offset,
            serde_json::json!({}),
        ) {
            Ok(id) => {
                self.pending_hovers.insert(id, document_id);
                self.message = None;
            }
            Err(error) => self.message = Some(format!("Could not request hover: {error}")),
        }
    }

    fn apply_completion(&mut self, context: &egui::Context, index: usize) {
        if self.completion_document_id != Some(self.documents[self.active_document].id) {
            self.completion_items.clear();
            return;
        }
        let Some(candidate) = self.completion_items.get(index).cloned() else {
            return;
        };
        let document = &mut self.documents[self.active_document];
        let offset = self
            .completion_cursor_offset
            .min(document.text.chars().count());
        let (start, end) = candidate
            .replace_range
            .unwrap_or_else(|| (completion_prefix_start(&document.text, offset), offset));
        let start = start.min(document.text.chars().count());
        let end = end.min(document.text.chars().count()).max(start);
        let start_byte = char_to_byte(&document.text, start);
        let end_byte = char_to_byte(&document.text, end);
        document
            .text
            .replace_range(start_byte..end_byte, &candidate.insert_text);
        document.mark_changed();
        let new_offset = start + candidate.insert_text.chars().count();
        let path = document.path.clone();
        let document_id = document.id;
        self.completion_items.clear();
        self.completion_selection = 0;
        self.completion_document_id = None;
        self.completion_anchor = None;
        self.set_editor_cursor(context, new_offset);
        if let (Some(lsp), Some(path)) = (&mut self.lsp, path) {
            if let Err(error) = lsp.did_change(&path, &self.documents[self.active_document].text) {
                self.message = Some(format!(
                    "Could not sync completion to language server: {error}"
                ));
            }
        }
        self.cursor_offsets.insert(document_id, new_offset);
    }

    fn resolve_navigation_response(
        &mut self,
        context: &egui::Context,
        message: &serde_json::Value,
    ) {
        if message.get("method").is_some() {
            return;
        }
        let Some(id) = message.get("id").and_then(serde_json::Value::as_i64) else {
            return;
        };
        let Some(pending) = self.pending_navigations.remove(&id) else {
            return;
        };
        if let Some(error) = message.get("error") {
            self.message = Some(format!("Language server navigation failed: {error}"));
            return;
        }
        let targets = locations_from_result(&message["result"]);
        if targets.is_empty() {
            self.message = Some("The language server found no navigation result.".into());
            return;
        }
        if matches!(pending.kind, LspNavigationKind::References) {
            self.navigation_results = targets;
            self.navigation_result_origin = Some(pending.origin);
        } else if let Some(target) = targets.into_iter().next() {
            self.navigate_to_target(context, target, Some(pending.origin));
        }
    }

    fn navigate_to_target(
        &mut self,
        context: &egui::Context,
        mut target: NavigationTarget,
        origin: Option<EditorLocation>,
    ) {
        target.path = fs::canonicalize(&target.path).unwrap_or(target.path);
        self.open_path(target.path.clone());
        if !self.documents[self.active_document]
            .path
            .as_ref()
            .is_some_and(|path| path == &target.path)
        {
            return;
        }
        let text = self.documents[self.active_document].text.clone();
        let character_offset = offset_from_position(&text, target.line, target.character);
        if let Some(origin) = origin {
            self.navigation_history.push(origin);
        }
        self.set_editor_cursor(context, character_offset);
    }

    fn navigate_back(&mut self, context: &egui::Context) {
        let Some(location) = self.navigation_history.pop() else {
            self.message = Some("There is no previous navigation location.".into());
            return;
        };
        self.open_path(location.path.clone());
        if self.documents[self.active_document]
            .path
            .as_ref()
            .is_some_and(|path| path == &location.path)
        {
            self.set_editor_cursor(context, location.character_offset);
        }
    }

    fn set_editor_cursor(&mut self, context: &egui::Context, character_offset: usize) {
        let document = &self.documents[self.active_document];
        let character_offset = character_offset.min(document.text.chars().count());
        let id = egui::Id::new(("sublemon_editor", document.id));
        let mut state = egui::TextEdit::load_state(context, id).unwrap_or_default();
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::one(
                egui::text::CCursor::new(character_offset),
            )));
        state.store(context, id);
        self.cursor_offsets.insert(document.id, character_offset);
        let prefix: String = document.text.chars().take(character_offset).collect();
        self.cursor_line = document_line_count(&prefix);
        self.cursor_column = prefix
            .rsplit('\n')
            .next()
            .map_or(1, |line| line.chars().count() + 1);
    }

    fn poll_language_server(&mut self, context: &egui::Context) {
        let (messages, ready) = match self.lsp.as_mut() {
            Some(lsp) => (lsp.poll(), lsp.is_ready()),
            None => return,
        };
        let mut exited = false;
        for message in messages {
            if message.get("method").is_none()
                && message.get("id") == Some(&serde_json::json!(1))
                && message.get("error").is_some()
            {
                self.message = Some(format!(
                    "Language server initialization failed: {}",
                    message["error"]
                ));
                exited = true;
            }
            match message.get("method").and_then(serde_json::Value::as_str) {
                Some("textDocument/publishDiagnostics") => {
                    let uri = message["params"]["uri"].as_str().unwrap_or("document");
                    let diagnostics: Vec<_> = message["params"]["diagnostics"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|diagnostic| {
                            let severity = match diagnostic["severity"].as_u64() {
                                Some(1) => "Error",
                                Some(2) => "Warning",
                                Some(3) => "Information",
                                Some(4) => "Hint",
                                _ => "Diagnostic",
                            };
                            format!(
                                "{uri}: {severity}: {}",
                                diagnostic["message"].as_str().unwrap_or_default()
                            )
                        })
                        .collect();
                    self.lsp_diagnostics.insert(uri.to_owned(), diagnostics);
                    self.lsp_diagnostic_count = self.lsp_diagnostics.values().map(Vec::len).sum();
                }
                Some("sublemon/error") => {
                    self.message = message["message"].as_str().map(str::to_owned);
                }
                Some("sublemon/exit") => exited = true,
                _ => {}
            }
            if message.get("method").is_none()
                && let Some(id) = message.get("id").and_then(serde_json::Value::as_i64)
            {
                if let Some(pending) = self.pending_completions.remove(&id) {
                    let unchanged = self
                        .documents
                        .iter()
                        .find(|document| document.id == pending.document_id)
                        .is_some_and(|document| document.text == pending.text_snapshot)
                        && self
                            .cursor_offsets
                            .get(&pending.document_id)
                            .is_some_and(|offset| *offset == pending.character_offset);
                    if unchanged {
                        self.completion_items =
                            completion_candidates(&message["result"], &pending.text_snapshot);
                        self.completion_selection = 0;
                        self.completion_document_id = Some(pending.document_id);
                        self.completion_cursor_offset = pending.character_offset;
                        if self.completion_items.is_empty() {
                            self.completion_document_id = None;
                        }
                    } else {
                        self.completion_items.clear();
                        self.completion_document_id = None;
                        self.completion_anchor = None;
                    }
                }
                if let Some(document_id) = self.pending_hovers.remove(&id) {
                    if self
                        .documents
                        .get(self.active_document)
                        .is_some_and(|document| document.id == document_id)
                    {
                        self.hover_content = hover_contents(&message["result"]);
                    }
                }
            }
            self.resolve_navigation_response(context, &message);
        }
        if ready {
            self.lsp_status = format!("LSP · {} diagnostics", self.lsp_diagnostic_count);
            if !self.lsp_documents_opened {
                let documents: Vec<_> = self
                    .documents
                    .iter()
                    .filter_map(|document| {
                        if document.text.len() > 512 * 1024 {
                            return None;
                        }
                        document
                            .path
                            .as_ref()
                            .map(|path| (path.clone(), document.text.clone()))
                    })
                    .collect();
                if let Some(lsp) = self.lsp.as_mut() {
                    let roots: Vec<_> = self.workspace_roots.iter().map(PathBuf::as_path).collect();
                    let _ = lsp.sync_workspace_folders(&roots);
                    for (path, text) in documents {
                        let _ = lsp.did_open(&path, &text);
                    }
                }
                self.lsp_documents_opened = true;
            }
        }
        if exited {
            if let Some(lsp) = self.lsp.take() {
                lsp.stop();
            }
            self.lsp_status = "Language server exited".into();
            self.lsp_auto_attempted_for = None;
        }
    }

    fn ensure_language_server_for_active_document(&mut self) {
        let Some(root) = self.workspace_roots.first() else {
            return;
        };
        if self.lsp.is_some() || self.lsp_auto_attempted_for == Some(u64::MAX) {
            return;
        }
        self.lsp_auto_attempted_for = Some(u64::MAX);
        match LspClient::start_builtin(root) {
            Ok(client) => {
                self.lsp = Some(client);
                self.lsp_status = "Starting Sublemon language service…".into();
                self.lsp_documents_opened = false;
                self.lsp_diagnostic_count = 0;
                self.lsp_diagnostics.clear();
            }
            Err(error) => {
                self.message = Some(format!(
                    "Could not start built-in language service: {error}"
                ))
            }
        }
    }

    fn save_active(&mut self) {
        let result = self.documents[self.active_document].save();
        self.set_save_result(result);
    }

    fn save_active_as(&mut self, path: PathBuf) {
        let result = self.documents[self.active_document].save_as(&path);
        self.set_save_result(result);
    }

    fn set_save_result(&mut self, result: std::io::Result<()>) {
        self.message = match result {
            Ok(()) => None,
            Err(error) => Some(format!("Could not save file: {error}")),
        };
        if self.message.is_none() {
            if let Some(index) = self.close_confirmation {
                if !self.documents[index].is_dirty() {
                    self.close_confirmation = None;
                    self.close_document(index);
                }
            }
        }
    }
}

impl eframe::App for SublemonApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_file_opens();
        self.ensure_language_server_for_active_document();
        self.poll_language_server(ui.ctx());
        let (definition, references, back, completion) = ui.input_mut(|input| {
            (
                input.consume_key(egui::Modifiers::NONE, egui::Key::F12),
                input.consume_key(egui::Modifiers::SHIFT, egui::Key::F12),
                input.consume_key(egui::Modifiers::ALT, egui::Key::ArrowLeft),
                input.consume_key(egui::Modifiers::CTRL, egui::Key::Space),
            )
        });
        let toggle_markdown_preview = ui.input_mut(|input| {
            let pressed = input.events.iter().any(|event| matches!(event,
                egui::Event::Key { key: egui::Key::V, pressed: true, modifiers, .. }
                    if modifiers.shift && (modifiers.ctrl || modifiers.command) && !modifiers.alt
            ));
            if pressed {
                input.events.retain(|event| !matches!(event,
                    egui::Event::Key { key: egui::Key::V, pressed: true, modifiers, .. }
                        if modifiers.shift && (modifiers.ctrl || modifiers.command) && !modifiers.alt
                ));
            }
            pressed
        });
        if toggle_markdown_preview {
            self.markdown_preview = !self.markdown_preview;
        }
        if definition {
            self.request_lsp_navigation(LspNavigationKind::Definition);
        } else if references {
            self.request_lsp_navigation(LspNavigationKind::References);
        }
        if back {
            self.navigate_back(ui.ctx());
        }
        if completion {
            self.request_completion();
        }
        if !self.completion_items.is_empty() {
            let (up, down, accept, dismiss) = ui.input_mut(|input| {
                (
                    input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                    input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                    input.consume_key(egui::Modifiers::NONE, egui::Key::Tab),
                    input.consume_key(egui::Modifiers::NONE, egui::Key::Escape),
                )
            });
            if up {
                self.completion_selection = self.completion_selection.saturating_sub(1);
            }
            if down {
                self.completion_selection =
                    (self.completion_selection + 1).min(self.completion_items.len() - 1);
            }
            if accept {
                self.apply_completion(ui.ctx(), self.completion_selection);
            }
            if dismiss {
                self.completion_items.clear();
                self.completion_document_id = None;
                self.completion_anchor = None;
            }
        }
        let colors = ThemeColors::new(self.light_theme);
        apply_editor_style(ui.ctx(), colors);

        egui::Panel::top("main_menu").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                ui.menu_button("File", |ui| {
                    if ui.button("New File").clicked() {
                        self.new_document();
                        ui.close();
                    }
                    ui.menu_button("Open", |ui| {
                        if ui.button("File…").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.open_path(path);
                            }
                            ui.close();
                        }
                        if ui.button("Folder…").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.open_folder(path);
                            }
                            ui.close();
                        }
                    });
                    ui.separator();
                    if ui.button("Save").clicked() {
                        if self.documents[self.active_document].path.is_some() {
                            self.save_active();
                        } else {
                            self.path_input.clear();
                            self.path_dialog = Some(PathDialog::SaveAs);
                        }
                        ui.close();
                    }
                    if ui.button("Save As…").clicked() {
                        self.path_input.clear();
                        self.path_dialog = Some(PathDialog::SaveAs);
                        ui.close();
                    }
                    if ui.button("Revert File").clicked() {
                        match self.documents[self.active_document].revert() {
                            Ok(()) => self.message = None,
                            Err(error) => {
                                self.message = Some(format!("Could not revert file: {error}"))
                            }
                        }
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Close File").clicked() {
                        let active = self.active_document;
                        self.close_document(active);
                        ui.close();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo                     Ctrl+Z").clicked() {
                        self.pending_edit_command = Some(EditCommand::Undo);
                        ui.close();
                    }
                    if ui.button("Redo                     Ctrl+Y").clicked() {
                        self.pending_edit_command = Some(EditCommand::Redo);
                        ui.close();
                    }
                    if ui.button("Complete             Ctrl+Space").clicked() {
                        self.request_completion();
                        ui.close();
                    }
                    ui.separator();
                    ui.add_enabled(false, egui::Button::new("Cut"));
                    ui.add_enabled(false, egui::Button::new("Copy"));
                    ui.add_enabled(false, egui::Button::new("Paste"));
                });

                ui.menu_button("Selection", |ui| {
                    if ui.button("Select All             Ctrl+A").clicked() {
                        self.pending_edit_command = Some(EditCommand::SelectAll);
                        ui.close();
                    }
                    ui.add_enabled(false, egui::Button::new("Expand Selection to Line"));
                    ui.add_enabled(false, egui::Button::new("Add Next Occurrence"));
                });

                ui.menu_button("Find", |ui| {
                    ui.add_enabled(false, egui::Button::new("Find…"));
                    ui.add_enabled(false, egui::Button::new("Replace…"));
                    ui.add_enabled(false, egui::Button::new("Find in Files…"));
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.sidebar_visible, "Show Sidebar");
                    ui.checkbox(&mut self.status_bar_visible, "Show Status Bar");
                    ui.separator();
                    ui.checkbox(&mut self.word_wrap, "Word Wrap");
                    ui.checkbox(&mut self.light_theme, "Light Theme");
                    ui.checkbox(
                        &mut self.markdown_preview,
                        "Markdown Preview    Ctrl+Shift+V",
                    );
                });

                ui.menu_button("Goto", |ui| {
                    ui.add_enabled(false, egui::Button::new("Goto Anything…"));
                    ui.add_enabled(false, egui::Button::new("Goto Line…"));
                    ui.add_enabled(false, egui::Button::new("Goto Symbol…"));
                    ui.separator();
                    if ui.button("Go to Definition    F12").clicked() {
                        self.request_lsp_navigation(LspNavigationKind::Definition);
                        ui.close();
                    }
                    if ui.button("Go to Declaration").clicked() {
                        self.request_lsp_navigation(LspNavigationKind::Declaration);
                        ui.close();
                    }
                    if ui.button("Go to Type Definition").clicked() {
                        self.request_lsp_navigation(LspNavigationKind::TypeDefinition);
                        ui.close();
                    }
                    if ui.button("Go to Implementation").clicked() {
                        self.request_lsp_navigation(LspNavigationKind::Implementation);
                        ui.close();
                    }
                    if ui.button("Find References    Shift+F12").clicked() {
                        self.request_lsp_navigation(LspNavigationKind::References);
                        ui.close();
                    }
                    if ui.button("Show Hover").clicked() {
                        self.request_hover();
                        ui.close();
                    }
                    if ui.button("Back    Alt+Left").clicked() {
                        self.navigate_back(ui.ctx());
                        ui.close();
                    }
                });

                ui.menu_button("Tools", |ui| {
                    ui.add_enabled(false, egui::Button::new("Build"));
                    ui.add_enabled(false, egui::Button::new("Command Palette…"));
                });

                ui.menu_button("Project", |ui| {
                    if ui.button("Add Folder to Project…").clicked() {
                        self.path_input.clear();
                        self.path_dialog = Some(PathDialog::ImportFolder);
                        ui.close();
                    }
                    ui.label(&self.lsp_status);
                });

                ui.menu_button("Preferences", |ui| {
                    ui.add_enabled(false, egui::Button::new("Settings"));
                    ui.add_enabled(false, egui::Button::new("Key Bindings"));
                    ui.add_enabled(false, egui::Button::new("Select Color Scheme…"));
                });

                ui.menu_button("Help", |ui| {
                    ui.label("Sublemon");
                    ui.label("An independently implemented text editor.");
                });
            });
        });

        if self.status_bar_visible && !self.welcome_screen {
            egui::Panel::bottom("status_bar")
                .frame(egui::Frame::new().fill(colors.status))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("◧");
                        ui.label(format!(
                            "Line {}, Column {}",
                            self.cursor_line, self.cursor_column
                        ));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let language = self
                                .active_document()
                                .path
                                .as_ref()
                                .and_then(|path| path.extension())
                                .map_or("Plain Text", |extension| {
                                    if extension.eq_ignore_ascii_case("md") {
                                        "Markdown"
                                    } else {
                                        "Plain Text"
                                    }
                                });
                            ui.label(language);
                            ui.label(&self.lsp_status).on_hover_text(
                                if self.lsp_diagnostics.is_empty() {
                                    self.lsp_status.clone()
                                } else {
                                    self.lsp_diagnostics
                                        .values()
                                        .flatten()
                                        .cloned()
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                },
                            );
                            ui.label("UTF-8").on_hover_text(format!(
                                "UTF-8 · {}",
                                self.active_document().line_ending.label()
                            ));
                            ui.separator();
                            ui.label("Tab Size: 4");
                            ui.separator();
                            ui.label("main ②");
                        });
                    });
                });
        }

        if self.sidebar_visible && !self.welcome_screen {
            let sidebar_default_size = (ui.ctx().content_rect().width() * 0.43).clamp(250.0, 550.0);
            egui::Panel::left("sidebar_layout_v2")
                .frame(egui::Frame::new().fill(colors.sidebar))
                .resizable(true)
                .default_size(sidebar_default_size)
                .min_size(0.0)
                .show(ui, |ui| {
                    ui.painter().rect_filled(ui.max_rect(), 0.0, colors.sidebar);
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.strong("FOLDERS");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let _ = ui.small_button("⋯");
                        });
                    });
                    ui.add_space(10.0);
                    let rows = self.workspace_tree_rows();
                    egui::ScrollArea::vertical()
                        .id_salt("project_file_tree")
                        .auto_shrink([false, false])
                        .show_rows(ui, 22.0, rows.len(), |ui, visible_rows| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            for index in visible_rows {
                                let (path, depth) = &rows[index];
                                self.show_folder_row(ui, path, *depth);
                            }
                        });
                });
        }

        let mut completion_anchor = None;
        let mut lsp_change = None;
        let mut request_completion_on_change = false;
        egui::CentralPanel::default_margins()
            .frame(egui::Frame::new().fill(colors.editor).inner_margin(0))
            .show(ui, |ui| {
                if self.welcome_screen {
                    ui.vertical_centered(|ui| {
                        ui.add_space((ui.available_height() * 0.20).max(48.0));
                        ui.heading(egui::RichText::new("Sublemon").size(34.0));
                        ui.add_space(8.0);
                        ui.label("A focused workspace for your code.");
                        ui.add_space(22.0);
                        if ui.button("Open Project…").clicked()
                            && let Some(path) = rfd::FileDialog::new().pick_folder()
                        {
                            self.open_folder(path);
                        }
                        if ui.button("Open File…").clicked()
                            && let Some(path) = rfd::FileDialog::new().pick_file()
                        {
                            self.open_path(path);
                        }
                        let recent_projects = self.app_state.recent_projects.clone();
                        if !recent_projects.is_empty() {
                            ui.add_space(18.0);
                            ui.weak("Recent projects");
                            for project in recent_projects {
                                let label = project.to_string_lossy().into_owned();
                                if ui.small_button(label).clicked() {
                                    self.open_folder(project);
                                }
                            }
                        }
                        if !self.pending_open_paths.is_empty() {
                            ui.add_space(12.0);
                            ui.spinner();
                            ui.label("Loading file in the background…");
                        }
                    });
                    return;
                }
                self.show_tabs(ui, colors);
                let gutter_width = 56.0;
                let content_width = ui.available_width();
                let editor_width =
                    (content_width - gutter_width - ui.spacing().item_spacing.x).max(1.0);
                let large_document = self.active_document().text.len() > FAST_TEXT_PATH_LIMIT;
                let effective_word_wrap = self.word_wrap && !large_document;
                let chars_per_visual_line =
                    ((editor_width - 16.0) / (14.0 * 0.60)).floor().max(1.0) as usize;
                // Keep only line lengths for gutter sizing instead of cloning the full
                // document on every UI frame, which was costly for large files.
                let document_id = self.active_document().id;
                let wrap_width_chars = if effective_word_wrap {
                    chars_per_visual_line
                } else {
                    usize::MAX
                };
                let revision = self.active_document().revision();
                let metrics_current = self
                    .line_metrics
                    .get(&document_id)
                    .is_some_and(|metrics| metrics.matches(revision, wrap_width_chars));
                let metrics_can_be_deferred = large_document
                    && self
                        .line_metrics
                        .get(&document_id)
                        .is_some_and(|metrics| metrics.matches_wrap_width(wrap_width_chars))
                    && self
                        .line_metrics_pending
                        .get(&document_id)
                        .is_some_and(|since| {
                            since.elapsed() < std::time::Duration::from_millis(250)
                        });
                if !metrics_current && !metrics_can_be_deferred {
                    self.line_metrics.insert(
                        document_id,
                        LineMetrics::new(&self.active_document().text, revision, wrap_width_chars),
                    );
                    self.line_metrics_pending.remove(&document_id);
                }
                let metrics = &self.line_metrics[&document_id];
                let preview_lines: Vec<_> = if self.markdown_preview {
                    self.active_document()
                        .text
                        .lines()
                        .map(|line| (line.is_empty(), markdown_preview_text(line)))
                        .collect()
                } else {
                    Vec::new()
                };
                let visual_rows = metrics.visual_rows();
                // TextEdit itself does not scroll its contents. Put the gutter and editor
                // inside one shared viewport and give the row its full document height, so
                // wheel/trackpad scrolling moves both columns together.
                let content_height = (visual_rows as f32 * 19.0 + 16.0).max(ui.available_height());
                egui::ScrollArea::vertical()
                    .id_salt("editor_text_viewport")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if self.markdown_preview {
                            egui::Frame::new().fill(colors.editor).show(ui, |ui| {
                                ui.set_min_height(content_height);
                                ui.set_min_width(editor_width + gutter_width);
                                ui.add_space(18.0);
                                for (is_empty, line) in &preview_lines {
                                    if *is_empty {
                                        ui.add_space(7.0);
                                    } else {
                                        ui.label(line.clone());
                                    }
                                }
                            });
                        } else {
                            ui.horizontal_top(|ui| {
                                let (_, rect) =
                                    ui.allocate_space(egui::vec2(gutter_width, content_height));
                                let painter = ui.painter_at(rect);
                                painter.rect_filled(rect, 0.0, colors.gutter);
                                let line_height = 19.0;
                                let text_top = rect.top() + 8.0;
                                let clip = ui.clip_rect();
                                let first_row =
                                    ((clip.top() - text_top) / line_height).floor().max(0.0)
                                        as usize;
                                let end_row = ((clip.bottom() - text_top) / line_height)
                                    .ceil()
                                    .max(first_row as f32)
                                    as usize;
                                let visible_lines =
                                    metrics.lines_in_visual_rows(first_row, end_row);
                                let active_line_color = if colors.is_light {
                                    egui::Color32::from_rgb(222, 229, 240)
                                } else {
                                    egui::Color32::from_rgb(70, 80, 94)
                                };
                                let number_color = if colors.is_light {
                                    egui::Color32::from_rgb(105, 115, 130)
                                } else {
                                    egui::Color32::from_rgb(118, 133, 153)
                                };
                                for index in visible_lines {
                                    let line = index + 1;
                                    let first_line_row = metrics.visual_row_start(index);
                                    if line == self.cursor_line {
                                        for row in first_line_row
                                            ..first_line_row + metrics.line_visual_rows(index)
                                        {
                                            painter.rect_filled(
                                                egui::Rect::from_min_size(
                                                    egui::pos2(
                                                        rect.left(),
                                                        text_top + row as f32 * line_height,
                                                    ),
                                                    egui::vec2(rect.width(), line_height),
                                                ),
                                                0.0,
                                                active_line_color,
                                            );
                                        }
                                    }
                                    let y = text_top + first_line_row as f32 * line_height;
                                    painter.text(
                                        egui::pos2(rect.right() - 8.0, y),
                                        egui::Align2::RIGHT_TOP,
                                        line.to_string(),
                                        egui::FontId::monospace(12.0),
                                        number_color,
                                    );
                                }
                                ui.allocate_ui_with_layout(
                                    egui::vec2(editor_width, content_height),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        let document_index = self.active_document;
                                        let document = &mut self.documents[document_index];
                                        let editor_id =
                                            egui::Id::new(("sublemon_editor", document.id));
                                        if egui::TextEdit::load_state(ui.ctx(), editor_id).is_none()
                                        {
                                            let mut state =
                                                egui::TextEdit::load_state(ui.ctx(), editor_id)
                                                    .unwrap_or_default();
                                            let mut caret = 0;
                                            for (line_index, line) in
                                                document.text.split('\n').take(9).enumerate()
                                            {
                                                caret += line.chars().count();
                                                if line_index < 8 {
                                                    caret += 1;
                                                }
                                            }
                                            state.cursor.set_char_range(Some(
                                                egui::text::CCursorRange::one(
                                                    egui::text::CCursor::new(
                                                        caret.min(document.text.chars().count()),
                                                    ),
                                                ),
                                            ));
                                            state.store(ui.ctx(), editor_id);
                                        }
                                        let mut pre_edit_changed = prepare_auto_pairs(
                                            ui.ctx(),
                                            editor_id,
                                            &mut document.text,
                                        ) == AutoPairResult::TextChanged;
                                        if prepare_enter_indentation(
                                            ui.ctx(),
                                            editor_id,
                                            &mut document.text,
                                        ) {
                                            pre_edit_changed = true;
                                        }
                                        if pre_edit_changed {
                                            self.completion_items.clear();
                                            self.completion_document_id = None;
                                            self.completion_anchor = None;
                                            document.mark_changed();
                                            if document.text.len() > FAST_TEXT_PATH_LIMIT {
                                                self.line_metrics_pending
                                                    .insert(document.id, Instant::now());
                                            }
                                            if document.text.len() <= 512 * 1024
                                                && let Some(path) = document.path.clone()
                                            {
                                                lsp_change = Some((path, document.text.clone()));
                                            }
                                        }
                                        let syntax_path = document.path.clone();
                                        let syntax_revision = document.revision();
                                        let mut layouter =
                                        |ui: &egui::Ui,
                                         buffer: &dyn egui::TextBuffer,
                                         wrap_width: f32| {
                                            crate::syntax::layout_cached(
                                                ui.ctx(), buffer.as_str(), syntax_revision,
                                                syntax_path.as_deref(), wrap_width, colors.is_light,
                                            )
                                        };
                                        let editor = egui::TextEdit::multiline(&mut document.text)
                                            .id(editor_id)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(if effective_word_wrap {
                                                ui.available_width()
                                            } else {
                                                f32::INFINITY
                                            })
                                            .desired_rows(40)
                                            .frame(egui::Frame::NONE)
                                            .margin(egui::Margin::symmetric(8, 8))
                                            .code_editor()
                                            .layouter(&mut layouter)
                                            .interactive(!document.read_only);
                                        let output = editor.show(ui);
                                        if output.response.changed() {
                                            self.completion_items.clear();
                                            self.completion_document_id = None;
                                            self.completion_anchor = None;
                                            document.mark_changed();
                                            if document.text.len() > 512 * 1024 {
                                                self.line_metrics_pending
                                                    .insert(document.id, Instant::now());
                                            }
                                            request_completion_on_change =
                                                output.cursor_range.is_some_and(|range| {
                                                    let cursor: usize = range.primary.index.into();
                                                    document
                                                        .text
                                                        .chars()
                                                        .take(cursor)
                                                        .last()
                                                        .is_some_and(|ch| {
                                                            ch.is_alphanumeric()
                                                                || matches!(ch, '_' | '$' | '.')
                                                        })
                                                });
                                            if document.text.len() <= 512 * 1024
                                                && let Some(path) = document.path.clone()
                                            {
                                                lsp_change = Some((path, document.text.clone()));
                                            }
                                        }
                                        let mut cursor_range = output.cursor_range;
                                        if let Some(command) = self.pending_edit_command.take() {
                                            let current_range = cursor_range.unwrap_or_else(|| {
                                                egui::text::CCursorRange::one(
                                                    egui::text::CCursor::new(
                                                        document.text.chars().count(),
                                                    ),
                                                )
                                            });
                                            let current = (current_range, document.text.clone());
                                            let mut undoer = output.state.undoer();
                                            let replacement = match command {
                                                EditCommand::Undo => undoer.undo(&current).cloned(),
                                                EditCommand::Redo => undoer.redo(&current).cloned(),
                                                EditCommand::SelectAll => Some((
                                                    egui::text::CCursorRange::two(
                                                        egui::text::CCursor::new(0),
                                                        egui::text::CCursor::new(
                                                            document.text.chars().count(),
                                                        ),
                                                    ),
                                                    document.text.clone(),
                                                )),
                                            };
                                            if let Some((range, text)) = replacement {
                                                document.text = text;
                                                document.mark_changed();
                                                cursor_range = Some(range);
                                                let mut state = output.state;
                                                state.cursor.set_char_range(Some(range));
                                                state.store(ui.ctx(), output.response.id);
                                            }
                                        }
                                        if let Some(cursor_range) = cursor_range {
                                            let cursor_index: usize =
                                                cursor_range.primary.index.into();
                                            let text_origin = output.galley_pos
                                                - egui::vec2(output.galley.rect.left(), 0.0);
                                            let cursor_rect = output
                                                .galley
                                                .pos_from_cursor(cursor_range.primary)
                                                .translate(text_origin.to_vec2());
                                            completion_anchor = Some(cursor_rect.left_top());
                                            self.cursor_offsets.insert(document.id, cursor_index);
                                            if self.completion_document_id == Some(document.id)
                                                && cursor_index != self.completion_cursor_offset
                                            {
                                                self.completion_items.clear();
                                                self.completion_document_id = None;
                                                self.completion_anchor = None;
                                            }
                                            (self.cursor_line, self.cursor_column) =
                                                metrics.cursor_line_column(cursor_index);
                                        }
                                    },
                                );
                            });
                        }
                    });
            });

        self.completion_anchor = completion_anchor;

        if let Some((path, text)) = lsp_change {
            if let Some(lsp) = self.lsp.as_mut() {
                if let Err(error) = lsp.did_change(&path, &text) {
                    self.message = Some(format!(
                        "Could not sync document to language server: {error}"
                    ));
                }
            }
        }

        if request_completion_on_change {
            self.request_completion_with_trigger(2);
        }

        if let Some(dialog) = self.path_dialog {
            let title = match dialog {
                PathDialog::SaveAs => "Save As",
                PathDialog::ImportFolder => "Add Folder to Project",
            };
            let mut open = true;
            let mut submit = false;
            let mut cancel = false;
            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.label(match dialog {
                        PathDialog::SaveAs => "Enter a file path:",
                        PathDialog::ImportFolder => "Enter a folder path:",
                    });
                    let response = ui
                        .add(egui::TextEdit::singleline(&mut self.path_input).desired_width(360.0));
                    if response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    {
                        submit = true;
                    }
                    ui.horizontal(|ui| {
                        if ui.button(title).clicked() {
                            submit = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel = true;
                        }
                    });
                });
            if submit {
                let path = PathBuf::from(self.path_input.trim());
                if !path.as_os_str().is_empty() {
                    match dialog {
                        PathDialog::SaveAs => self.save_active_as(path),
                        PathDialog::ImportFolder => self.import_folder(path),
                    }
                }
                open = false;
            }
            if cancel {
                open = false;
            }
            if !open {
                self.path_dialog = None;
            }
        }

        if !self.navigation_results.is_empty() {
            let results = self.navigation_results.clone();
            let mut open = true;
            let mut selected = None;
            egui::Window::new("References")
                .collapsible(false)
                .resizable(true)
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("{} results", results.len()));
                    egui::ScrollArea::vertical()
                        .max_height(360.0)
                        .show(ui, |ui| {
                            for (index, target) in results.iter().enumerate() {
                                let filename = target.path.file_name().map_or_else(
                                    || target.path.display().to_string(),
                                    |name| name.to_string_lossy().into_owned(),
                                );
                                if ui
                                    .selectable_label(
                                        false,
                                        format!(
                                            "{filename}:{}  {}",
                                            target.line + 1,
                                            target.path.display()
                                        ),
                                    )
                                    .clicked()
                                {
                                    selected = Some(index);
                                }
                            }
                        });
                });
            if let Some(index) = selected {
                let target = self.navigation_results[index].clone();
                let origin = self.navigation_result_origin.take();
                self.navigation_results.clear();
                self.navigate_to_target(ui.ctx(), target, origin);
            } else if !open {
                self.navigation_results.clear();
                self.navigation_result_origin = None;
            }
        }

        if !self.completion_items.is_empty() {
            let document = &self.documents[self.active_document];
            if self.completion_document_id == Some(document.id)
                && let (Some(candidate), Some(anchor)) = (
                    self.completion_items.get(self.completion_selection),
                    self.completion_anchor,
                )
            {
                let offset = self
                    .completion_cursor_offset
                    .min(document.text.chars().count());
                let prefix_start = completion_prefix_start(&document.text, offset);
                let prefix: String = document
                    .text
                    .chars()
                    .skip(prefix_start)
                    .take(offset.saturating_sub(prefix_start))
                    .collect();
                let insertion = candidate.insert_text.lines().next().unwrap_or_default();
                let suffix = insertion
                    .strip_prefix(&prefix)
                    .filter(|suffix| !suffix.is_empty())
                    .unwrap_or(&candidate.label);
                let suggestion: String = suffix.chars().take(64).collect();
                if !suggestion.is_empty() {
                    // Inline continuation at the caret: it stays visually quiet and
                    // leaves the surrounding editor content unobstructed.
                    let painter = ui.ctx().layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("sublemon_inline_completion"),
                    ));
                    let ghost = if colors.is_light {
                        egui::Color32::from_rgb(139, 148, 161)
                    } else {
                        egui::Color32::from_rgb(112, 125, 145)
                    };
                    let text_rect = painter.text(
                        anchor,
                        egui::Align2::LEFT_TOP,
                        suggestion,
                        egui::FontId::monospace(14.0),
                        ghost,
                    );
                    painter.text(
                        text_rect.right_top() + egui::vec2(8.0, 2.0),
                        egui::Align2::LEFT_TOP,
                        "Tab ↹",
                        egui::FontId::proportional(10.0),
                        ghost,
                    );
                }
            }
        }

        if let Some(hover) = self.hover_content.clone() {
            let mut open = true;
            egui::Window::new("Language Server Hover")
                .collapsible(false)
                .resizable(true)
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.label(egui::RichText::new(hover).monospace());
                });
            if !open {
                self.hover_content = None;
            }
        }

        if let Some(index) = self.close_confirmation {
            let title = self
                .documents
                .get(index)
                .map_or_else(|| "document".to_owned(), |doc| doc.title(index + 1));
            let mut save = false;
            let mut discard = false;
            let mut cancel = false;
            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("Save changes to {title} before closing?"));
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            save = true;
                        }
                        if ui.button("Discard").clicked() {
                            discard = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel = true;
                        }
                    });
                });
            if save {
                self.active_document = index;
                if self.documents[index].path.is_some() {
                    self.save_active();
                } else {
                    self.active_document = index;
                    self.path_input.clear();
                    self.path_dialog = Some(PathDialog::SaveAs);
                }
            } else if discard {
                self.documents[index].discard_changes();
                self.close_confirmation = None;
                self.close_document(index);
            } else if cancel {
                self.close_confirmation = None;
            }
        }

        if let Some(message) = self.message.clone() {
            egui::Area::new(egui::Id::new("document_message"))
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 42.0))
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(message);
                            if ui.small_button("×").clicked() {
                                self.message = None;
                            }
                        });
                    });
                });
        }
    }
}

fn prepare_enter_indentation(
    context: &egui::Context,
    editor_id: egui::Id,
    text: &mut String,
) -> bool {
    let has_focus = context.memory(|memory| memory.has_focus(editor_id));
    if !has_focus {
        return false;
    }
    let enter_pressed = context.input(|input| {
        input.events.iter().any(|event| {
            matches!(event,
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    modifiers,
                    ..
                } if !modifiers.ctrl && !modifiers.command && !modifiers.alt
            )
        })
    });
    if !enter_pressed {
        return false;
    }

    let Some(mut state) = egui::TextEdit::load_state(context, editor_id) else {
        return false;
    };
    let range = state.cursor.char_range().unwrap_or_else(|| {
        egui::text::CCursorRange::one(egui::text::CCursor::new(text.chars().count()))
    });
    let selected = range.as_sorted_char_range();
    let start_char: usize = selected.start.into();
    let end_char: usize = selected.end.into();
    let start_byte = char_to_byte(text, start_char);
    let end_byte = char_to_byte(text, end_char);
    let line_start = text[..start_byte].rfind('\n').map_or(0, |index| index + 1);
    let line_prefix = &text[line_start..start_byte];
    let mut indentation: String = line_prefix
        .chars()
        .take_while(|character| matches!(character, ' ' | '\t'))
        .collect();
    if line_prefix.trim_end().ends_with(['{', '[', '(']) {
        indentation.push_str("    ");
    }
    let caret_char = text[..start_byte].chars().count() + 1 + indentation.chars().count();
    text.replace_range(start_byte..end_byte, &format!("\n{indentation}"));
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::one(
            egui::text::CCursor::new(caret_char),
        )));
    state.store(context, editor_id);

    context.input_mut(|input| {
        input.events.retain(|event| {
            !matches!(event,
                egui::Event::Key { key: egui::Key::Enter, pressed: true, modifiers,
                    .. } if !modifiers.ctrl && !modifiers.command && !modifiers.alt
            )
        });
    });
    true
}

fn prepare_auto_pairs(
    context: &egui::Context,
    editor_id: egui::Id,
    text: &mut String,
) -> AutoPairResult {
    if !context.memory(|memory| memory.has_focus(editor_id)) {
        return AutoPairResult::NotHandled;
    }
    let Some(mut state) = egui::TextEdit::load_state(context, editor_id) else {
        return AutoPairResult::NotHandled;
    };
    let Some(range) = state.cursor.char_range() else {
        return AutoPairResult::NotHandled;
    };
    let selected = range.as_sorted_char_range();
    let start_char: usize = selected.start.into();
    let end_char: usize = selected.end.into();
    let start_byte = char_to_byte(text, start_char);
    let end_byte = char_to_byte(text, end_char);
    let next_char = text[end_byte..].chars().next();
    let event = context.input(|input| {
        input.events.iter().find_map(|event| match event {
            egui::Event::Text(value) if value.chars().count() == 1 => {
                value.chars().next().map(|ch| (ch, value.clone()))
            }
            _ => None,
        })
    });
    let Some((character, event_text)) = event else {
        return AutoPairResult::NotHandled;
    };
    let pair = match character {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        '"' => Some('"'),
        '\'' => Some('\''),
        '`' => Some('`'),
        _ => None,
    };
    let result = if matches!(character, ')' | ']' | '}' | '"' | '\'' | '`')
        && next_char == Some(character)
    {
        let caret = start_char + 1;
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::one(
                egui::text::CCursor::new(caret),
            )));
        AutoPairResult::CursorOnly
    } else if let Some(close) = pair {
        let selection = &text[start_byte..end_byte];
        let replacement = format!("{character}{selection}{close}");
        let caret_start = start_char + 1;
        let caret_end = caret_start + selection.chars().count();
        text.replace_range(start_byte..end_byte, &replacement);
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::new(caret_start),
                egui::text::CCursor::new(caret_end),
            )));
        AutoPairResult::TextChanged
    } else {
        return AutoPairResult::NotHandled;
    };
    state.store(context, editor_id);
    context.input_mut(|input| {
        input
            .events
            .retain(|event| !matches!(event, egui::Event::Text(value) if value == &event_text));
    });
    result
}

fn markdown_preview_text(line: &str) -> egui::RichText {
    let trimmed = line.trim_start();
    if let Some(heading) = trimmed.strip_prefix("###### ") {
        return egui::RichText::new(heading).size(16.0).strong();
    }
    if let Some(heading) = trimmed.strip_prefix("##### ") {
        return egui::RichText::new(heading).size(18.0).strong();
    }
    if let Some(heading) = trimmed.strip_prefix("#### ") {
        return egui::RichText::new(heading).size(20.0).strong();
    }
    if let Some(heading) = trimmed.strip_prefix("### ") {
        return egui::RichText::new(heading).size(22.0).strong();
    }
    if let Some(heading) = trimmed.strip_prefix("## ") {
        return egui::RichText::new(heading).size(25.0).strong();
    }
    if let Some(heading) = trimmed.strip_prefix("# ") {
        return egui::RichText::new(heading).size(29.0).strong();
    }
    if let Some(item) = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
    {
        return egui::RichText::new(format!("• {item}"));
    }
    if let Some(quote) = trimmed.strip_prefix("> ") {
        return egui::RichText::new(format!("│  {quote}"))
            .color(egui::Color32::from_rgb(160, 175, 195));
    }
    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        return egui::RichText::new(trimmed)
            .monospace()
            .color(egui::Color32::from_rgb(150, 172, 202));
    }
    egui::RichText::new(line.to_owned())
}

fn char_to_byte(text: &str, character_index: usize) -> usize {
    text.char_indices()
        .nth(character_index)
        .map_or(text.len(), |(byte_index, _)| byte_index)
}

fn tree_file_icon(path: &Path) -> TreeIcon {
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    let lower_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let associations = tree_icon_associations();
    if let Some(icon) = associations.files.get(&filename) {
        return icon.clone();
    }
    if let Some((_, icon)) = associations
        .files
        .iter()
        .find(|(pattern, _)| pattern.contains('/') && lower_path.ends_with(pattern.as_str()))
    {
        return icon.clone();
    }

    // Match the longest suffix first for compound extensions such as d.ts and schema.json.
    associations
        .extensions
        .iter()
        .find(|(pattern, _)| lower_path.ends_with(&format!(".{pattern}")))
        .map_or_else(|| "document".to_owned(), |(_, icon)| icon.clone())
}

fn tree_folder_icon(name: &str, is_root: bool) -> TreeIcon {
    if is_root {
        return "folder-root".to_owned();
    }
    tree_icon_associations()
        .folders
        .get(&name.to_ascii_lowercase())
        .cloned()
        .unwrap_or_else(|| "folder".to_owned())
}

fn canonical_folder(path: &Path) -> std::io::Result<PathBuf> {
    let root = fs::canonicalize(path)?;
    if !root.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "the selected path is not a folder",
        ));
    }
    Ok(root)
}

fn load_project_settings(primary_root: &Path) -> std::io::Result<(Vec<PathBuf>, Option<String>)> {
    let primary = fs::canonicalize(primary_root).unwrap_or_else(|_| primary_root.to_path_buf());
    let settings_path = primary.join(".sublemon.json");
    let source = match fs::read_to_string(&settings_path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((vec![primary], None));
        }
        Err(error) => return Err(error),
    };
    let settings: serde_json::Value = serde_json::from_str(&source)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    let mut roots = vec![primary.clone()];
    if let Some(configured_roots) = settings["workspace_folders"].as_array() {
        for configured in configured_roots
            .iter()
            .filter_map(serde_json::Value::as_str)
        {
            let configured = PathBuf::from(configured);
            let configured = if configured.is_absolute() {
                configured
            } else {
                primary.join(configured)
            };
            if let Ok(root) = canonical_folder(&configured)
                && !roots.contains(&root)
            {
                roots.push(root);
            }
        }
    }
    Ok((roots, None))
}

fn save_project_settings(roots: &[PathBuf]) -> std::io::Result<()> {
    let Some(primary) = roots.first() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "there is no primary project folder",
        ));
    };
    let path = primary.join(".sublemon.json");
    let mut settings = if path.exists() {
        let source = fs::read_to_string(&path)?;
        serde_json::from_str::<serde_json::Value>(&source)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?
    } else {
        serde_json::json!({})
    };
    let Some(object) = settings.as_object_mut() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            ".sublemon.json must contain a JSON object",
        ));
    };
    object.insert("version".into(), serde_json::json!(1));
    object.insert(
        "workspace_folders".into(),
        serde_json::json!(
            roots
                .iter()
                .map(|root| root.to_string_lossy())
                .collect::<Vec<_>>()
        ),
    );
    object.remove("language_server_command");
    fs::write(
        path,
        serde_json::to_vec_pretty(&settings).map_err(std::io::Error::other)?,
    )
}

fn locations_from_result(result: &serde_json::Value) -> Vec<NavigationTarget> {
    let values: Vec<&serde_json::Value> = match result {
        serde_json::Value::Array(values) => values.iter().collect(),
        serde_json::Value::Object(_) => vec![result],
        _ => Vec::new(),
    };
    values
        .into_iter()
        .filter_map(|value| {
            let uri = value
                .get("targetUri")
                .or_else(|| value.get("uri"))?
                .as_str()?;
            let range = value
                .get("targetSelectionRange")
                .or_else(|| value.get("range"))
                .or_else(|| value.get("targetRange"))?;
            let start = &range["start"];
            Some(NavigationTarget {
                path: uri_to_path(uri)?,
                line: start["line"].as_u64()? as u32,
                character: start["character"].as_u64()? as u32,
            })
        })
        .collect()
}

fn completion_candidates(result: &serde_json::Value, text: &str) -> Vec<CompletionCandidate> {
    let items = result
        .as_array()
        .or_else(|| result.get("items").and_then(serde_json::Value::as_array));
    items
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let label = item.get("label")?.as_str()?;
            let edit = item.get("textEdit");
            let range = edit
                .and_then(|edit| edit.get("range").or_else(|| edit.get("replace")))
                .and_then(|range| {
                    let start = &range["start"];
                    let end = &range["end"];
                    Some((
                        offset_from_position(
                            text,
                            start["line"].as_u64()? as u32,
                            start["character"].as_u64()? as u32,
                        ),
                        offset_from_position(
                            text,
                            end["line"].as_u64()? as u32,
                            end["character"].as_u64()? as u32,
                        ),
                    ))
                });
            let insert_text = edit
                .and_then(|edit| edit.get("newText"))
                .and_then(serde_json::Value::as_str)
                .or_else(|| item.get("insertText").and_then(serde_json::Value::as_str))
                .unwrap_or(label)
                .to_owned();
            Some(CompletionCandidate {
                label: label.to_owned(),
                insert_text,
                replace_range: range,
            })
        })
        .collect()
}

fn hover_contents(result: &serde_json::Value) -> Option<String> {
    fn extract(value: &serde_json::Value, output: &mut Vec<String>) {
        match value {
            serde_json::Value::String(text) => output.push(text.clone()),
            serde_json::Value::Array(items) => {
                for item in items {
                    extract(item, output);
                }
            }
            serde_json::Value::Object(object) => {
                if let Some(text) = object.get("value").and_then(serde_json::Value::as_str) {
                    output.push(text.to_owned());
                }
            }
            _ => {}
        }
    }
    let mut output = Vec::new();
    extract(result.get("contents")?, &mut output);
    let text = output.join("\n\n");
    (!text.trim().is_empty()).then_some(text)
}

fn completion_prefix_start(text: &str, cursor_offset: usize) -> usize {
    let prefix: Vec<char> = text.chars().take(cursor_offset).collect();
    prefix
        .iter()
        .rposition(|ch| !(ch.is_alphanumeric() || matches!(ch, '_' | '$')))
        .map_or(0, |index| index + 1)
}

struct TreeIconAssociations {
    files: HashMap<String, String>,
    extensions: Vec<(String, String)>,
    folders: HashMap<String, String>,
}

fn tree_icon_associations() -> &'static TreeIconAssociations {
    static ASSOCIATIONS: OnceLock<TreeIconAssociations> = OnceLock::new();
    ASSOCIATIONS.get_or_init(|| {
        let mut associations = TreeIconAssociations {
            files: HashMap::new(),
            extensions: Vec::new(),
            folders: HashMap::new(),
        };
        for line in include_str!("../assets/material-icon-theme/associations.tsv").lines() {
            let mut fields = line.split('\t');
            let Some(kind) = fields.next() else { continue };
            let Some(pattern) = fields.next() else {
                continue;
            };
            let Some(icon) = fields.next() else { continue };
            match kind {
                "name" => {
                    associations
                        .files
                        .insert(pattern.to_owned(), icon.to_owned());
                }
                "ext" => associations
                    .extensions
                    .push((pattern.to_owned(), icon.to_owned())),
                "folder" => {
                    associations
                        .folders
                        .insert(pattern.to_owned(), icon.to_owned());
                }
                _ => {}
            }
        }
        associations
            .extensions
            .sort_by_key(|(pattern, _)| std::cmp::Reverse(pattern.len()));
        associations
    })
}

fn material_icon_png(name: &str) -> &'static [u8] {
    MATERIAL_ICON_PNGS
        .iter()
        .find_map(|(icon_name, bytes)| (*icon_name == name).then_some(*bytes))
        .or_else(|| {
            MATERIAL_ICON_PNGS
                .iter()
                .find_map(|(icon_name, bytes)| (*icon_name == "document").then_some(*bytes))
        })
        .expect("Material Icon Theme must include its document icon")
}

include!(concat!(env!("OUT_DIR"), "/material_icon_pngs.rs"));

fn paint_disclosure(painter: &egui::Painter, rect: egui::Rect, expanded: bool) {
    let center = rect.center();
    let color = egui::Color32::from_rgb(145, 157, 174);
    let points = if expanded {
        [
            egui::pos2(center.x - 3.0, center.y - 1.0),
            egui::pos2(center.x + 3.0, center.y - 1.0),
            egui::pos2(center.x, center.y + 3.0),
        ]
    } else {
        [
            egui::pos2(center.x - 1.0, center.y - 3.0),
            egui::pos2(center.x - 1.0, center.y + 3.0),
            egui::pos2(center.x + 3.0, center.y),
        ]
    };
    painter.add(egui::Shape::convex_polygon(
        points.to_vec(),
        color,
        egui::Stroke::NONE,
    ));
}

fn apply_editor_style(context: &egui::Context, colors: ThemeColors) {
    context.set_theme(if colors.is_light {
        egui::Theme::Light
    } else {
        egui::Theme::Dark
    });
    let mut visuals = if colors.is_light {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    visuals.panel_fill = colors.ui;
    visuals.window_fill = colors.ui;
    visuals.extreme_bg_color = colors.editor;
    visuals.code_bg_color = colors.editor;
    visuals.faint_bg_color = if colors.is_light {
        egui::Color32::from_rgb(232, 234, 238)
    } else {
        egui::Color32::from_rgb(57, 62, 73)
    };
    visuals.selection.bg_fill = if colors.is_light {
        egui::Color32::from_rgb(194, 211, 239)
    } else {
        egui::Color32::from_rgb(70, 80, 94)
    };
    visuals.selection.stroke.color = if colors.is_light {
        egui::Color32::from_rgb(72, 102, 150)
    } else {
        egui::Color32::from_rgb(173, 186, 207)
    };
    visuals.widgets.noninteractive.bg_fill = colors.ui;
    visuals.widgets.inactive.bg_fill = colors.ui;
    visuals.widgets.inactive.fg_stroke.color = colors.foreground;
    visuals.widgets.hovered.bg_fill = if colors.is_light {
        egui::Color32::from_rgb(220, 225, 233)
    } else {
        egui::Color32::from_rgb(66, 67, 60)
    };
    visuals.widgets.hovered.fg_stroke.color = colors.foreground;
    visuals.widgets.active.bg_fill = if colors.is_light {
        egui::Color32::from_rgb(205, 214, 229)
    } else {
        egui::Color32::from_rgb(81, 81, 72)
    };
    visuals.widgets.active.fg_stroke.color = colors.foreground;
    context.set_visuals(visuals);

    context.style_mut_of(context.theme(), |style| {
        style.spacing.item_spacing = egui::vec2(4.0, 3.0);
        style.spacing.button_padding = egui::vec2(5.0, 3.0);
        style.visuals.dark_mode = !colors.is_light;
    });
}

fn document_line_count(text: &str) -> usize {
    text.bytes().filter(|byte| *byte == b'\n').count() + 1
}
