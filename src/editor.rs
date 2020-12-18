use std::sync;
use parking_lot::Mutex;

use kas::event::*;
use kas::macros::make_widget;
use kas::widget::Window;
use ruffbox_synth::ruffbox::Ruffbox;

mod custom_editbox;
use custom_editbox::{EditBox, EditBoxVoid};

use crate::builtin_types::*;
use crate::session::{Session, OutputMode};
use crate::parser;
use crate::interpreter;


pub fn run_editor<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>, mode: OutputMode) -> Result<(), anyhow::Error> {

    let doc = r"Hello";
    
    let session = sync::Arc::new(Mutex::new(Session::<BUFSIZE, NCHAN>::with_mode(mode)));
    let sample_set = Box::new(SampleSet::new());
    let parts_store = Box::new(PartsStore::new());

    let callback = move |text: &String| {

	/*
	let pfa_in = parser::eval_from_str(text, &sample_set, &parts_store, mode);		
	match pfa_in {
	    Err(e) => {
		println!("can't parse");			
	    },
	    Ok(pfa) => {
		interpreter::interpret(&session, &mut sample_set, &mut parts_store, pfa, &ruffbox);
	    }
	}*/
    };
    
    let window = Window::new(
        "Markdown parser",
        make_widget! {
            #[layout(grid)]
            #[handler(msg = VoidMsg)]
            struct {
                #[widget(row=0, col=0)] editor: EditBoxVoid = EditBox::new(doc, callback).multi_line(true),                
            }	    
        },
    );

    let theme = kas_theme::FlatTheme::new().with_colours("dark").with_font_size(7.0);
    kas_wgpu::Toolkit::new(theme)?.with(window)?.run()    
}
