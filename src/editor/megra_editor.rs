/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MegraEditor {
    content: String,
}

impl Default for MegraEditor {
    fn default() -> Self {
        Self {
            content: "(sx 'ga #t (infer 'troll :events 'a (saw 400) :rules (rule 'a 'a 100 400)))".to_owned(),
        }
    }
}

impl egui::app::App for MegraEditor {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn ui(
        &mut self,
        ctx: &std::sync::Arc<egui::Context>,
        integration_context: &mut egui::app::IntegrationContext,
    ) {
        let MegraEditor { content } = self;

        // Example used in `README.md`.
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mégra Editor");

	    let tx = egui::TextEdit::multiline(content).desired_rows(31).desired_width(640.0);
	    
            ui.horizontal(|ui| {                
		ui.add(tx)
            });
                        
        });

        integration_context.output.window_size = Some(egui::Vec2::new(640.0, 480.0)); // resize the window to be just the size we need it to be
    }

    fn on_exit(&mut self, storage: &mut dyn egui::app::Storage) {
        egui::app::set_value(storage, egui::app::APP_KEY, self);
    }
}
