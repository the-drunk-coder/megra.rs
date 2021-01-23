use std::{fs, path, sync::*};
use parking_lot::Mutex;
use egui::ScrollArea;
use directories_next::ProjectDirs;
use chrono::*;

#[derive(PartialEq)]
enum SketchNumber {
    Num(usize)
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
}

impl Default for MegraEditor {
    fn default() -> Self {
        Self {
            content: "(sx 'ga #t (infer 'troll :events 'a (saw 400) :rules (rule 'a 'a 100 400)))".to_owned(),
	    callback: None,
	    sketch_list: Vec::new(),
	    current_sketch: "".to_string(),
	    sketch_number: 0,		
        }
    }
}

impl MegraEditor {
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

    fn load(&mut self, storage: &dyn epi::Storage) {
	// make sure callback is carried over after loading
	let callback = if let Some(tmp_callback) = &self.callback {
	    Some(Arc::clone(&tmp_callback))
	} else {
	    None
	};
	   
        *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();

	if let Some(tmp_callback) = callback {
	    self.set_callback(&tmp_callback);
	}

	self.content = format!(";; Created {}", Local::now().format("%A, %F, %H:%M:%S ... good luck!"));
	
	// create sketch and load sketch file list ...
	if let Some(proj_dirs) = ProjectDirs::from("de", "parkellipsen", "megra") {
	    let sketchbook_path = proj_dirs.config_dir().join("sketchbook");
	    if sketchbook_path.exists() {
		// create sketch ... 	
		let id = format!("sketch_{}.megra3", Local::now().format("%Y%m%d_%H%M_%S"));
		let file_path = sketchbook_path.join(id);

		self.current_sketch = file_path.to_str().unwrap().to_string();		
		self.sketch_list.push(self.current_sketch.clone());
		
		if let Ok(entries) = fs::read_dir(sketchbook_path) {
		    for entry in entries {
			if let Ok(entry) = entry {
			    let path = entry.path();
			    // only consider files here ...
			    if path.is_file() {
				if let Some(ext) = path.extension() {
				    if ext == "megra3" {
					self.sketch_list.push(path.to_str().unwrap().to_string());					
				    }
				}
			    }
			}	    
		    }
		}
	    }
	}
    }
    
    fn save(&mut self, storage: &mut dyn epi::Storage) {
	if self.current_sketch != "" {
	    let p = path::Path::new(&self.current_sketch);
	    match fs::write(p, &self.content.as_bytes()) {
		Ok(_) => {},
		Err(e) => {println!("couldn't save sketch {}", e);}		   
	    }			    
	}
	
	epi::set_value(storage, epi::APP_KEY, self);
    }
            
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(
        &mut self,
	ctx: &egui::CtxRef,
	_: &mut epi::Frame<'_>) {

	// some frame options ...
       	let mut frame = egui::Frame::none();
	frame.fill = egui::Color32::BLACK;
	frame.margin = egui::Vec2::new(3.0, 3.0);
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {

	    let mut sketch_number = SketchNumber::Num(self.sketch_number);
	    
	    ui.horizontal(|ui| {
		ui.add(egui::Label::new("Mégra Editor").text_color(egui::Color32::from_rgb(150, 250, 100)).monospace());
				
		let id = ui.make_persistent_id("file_chooser_box");
		egui::combo_box(ui, id, &self.sketch_list[self.sketch_number], |ui| {
		    for i in 0..self.sketch_list.len() {
			ui.selectable_value(&mut sketch_number, SketchNumber::Num(i), &self.sketch_list[i]);
		    }		    		    
		});
	    });

	    let sk_num = match sketch_number {
		SketchNumber::Num(i) => i
	    };

	    if sk_num != self.sketch_number {
		println!("switch sketch from {} to {}", self.sketch_number, sk_num);
		self.sketch_number = sk_num;

		// store content explicitly when changing ...
		if self.current_sketch != "" {
		    let p = path::Path::new(&self.current_sketch);
		    match fs::write(p, &self.content.as_bytes()) {
			Ok(_) => {},
			Err(e) => {println!("couldn't save sketch {}", e);}		   
		    }			    
		}

		self.current_sketch = self.sketch_list[sk_num].clone();
		let p = path::Path::new(&self.current_sketch);
		match fs::read_to_string(p) {
		    Ok(s) => {self.content = s},
		    Err(e) => {println!("couldn't read sketch {}", e);}		   
		}	
	    }
	    	    
	    ui.separator();
	    ScrollArea::auto_sized().show(ui, |ui| {    
		let tx = if let Some(cb) = self.callback.as_ref() {		
		    egui::CallbackTextEdit::multiline(&mut self.content)
			.desired_rows(20)
			.text_style(egui::TextStyle::Monospace)
			.desired_width(800.0)
			.eval_callback(&cb)		
		} else {
		    egui::CallbackTextEdit::multiline(&mut self.content)
			.desired_rows(20)
			.desired_width(800.0)
			.text_style(egui::TextStyle::Monospace)
		};						    

		ui.add(tx);
            });
	});	
    }    
}
