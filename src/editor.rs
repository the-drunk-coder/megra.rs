use std::sync::*;
use parking_lot::Mutex;
use ruffbox_synth::ruffbox::Ruffbox;

mod megra_editor;
use megra_editor::MegraEditor;

use crate::session::{Session, OutputMode};
use crate::sample_set::SampleSet;
use crate::builtin_types::*;
use crate::parser;
use crate::interpreter;

pub fn run_editor<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>, mode: OutputMode) {
    
    // Restore editor from file, or create new editor:
    let mut app: MegraEditor = MegraEditor::default();

    let mut sample_set = Arc::new(Mutex::new(SampleSet::new()));
    let mut parts_store = PartsStore::new();
    let session = Arc::new(Mutex::new(Session::with_mode(mode)));
    let ruffbox2 = Arc::clone(ruffbox);
    let global_parameters = Arc::new(GlobalParameters::with_capacity(1));
        
    let callback_ref:Arc<Mutex<dyn FnMut(&String)>> = Arc::new(Mutex::new(	
	move |text: &String| {
	    let pfa_in = parser::eval_from_str(text, &sample_set, &parts_store, mode);
	    match pfa_in {
		Ok(pfa) => {
		    interpreter::interpret(pfa, &session, &ruffbox2, &global_parameters, &mut sample_set, &mut parts_store);
		},
		Err(_) => {println!("could not parse this! {}", text)},
	    }
	}
    ));
    
    app.set_callback(&callback_ref);
    
    egui_glium::run(Box::new(app));
}
