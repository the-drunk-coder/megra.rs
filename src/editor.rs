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
    
    // Restore editor from file, or create new editor:
    let mut app: MegraEditor = MegraEditor::default();

    let mut sample_set = SampleSet::new();
    let mut parts_store = PartsStore::new();
    let session = Arc::new(Mutex::new(Session::with_mode(mode)));
    let ruffbox2 = Arc::clone(ruffbox);
        
    let callback_ref:Arc<Mutex<dyn FnMut(&String)>> = Arc::new(Mutex::new(
	move |text: &String| {	    
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
    
    egui_glium::run(Box::new(app));
}
