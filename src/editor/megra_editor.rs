use chrono::*;
use directories_next::ProjectDirs;
use egui::ScrollArea;
use parking_lot::Mutex;
use std::{fs, path, sync::*};

use egui::style::Margin;
use egui::FontId;
use epaint::text::{FontData, FontDefinitions, FontFamily};
// custom text edit window
use crate::editor::livecode_text_edit::LivecodeTextEdit;
use crate::editor::syntax_highlighting::*;

#[derive(PartialEq)]
enum SketchNumber {
    Num(usize),
}

pub enum EditorFont {
    ComicMono,
    Mononoki,
    Custom(String),
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MegraEditor {
    content: String,
    #[serde(skip)]
    callback: Option<Arc<Mutex<dyn FnMut(&String)>>>,
    #[serde(skip)]
    sketch_list: Vec<String>,
    #[serde(skip)]
    current_sketch: String,
    #[serde(skip)]
    sketch_number: usize,
    #[serde(skip)]
    font: Option<EditorFont>,
}

impl Default for MegraEditor {
    fn default() -> Self {
        Self {
            content: "(sx 'ga #t (infer 'troll :events 'a (saw 400) :rules (rule 'a 'a 100 400)))"
                .to_owned(),
            callback: None,
            sketch_list: Vec::new(),
            current_sketch: "".to_string(),
            sketch_number: 0,
            font: None,
        }
    }
}

impl MegraEditor {
    pub fn set_font(&mut self, font: EditorFont) {
        self.font = Some(font);
    }

    pub fn set_callback(&mut self, callback: &Arc<Mutex<dyn FnMut(&String)>>) {
        self.callback = Some(Arc::clone(callback));
    }
}

impl epi::App for MegraEditor {
    fn name(&self) -> &str {
        "Mégra Editor"
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(5)
    }

    fn setup(
        &mut self,
        ctx: &egui::Context,
        _frame: &epi::Frame,
        storage: Option<&dyn epi::Storage>,
    ) {
        let mut fonts = FontDefinitions::default();

        // Two built-in options ...
        fonts.font_data.insert(
            "mononoki".to_owned(),
            FontData::from_static(include_bytes!("../../fonts/mononoki-Bold.ttf")),
        );

        fonts.font_data.insert(
            "ComicMono".to_owned(),
            FontData::from_static(include_bytes!("../../fonts/ComicMono.ttf")),
        );

        match &self.font {
            Some(EditorFont::ComicMono) => {
                fonts
                    .families
                    .get_mut(&FontFamily::Monospace)
                    .unwrap()
                    .insert(0, "ComicMono".to_owned());
            }
            Some(EditorFont::Mononoki) => {
                fonts
                    .families
                    .get_mut(&FontFamily::Monospace)
                    .unwrap()
                    .insert(0, "ComicMono".to_owned());
            }
            Some(EditorFont::Custom(path)) => match fs::read(path) {
                Ok(font_data) => {
                    fonts
                        .font_data
                        .insert("custom_font".to_owned(), FontData::from_owned(font_data));
                    fonts
                        .families
                        .get_mut(&FontFamily::Monospace)
                        .unwrap()
                        .insert(0, "custom_font".to_owned());
                }
                Err(_) => {
                    println!("couldn't read custom font, using default");
                    fonts
                        .families
                        .get_mut(&FontFamily::Monospace)
                        .unwrap()
                        .insert(0, "ComicMono".to_owned());
                }
            },
            None => {
                fonts
                    .families
                    .get_mut(&FontFamily::Monospace)
                    .unwrap()
                    .insert(0, "mononoki".to_owned());
            }
        }

        ctx.set_fonts(fonts);

        // make sure callback is carried over after loading
        let callback = self
            .callback
            .as_ref()
            .map(|tmp_callback| Arc::clone(tmp_callback));

        if let Some(s) = storage {
            *self = epi::get_value(s, epi::APP_KEY).unwrap_or_default();
        }

        if let Some(tmp_callback) = callback {
            self.set_callback(&tmp_callback);
        }

        self.content = format!(
            ";; Created {}",
            Local::now().format("%A, %F, %H:%M:%S ... good luck!")
        );

        // create sketch and load sketch file list ...
        if let Some(proj_dirs) = ProjectDirs::from("de", "parkellipsen", "megra") {
            let sketchbook_path = proj_dirs.config_dir().join("sketchbook");
            if sketchbook_path.exists() {
                // prepare sketch marked with date
                let id = format!("sketch_{}.megra3", Local::now().format("%Y%m%d_%H%M_%S"));
                let file_path = sketchbook_path.join(id);
                self.current_sketch = file_path.to_str().unwrap().to_string();
                // push current sketch so it'll be the one visible
                self.sketch_list.push(self.current_sketch.clone());

                if let Ok(entries) = fs::read_dir(sketchbook_path) {
                    let mut disk_sketches = Vec::new();
                    for entry in entries.flatten() {
                        let path = entry.path();
                        // only consider files here ...
                        if path.is_file() {
                            if let Some(ext) = path.extension() {
                                if ext == "megra3" {
                                    disk_sketches.push(path.to_str().unwrap().to_string());
                                }
                            }
                        }
                    }

                    disk_sketches.sort();
                    // sort sketch list so it's easier to find the sketches
                    self.sketch_list.append(&mut disk_sketches);
                }
            }
        }
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        if !self.current_sketch.is_empty() {
            let p = path::Path::new(&self.current_sketch);
            match fs::write(p, &self.content.as_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    println!("couldn't save sketch {}", e);
                }
            }
        }

        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _: &epi::Frame) {
        // some frame options ...
        let mut frame = egui::Frame::none();
        frame.fill = egui::Color32::BLACK;
        frame.margin = Margin::symmetric(3.0, 3.0);
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let mut sketch_number = SketchNumber::Num(self.sketch_number);

            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new("Mégra Editor").font(FontId::monospace(13.0)),
                    )
                    .wrap(false),
                );

                let id = ui.make_persistent_id("file_chooser_box");
                egui::ComboBox::from_id_source(id)
                    .selected_text(&self.sketch_list[self.sketch_number])
                    .show_ui(ui, |ui| {
                        for i in 0..self.sketch_list.len() {
                            ui.selectable_value(
                                &mut sketch_number,
                                SketchNumber::Num(i),
                                &self.sketch_list[i],
                            );
                        }
                    });
            });

            let SketchNumber::Num(sk_num) = sketch_number;

            //let mut sketch_switched = false;
            if sk_num != self.sketch_number {
                println!("switch sketch from {} to {}", self.sketch_number, sk_num);
                self.sketch_number = sk_num;

                // store content explicitly when changing ...
                if !self.current_sketch.is_empty() {
                    let p = path::Path::new(&self.current_sketch);
                    match fs::write(p, &self.content.as_bytes()) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("couldn't save sketch {}", e);
                        }
                    }
                }

                self.current_sketch = self.sketch_list[sk_num].clone();
                let p = path::Path::new(&self.current_sketch);
                match fs::read_to_string(p) {
                    Ok(mut s) => {
                        if !s.ends_with('\n') {
                            s.push('\n');
                        }
                        self.content = s
                    }
                    Err(e) => {
                        println!("couldn't read sketch {}", e);
                    }
                }
            }

            ui.separator();

            ScrollArea::vertical()
                .always_show_scroll(true)
                .show(ui, |ui| {
                    let num_lines = self.content.lines().count() + 1;

                    let theme = CodeTheme::from_memory(ui.ctx());
                    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                        let layout_job = highlight(ui.ctx(), &theme, string);
                        ui.fonts().layout_job(layout_job)
                    };

                    let tx = if let Some(cb) = self.callback.as_ref() {
                        LivecodeTextEdit::multiline(&mut self.content)
                            .desired_rows(30)
                            //.reset_cursor(sketch_switched)
                            .code_editor()
                            .desired_width(800.0)
                            .eval_callback(cb)
                            .layouter(&mut layouter)
                    } else {
                        LivecodeTextEdit::multiline(&mut self.content)
                            .desired_rows(30)
                            .code_editor()
                            //.reset_cursor(!sketch_switched)
                            .desired_width(800.0)
                            .layouter(&mut layouter)
                    };

                    let mut linenums = "".to_owned();
                    for i in 1..num_lines {
                        linenums.push_str(format!("{}\n", i).as_str());
                    }

                    let ln = egui::Label::new(
                        egui::RichText::new(linenums).font(FontId::monospace(13.0)),
                    );

                    ui.horizontal(|ui| {
                        ui.add(ln);
                        ui.add(tx);
                    });
                });
        });
    }
}
