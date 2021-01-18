use std::sync::*;
use parking_lot::Mutex;
use egui::ScrollArea;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MegraEditor {
    content: String,
    #[serde(skip)]
    callback: Option<Arc<Mutex<dyn FnMut(&String)>>>,    
}

impl Default for MegraEditor {
    fn default() -> Self {
        Self {
            content: "(sx 'ga #t (infer 'troll :events 'a (saw 400) :rules (rule 'a 'a 100 400)))".to_owned(),
	    callback: None,	    
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
	_: &mut epi::Frame<'_>) {

	// some frame options ...
       	let mut frame = egui::Frame::none();
	frame.fill = egui::Color32::BLACK;
	frame.margin = egui::Vec2::new(3.0, 3.0);
	
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
	    ui.add(egui::Label::new("Mégra Editor").text_color(egui::Color32::from_rgb(150, 250, 100)).monospace());
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
