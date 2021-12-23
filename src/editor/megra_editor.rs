use chrono::*;
use directories_next::ProjectDirs;
use egui::ScrollArea;
use parking_lot::Mutex;
use std::{collections::HashMap, fs, path, sync::*};

#[derive(PartialEq)]
enum SketchNumber {
    Num(usize),
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MegraEditor<'a> {
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
    function_names: Vec<&'a str>,
    //#[serde(skip)]
    //colors: HashMap<egui::CodeColors, egui::Color32>,
}

impl<'a> Default for MegraEditor<'a> {
    fn default() -> Self {
        Self {
            content: "(sx 'ga #t (infer 'troll :events 'a (saw 400) :rules (rule 'a 'a 100 400)))"
                .to_owned(),
            callback: None,
            sketch_list: Vec::new(),
            current_sketch: "".to_string(),
            sketch_number: 0,
            function_names: Vec::new(),
            //colors: HashMap::new(),
        }
    }
}

impl<'a> MegraEditor<'a> {
    pub fn set_callback(&mut self, callback: &Arc<Mutex<dyn FnMut(&String)>>) {
        self.callback = Some(Arc::clone(callback));
    }
}

impl<'a> epi::App for MegraEditor<'a> {
    fn name(&self) -> &str {
        "Mégra Editor"
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(5)
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        // make sure callback is carried over after loading
        let callback = self
            .callback
            .as_ref()
            .map(|tmp_callback| Arc::clone(&tmp_callback));

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

        self.function_names.push("apple");
        self.function_names.push("export-dot");
        self.function_names.push("step-part");
        self.function_names.push("friendship");
        self.function_names.push("tmod");
        self.function_names.push("latency");
        self.function_names.push("global-resources");
        self.function_names.push("learn");
        self.function_names.push("delay");
        self.function_names.push("reverb");
        self.function_names.push("pear");
        self.function_names.push("nuc");
        self.function_names.push("fully");
        self.function_names.push("flower");
        self.function_names.push("sx");
        self.function_names.push("cyc");
        self.function_names.push("xspread");
        self.function_names.push("xdup");
        self.function_names.push("life");
        self.function_names.push("ls");
        self.function_names.push("every");
        self.function_names.push("defpart");
        self.function_names.push("infer");
        self.function_names.push("clear");
        self.function_names.push("once");
        self.function_names.push("cub");
        self.function_names.push("cmp");
        self.function_names.push("chop");
        self.function_names.push("rnd");
        self.function_names.push("rep");
        self.function_names.push("inh");
        self.function_names.push("exh");
        self.function_names.push("inexh");
        self.function_names.push("stages");

	/*
        self.colors.insert(
            egui::CodeColors::Keyword,
            egui::Color32::from_rgb(200, 20, 200),
        );
        self.colors.insert(
            egui::CodeColors::Function,
            egui::Color32::from_rgb(220, 20, 100),
        );
        self.colors
            .insert(egui::CodeColors::Comment, egui::Color32::from_gray(128));
        self.colors.insert(
            egui::CodeColors::Boolean,
            egui::Color32::from_rgb(0, 200, 100),
        );
        self.colors.insert(
            egui::CodeColors::String,
            egui::Color32::from_rgb(200, 200, 10),
        );*/

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
    fn update(&mut self, ctx: &egui::CtxRef, _: &mut epi::Frame<'_>) {
        // some frame options ...
        let mut frame = egui::Frame::none();
        frame.fill = egui::Color32::BLACK;
        frame.margin = egui::Vec2::new(3.0, 3.0);
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let mut sketch_number = SketchNumber::Num(self.sketch_number);

            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new("Mégra Editor")
                        .text_color(egui::Color32::from_rgb(150, 250, 100))
                        .wrap(false)
                        .monospace(),
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

            let mut sketch_switched = false;
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
                    Ok(s) => self.content = s,
                    Err(e) => {
                        println!("couldn't read sketch {}", e);
                    }
                }
                sketch_switched = true;
            }

            ui.separator();

            ScrollArea::vertical()
                .always_show_scroll(true)
                .show(ui, |ui| {
                    let num_lines = self.content.lines().count() + 1;

                    let tx = if let Some(cb) = self.callback.as_ref() {
                        egui::TextEdit::multiline(
                            &mut self.content,                            
                        )
                        .desired_rows(22)
                        //.reset_cursor(sketch_switched)
                        .text_style(egui::TextStyle::Monospace)
                        .desired_width(800.0)
                        //.eval_callback(&cb)
                    } else {
                        egui::TextEdit::multiline(
                            &mut self.content,                            
                        )
                        .desired_rows(22)
                        //.reset_cursor(sketch_switched)
                        .desired_width(800.0)
                        .text_style(egui::TextStyle::Monospace)
                    };

                    let mut linenums = "".to_owned();
                    for i in 1..num_lines {
                        linenums.push_str(format!("{}\n", i).as_str());
                    }

                    let ln = egui::Label::new(linenums).text_style(egui::TextStyle::Monospace);

                    ui.horizontal(|ui| {
                        ui.add(ln);
                        ui.add(tx);
                    });
                });
        });
    }
}
