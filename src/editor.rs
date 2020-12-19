use std::sync::*;
use parking_lot::Mutex;
use crate::session::OutputMode;
use ruffbox_synth::ruffbox::Ruffbox;

use crate::builtin_types::*;
mod megra_editor;
use megra_editor::MegraEditor;
use crate::parser;

pub fn run_editor<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>, mode: OutputMode) {

    // Persist app state to file:
    let storage = egui_glium::storage::FileStorage::from_path(".megra_edit.json");
    
    // Restore editor from file, or create new editor:
    let mut app: MegraEditor = egui::app::get_value(&storage, egui::app::APP_KEY).unwrap_or_default();

    let sample_set = SampleSet::new();
    let parts_store = PartsStore::new();
        
    let callback_ref:Arc<Mutex<dyn FnMut(&String)>> = Arc::new(Mutex::new(
	move |text: &String| {
	    println!("{}", text);
	    let pfa_in = parser::eval_from_str(text, &sample_set, &parts_store, mode);
	    match pfa_in {
		Ok(pfa) => {println!("valid!")},
		Err(_) => {println!("error!")},
	    }
	}
    ));
    
    app.set_callback(&callback_ref);
    
    egui_glium::run(Box::new(storage), Box::new(app));
}
