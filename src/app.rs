use crate::document::Document;
use eframe::egui;
use std::path::PathBuf;

const UI: egui::Color32 = egui::Color32::from_rgb(45, 45, 45);
const EDITOR: egui::Color32 = egui::Color32::from_rgb(39, 40, 34);
const GUTTER: egui::Color32 = egui::Color32::from_rgb(39, 40, 34);
const TAB_ACTIVE: egui::Color32 = egui::Color32::from_rgb(57, 58, 51);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(230, 190, 90);

pub struct SublemonApp {
    documents: Vec<Document>,
    active_document: usize,
    sidebar_visible: bool,
    minimap_visible: bool,
    status_bar_visible: bool,
    word_wrap: bool,
    cursor_line: usize,
    cursor_column: usize,
    path_dialog: Option<PathDialog>,
    path_input: String,
    message: Option<String>,
    close_confirmation: Option<usize>,
}

#[derive(Clone, Copy)]
enum PathDialog {
    Open,
    SaveAs,
}

impl Default for SublemonApp {
    fn default() -> Self {
        Self {
            documents: vec![Document::untitled()],
            active_document: 0,
            sidebar_visible: true,
            minimap_visible: true,
            status_bar_visible: true,
            word_wrap: false,
            cursor_line: 1,
            cursor_column: 1,
            path_dialog: None,
            path_input: String::new(),
            message: None,
            close_confirmation: None,
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
                    ui.add_enabled(false, egui::Button::new("Undo"));
                    ui.add_enabled(false, egui::Button::new("Redo"));
                    ui.separator();
                    ui.add_enabled(false, egui::Button::new("Cut"));
                    ui.add_enabled(false, egui::Button::new("Copy"));
                    ui.add_enabled(false, egui::Button::new("Paste"));
                });

                ui.menu_button("Selection", |ui| {
                    ui.add_enabled(false, egui::Button::new("Select All"));
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
                    ui.checkbox(&mut self.minimap_visible, "Show Minimap");
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

        egui::Panel::top("tabs").show(ui, |ui| {
            ui.add_space(3.0);
            egui::ScrollArea::horizontal()
                .id_salt("document_tabs")
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let mut close_index = None;
                        for (index, document) in self.documents.iter().enumerate() {
                            let active = index == self.active_document;
                            let title = document.title(index + 1);
                            let title = if document.is_dirty() {
                                format!("● {title}")
                            } else {
                                title
                            };
                            let tab = egui::Frame::new()
                                .fill(if active { TAB_ACTIVE } else { UI })
                                .inner_margin(egui::Margin::symmetric(10, 5));
                            tab.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if ui.selectable_label(active, title).clicked() {
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
                    });
                });
        });

        if self.status_bar_visible {
            egui::Panel::bottom("status_bar").show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Plain Text");
                    ui.separator();
                    ui.label(format!(
                        "Line {}, Column {}",
                        self.cursor_line, self.cursor_column
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("UTF-8");
                        ui.separator();
                        ui.label(self.active_document().line_ending.label());
                        ui.separator();
                        ui.label("Tab Size: 4");
                    });
                });
            });
        }

        if self.sidebar_visible {
            egui::Panel::left("sidebar")
                .resizable(true)
                .default_size(210.0)
                .show(ui, |ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.strong("OPEN FILES");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let _ = ui.small_button("⋯");
                        });
                    });
                    ui.separator();
                    for (index, document) in self.documents.iter().enumerate() {
                        let selected = index == self.active_document;
                        let title = document.title(index + 1);
                        let title = if document.is_dirty() {
                            format!("● {title}")
                        } else {
                            title
                        };
                        if ui
                            .selectable_label(selected, format!("▤  {title}"))
                            .clicked()
                        {
                            self.active_document = index;
                        }
                    }
                });
        }

        egui::Panel::left("line_numbers")
            .resizable(false)
            .default_size(46.0)
            .show(ui, |ui| {
                ui.painter().rect_filled(ui.max_rect(), 0.0, GUTTER);
                let line_count = document_line_count(&self.active_document().text);
                egui::ScrollArea::vertical()
                    .id_salt("line_number_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(8.0);
                        for line in 1..=line_count {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                ui.add_sized(
                                    [32.0, 19.0],
                                    egui::Label::new(
                                        egui::RichText::new(line.to_string())
                                            .monospace()
                                            .size(12.0)
                                            .color(egui::Color32::from_rgb(117, 113, 94)),
                                    ),
                                );
                            });
                        }
                    });
            });

        if self.minimap_visible {
            egui::Panel::right("minimap")
                .resizable(false)
                .default_size(104.0)
                .show(ui, |ui| {
                    ui.painter().rect_filled(
                        ui.max_rect(),
                        0.0,
                        egui::Color32::from_rgb(39, 40, 34),
                    );
                    let text = self.active_document().text.clone();
                    ui.add_space(5.0);
                    egui::ScrollArea::vertical()
                        .id_salt("minimap_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(text)
                                        .monospace()
                                        .size(5.5)
                                        .color(egui::Color32::from_rgb(155, 151, 127)),
                                )
                                .wrap_mode(egui::TextWrapMode::Extend),
                            );
                        });
                });
        }

        egui::CentralPanel::default_margins()
            .frame(egui::Frame::new().fill(EDITOR).inner_margin(0))
            .show(ui, |ui| {
                let document = &mut self.documents[self.active_document];
                let mut editor = egui::TextEdit::multiline(&mut document.text)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(40)
                    .frame(egui::Frame::NONE)
                    .margin(egui::Margin::symmetric(8, 8))
                    .interactive(!document.read_only);
                if !self.word_wrap {
                    editor = editor.code_editor();
                }
                let output = editor.show(ui);
                if let Some(cursor_range) = output.cursor_range {
                    let cursor_index: usize = cursor_range.primary.index.into();
                    let prefix: String = document.text.chars().take(cursor_index).collect();
                    self.cursor_line = document_line_count(&prefix);
                    self.cursor_column = prefix
                        .rsplit('\n')
                        .next()
                        .map_or(1, |line| line.chars().count() + 1);
                }
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

fn apply_dark_editor_style(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = UI;
    visuals.window_fill = UI;
    visuals.extreme_bg_color = EDITOR;
    visuals.code_bg_color = EDITOR;
    visuals.faint_bg_color = egui::Color32::from_rgb(52, 53, 47);
    visuals.selection.bg_fill = egui::Color32::from_rgb(73, 72, 62);
    visuals.selection.stroke.color = ACCENT;
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
