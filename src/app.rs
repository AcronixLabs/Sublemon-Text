use crate::document::Document;
use eframe::egui;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

const UI: egui::Color32 = egui::Color32::from_rgb(52, 51, 52);
const SIDEBAR: egui::Color32 = egui::Color32::from_rgb(34, 37, 41);
const STATUS: egui::Color32 = egui::Color32::from_rgb(45, 48, 54);
const EDITOR: egui::Color32 = egui::Color32::from_rgb(49, 54, 64);
const GUTTER: egui::Color32 = egui::Color32::from_rgb(49, 54, 64);
const TAB_ACTIVE: egui::Color32 = egui::Color32::from_rgb(49, 54, 64);
const TAB_INACTIVE: egui::Color32 = egui::Color32::from_rgb(79, 83, 92);

#[derive(Clone, Copy)]
enum EditCommand {
    Undo,
    Redo,
    SelectAll,
}

type TreeIcon = String;

pub struct SublemonApp {
    documents: Vec<Document>,
    active_document: usize,
    sidebar_visible: bool,
    status_bar_visible: bool,
    word_wrap: bool,
    cursor_line: usize,
    cursor_column: usize,
    path_dialog: Option<PathDialog>,
    path_input: String,
    message: Option<String>,
    close_confirmation: Option<usize>,
    pending_edit_command: Option<EditCommand>,
    workspace_root: PathBuf,
    expanded_folders: HashSet<PathBuf>,
    tree_icon_textures: HashMap<String, egui::TextureHandle>,
}

#[derive(Clone, Copy)]
enum PathDialog {
    Open,
    SaveAs,
}

impl Default for SublemonApp {
    fn default() -> Self {
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut documents: Vec<Document> = ["README.md", "AGENTS.md"]
            .into_iter()
            .filter_map(|name| Document::open(&workspace_root.join(name)).ok())
            .collect();
        if documents.is_empty() {
            documents.push(Document::untitled());
        }
        let expanded_folders = HashSet::from([workspace_root.clone()]);
        Self {
            documents,
            active_document: 0,
            sidebar_visible: true,
            status_bar_visible: true,
            word_wrap: true,
            cursor_line: 1,
            cursor_column: 1,
            path_dialog: None,
            path_input: String::new(),
            message: None,
            close_confirmation: None,
            pending_edit_command: None,
            workspace_root,
            expanded_folders,
            tree_icon_textures: HashMap::new(),
        }
    }
}

impl SublemonApp {
    fn new_document(&mut self) {
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

    fn show_tabs(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new().fill(TAB_INACTIVE).show(ui, |ui| {
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
                                .fill(if active { TAB_ACTIVE } else { TAB_INACTIVE })
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
            tree_folder_icon(&name, path == self.workspace_root)
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
        if is_directory && self.expanded_folders.contains(path) {
            for child in self.workspace_entries(path) {
                self.show_folder_row(ui, &child, depth + 1);
            }
        }
    }

    fn workspace_entries(&self, folder: &Path) -> Vec<PathBuf> {
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
        entries
    }

    fn close_document(&mut self, index: usize) {
        if self.documents[index].is_dirty() {
            self.close_confirmation = Some(index);
            return;
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
        match Document::open(&path) {
            Ok(document) => {
                if let Some(existing) = self
                    .documents
                    .iter()
                    .position(|item| item.path == document.path)
                {
                    self.active_document = existing;
                } else {
                    self.documents.push(document);
                    self.active_document = self.documents.len() - 1;
                }
                self.message = None;
            }
            Err(error) => self.message = Some(format!("Could not open file: {error}")),
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
        apply_dark_editor_style(ui.ctx());

        egui::Panel::top("main_menu").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                ui.menu_button("File", |ui| {
                    if ui.button("New File").clicked() {
                        self.new_document();
                        ui.close();
                    }
                    if ui.button("Open File…").clicked() {
                        self.path_input.clear();
                        self.path_dialog = Some(PathDialog::Open);
                        ui.close();
                    }
                    ui.add_enabled(false, egui::Button::new("Open Folder…"));
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
                });

                ui.menu_button("Goto", |ui| {
                    ui.add_enabled(false, egui::Button::new("Goto Anything…"));
                    ui.add_enabled(false, egui::Button::new("Goto Line…"));
                    ui.add_enabled(false, egui::Button::new("Goto Symbol…"));
                });

                ui.menu_button("Tools", |ui| {
                    ui.add_enabled(false, egui::Button::new("Build"));
                    ui.add_enabled(false, egui::Button::new("Command Palette…"));
                });

                ui.menu_button("Project", |ui| {
                    ui.add_enabled(false, egui::Button::new("Open Project…"));
                    ui.add_enabled(false, egui::Button::new("Add Folder to Project…"));
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

        if self.status_bar_visible {
            egui::Panel::bottom("status_bar")
                .frame(egui::Frame::new().fill(STATUS))
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

        if self.sidebar_visible {
            let sidebar_default_size = (ui.ctx().content_rect().width() * 0.43).clamp(250.0, 550.0);
            egui::Panel::left("sidebar_layout_v2")
                .frame(egui::Frame::new().fill(SIDEBAR))
                .resizable(true)
                .default_size(sidebar_default_size)
                .min_size(0.0)
                .show(ui, |ui| {
                    ui.painter().rect_filled(ui.max_rect(), 0.0, SIDEBAR);
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.strong("FOLDERS");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let _ = ui.small_button("⋯");
                        });
                    });
                    ui.add_space(10.0);
                    self.show_folder_row(ui, &self.workspace_root.clone(), 0);
                });
        }

        egui::CentralPanel::default_margins()
            .frame(egui::Frame::new().fill(EDITOR).inner_margin(0))
            .show(ui, |ui| {
                self.show_tabs(ui);
                let gutter_width = 56.0;
                let content_width = ui.available_width();
                let content_height = ui.available_height();
                let editor_width =
                    (content_width - gutter_width - ui.spacing().item_spacing.x).max(1.0);
                let chars_per_visual_line =
                    ((editor_width - 16.0) / (14.0 * 0.60)).floor().max(1.0) as usize;
                let text_for_gutter = self.active_document().text.clone();
                ui.horizontal_top(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(gutter_width, content_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.painter().rect_filled(ui.max_rect(), 0.0, GUTTER);
                            ui.add_space(8.0);
                            egui::ScrollArea::vertical()
                                .id_salt("line_number_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for (index, line_text) in
                                        text_for_gutter.split('\n').enumerate()
                                    {
                                        let line = index + 1;
                                        let line_height = 19.0;
                                        if line == self.cursor_line {
                                            ui.painter().rect_filled(
                                                egui::Rect::from_min_size(
                                                    ui.cursor().min,
                                                    egui::vec2(ui.available_width(), line_height),
                                                ),
                                                0.0,
                                                egui::Color32::from_rgb(70, 80, 94),
                                            );
                                        }
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Min),
                                            |ui| {
                                                ui.add_sized(
                                                    [42.0, line_height],
                                                    egui::Label::new(
                                                        egui::RichText::new(line.to_string())
                                                            .monospace()
                                                            .size(12.0)
                                                            .color(egui::Color32::from_rgb(
                                                                118, 133, 153,
                                                            )),
                                                    ),
                                                );
                                            },
                                        );
                                        let wrapped_rows = if self.word_wrap {
                                            line_text
                                                .chars()
                                                .count()
                                                .max(1)
                                                .div_ceil(chars_per_visual_line)
                                        } else {
                                            1
                                        };
                                        for _ in 1..wrapped_rows {
                                            ui.add_space(line_height);
                                        }
                                    }
                                });
                        },
                    );
                    ui.allocate_ui_with_layout(
                        egui::vec2(editor_width, content_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            let document_index = self.active_document;
                            let document = &mut self.documents[document_index];
                            let editor_id = egui::Id::new(("sublemon_editor", document.id));
                            if egui::TextEdit::load_state(ui.ctx(), editor_id).is_none() {
                                let mut state = egui::TextEdit::load_state(ui.ctx(), editor_id)
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
                                state
                                    .cursor
                                    .set_char_range(Some(egui::text::CCursorRange::one(
                                        egui::text::CCursor::new(
                                            caret.min(document.text.chars().count()),
                                        ),
                                    )));
                                state.store(ui.ctx(), editor_id);
                            }
                            if prepare_enter_indentation(ui.ctx(), editor_id, &mut document.text) {
                                if let Some(mut state) =
                                    egui::TextEdit::load_state(ui.ctx(), editor_id)
                                {
                                    let end = document.text.chars().count();
                                    state.cursor.set_char_range(Some(
                                        egui::text::CCursorRange::one(egui::text::CCursor::new(
                                            end,
                                        )),
                                    ));
                                    state.store(ui.ctx(), editor_id);
                                }
                            }
                            let is_markdown = document
                                .path
                                .as_ref()
                                .and_then(|path| path.extension())
                                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"));
                            let mut layouter =
                                |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, wrap_width: f32| {
                                    let mut job = if is_markdown {
                                        markdown_layout(buffer.as_str(), wrap_width)
                                    } else {
                                        egui::text::LayoutJob::simple(
                                            buffer.as_str().to_owned(),
                                            egui::FontId::monospace(14.0),
                                            egui::Color32::from_rgb(204, 205, 207),
                                            wrap_width,
                                        )
                                    };
                                    job.wrap.max_width = wrap_width;
                                    ui.fonts_mut(|fonts| fonts.layout_job(job))
                                };
                            let editor = egui::TextEdit::multiline(&mut document.text)
                                .id(editor_id)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(if self.word_wrap {
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
                            let mut cursor_range = output.cursor_range;
                            if let Some(command) = self.pending_edit_command.take() {
                                let current_range = cursor_range.unwrap_or_else(|| {
                                    egui::text::CCursorRange::one(egui::text::CCursor::new(
                                        document.text.chars().count(),
                                    ))
                                });
                                let current = (current_range, document.text.clone());
                                let mut undoer = output.state.undoer();
                                let replacement = match command {
                                    EditCommand::Undo => undoer.undo(&current).cloned(),
                                    EditCommand::Redo => undoer.redo(&current).cloned(),
                                    EditCommand::SelectAll => Some((
                                        egui::text::CCursorRange::two(
                                            egui::text::CCursor::new(0),
                                            egui::text::CCursor::new(document.text.chars().count()),
                                        ),
                                        document.text.clone(),
                                    )),
                                };
                                if let Some((range, text)) = replacement {
                                    document.text = text;
                                    cursor_range = Some(range);
                                    let mut state = output.state;
                                    state.cursor.set_char_range(Some(range));
                                    state.store(ui.ctx(), output.response.id);
                                }
                            }
                            if let Some(cursor_range) = cursor_range {
                                let cursor_index: usize = cursor_range.primary.index.into();
                                let prefix: String =
                                    document.text.chars().take(cursor_index).collect();
                                self.cursor_line = document_line_count(&prefix);
                                self.cursor_column = prefix
                                    .rsplit('\n')
                                    .next()
                                    .map_or(1, |line| line.chars().count() + 1);
                            }
                        },
                    );
                });
            });

        if let Some(dialog) = self.path_dialog {
            let title = match dialog {
                PathDialog::Open => "Open File",
                PathDialog::SaveAs => "Save As",
            };
            let mut open = true;
            let mut submit = false;
            let mut cancel = false;
            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.label("Enter a file path:");
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
                        PathDialog::Open => self.open_path(path),
                        PathDialog::SaveAs => self.save_active_as(path),
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

fn markdown_layout(text: &str, wrap_width: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let body_color = egui::Color32::from_rgb(205, 206, 210);
    let heading_color = egui::Color32::from_rgb(179, 199, 226);
    let marker_color = egui::Color32::from_rgb(230, 116, 72);
    let bold_color = egui::Color32::from_rgb(190, 198, 224);

    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let hashes = trimmed
            .chars()
            .take_while(|character| *character == '#')
            .count();
        if hashes > 0 {
            let leading = line.len() - trimmed.len();
            let marker_end = (leading
                + hashes
                + trimmed[hashes..]
                    .chars()
                    .take_while(|character| *character == ' ')
                    .count())
            .min(line.len());
            job.append(
                &line[..marker_end],
                0.0,
                egui::TextFormat::simple(egui::FontId::monospace(14.0), marker_color),
            );
            append_markdown_body(&mut job, &line[marker_end..], heading_color, bold_color);
        } else {
            append_markdown_body(&mut job, line, body_color, bold_color);
        }
    }
    job
}

fn append_markdown_body(
    job: &mut egui::text::LayoutJob,
    text: &str,
    body_color: egui::Color32,
    bold_color: egui::Color32,
) {
    let normal = egui::TextFormat::simple(egui::FontId::monospace(14.0), body_color);
    let bold = egui::TextFormat::simple(egui::FontId::monospace(14.0), bold_color);
    let marker = egui::TextFormat::simple(
        egui::FontId::monospace(14.0),
        egui::Color32::from_rgb(205, 135, 173),
    );
    let mut remaining = text;
    while let Some(start) = remaining.find("**") {
        job.append(&remaining[..start], 0.0, normal.clone());
        let after_start = &remaining[start + 2..];
        let Some(end) = after_start.find("**") else {
            job.append(&remaining[start..], 0.0, normal);
            return;
        };
        job.append("**", 0.0, marker.clone());
        job.append(&after_start[..end], 0.0, bold.clone());
        job.append("**", 0.0, marker.clone());
        remaining = &after_start[end + 2..];
    }
    job.append(remaining, 0.0, normal);
}

fn apply_dark_editor_style(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = UI;
    visuals.window_fill = UI;
    visuals.extreme_bg_color = EDITOR;
    visuals.code_bg_color = EDITOR;
    visuals.faint_bg_color = egui::Color32::from_rgb(57, 62, 73);
    visuals.selection.bg_fill = egui::Color32::from_rgb(70, 80, 94);
    visuals.selection.stroke.color = egui::Color32::from_rgb(173, 186, 207);
    visuals.widgets.noninteractive.bg_fill = UI;
    visuals.widgets.inactive.bg_fill = UI;
    visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(205, 205, 198);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(66, 67, 60);
    visuals.widgets.hovered.fg_stroke.color = egui::Color32::WHITE;
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(81, 81, 72);
    visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;
    context.set_visuals(visuals);

    context.style_mut_of(context.theme(), |style| {
        style.spacing.item_spacing = egui::vec2(4.0, 3.0);
        style.spacing.button_padding = egui::vec2(5.0, 3.0);
        style.visuals.dark_mode = true;
    });
}

fn document_line_count(text: &str) -> usize {
    text.bytes().filter(|byte| *byte == b'\n').count() + 1
}
