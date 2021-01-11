use std::sync::*;
use parking_lot::Mutex;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MegraEditor {
    content: String,
    #[serde(skip)]
    callback: Option<Arc<Mutex<dyn FnMut(&String)>>>,
    #[serde(skip)]
    selection_toggle: atomic::AtomicBool,
}

impl Default for MegraEditor {
    fn default() -> Self {
        Self {
            content: "(sx 'ga #t (infer 'troll :events 'a (saw 400) :rules (rule 'a 'a 100 400)))".to_owned(),
	    callback: None,
	    selection_toggle: atomic::AtomicBool::new(false),
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
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }
        
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(
        &mut self,
	ctx: &egui::CtxRef,
	frame: &mut epi::Frame<'_>
    ) {
       	
        // Example used in `README.md`.
        egui::CentralPanel::default().show(ctx, |ui| {
            
	    let tx = if let Some(cb) = self.callback.as_ref() {		
		egui::CallbackTextEdit::multiline(&mut self.content, &mut self.selection_toggle)
		    .desired_rows(34)
		    .text_style(egui::TextStyle::Monospace)
		    .desired_width(640.0)
		    .eval_callback(&cb)		
	    } else {
		egui::CallbackTextEdit::multiline(&mut self.content, &mut self.selection_toggle)
		    .desired_rows(34)
		    .desired_width(640.0)
		    .text_style(egui::TextStyle::Monospace)
	    };

	    ui.add(egui::Label::new("Mégra Editor").text_color(egui::Color32::from_rgb(150, 250, 100)).monospace());
	    ui.horizontal(|ui| {
	
		ui.add(tx)
            });
            
        });
	
	//integration_context.output.window_size = Some(egui::Vec2::new(640.0, 480.0)); // resize the window to be just the size we need it to be
    }    
}
