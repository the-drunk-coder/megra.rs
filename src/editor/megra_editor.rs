use std::sync::*;
use parking_lot::Mutex;

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

impl egui::app::App for MegraEditor {
    fn name(&self) -> &str {
        "Mégra Editor"
    }

    fn load(&mut self, storage: &dyn egui::app::Storage) {
	println!("load");
        *self = egui::app::get_value(storage, egui::app::APP_KEY).unwrap_or_default()
    }

    fn save(&mut self, storage: &mut dyn egui::app::Storage) {
	println!("save");
        egui::app::set_value(storage, egui::app::APP_KEY, self);
    }
        
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn ui(
        &mut self,
        ctx: &egui::CtxRef,
        integration_context: &mut egui::app::IntegrationContext,
    ) {
       	
        // Example used in `README.md`.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mégra Editor");

	    let tx = if let Some(cb) = self.callback.as_ref() {
		egui::CallbackTextEdit::multiline(&mut self.content).desired_rows(31).desired_width(640.0).eval_callback(&cb)		
	    } else {
		egui::CallbackTextEdit::multiline(&mut self.content).desired_rows(31).desired_width(640.0)
	    };
	    	    
	    ui.horizontal(|ui| {                
		ui.add(tx)
            });
            
        });
	
	integration_context.output.window_size = Some(egui::Vec2::new(640.0, 480.0)); // resize the window to be just the size we need it to be
    }    
}
