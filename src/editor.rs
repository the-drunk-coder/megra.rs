use std::sync::*;
use parking_lot::Mutex;
use ruffbox_synth::ruffbox::Ruffbox;

mod megra_editor;
use megra_editor::MegraEditor;

use crate::session::{Session, OutputMode};
use crate::builtin_types::*;
use crate::parser;
use crate::interpreter;

pub fn run_editor<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>, mode: OutputMode) {

    // Persist app state to file:
    let storage = egui_glium::storage::FileStorage::from_path(".megra_edit.json");
    
    // Restore editor from file, or create new editor:
    let mut app: MegraEditor = egui::app::get_value(&storage, egui::app::APP_KEY).unwrap_or_default();

    let mut sample_set = SampleSet::new();
    let mut parts_store = PartsStore::new();
    let session = Arc::new(Mutex::new(Session::with_mode(mode)));
    let ruffbox2 = Arc::clone(ruffbox);
        
    let callback_ref:Arc<Mutex<dyn FnMut(&String)>> = Arc::new(Mutex::new(
	move |text: &String| {
	    println!("{}", text);
	    let pfa_in = parser::eval_from_str(text, &sample_set, &parts_store, mode);
	    match pfa_in {
		Ok(pfa) => {
		    interpreter::interpret(&session, &mut sample_set, &mut parts_store, pfa, &ruffbox2);
		},
		Err(_) => {println!("could not parse this!")},
	    }
	}
    ));
    
    app.set_callback(&callback_ref);
    
    egui_glium::run(Box::new(storage), Box::new(app));
}
