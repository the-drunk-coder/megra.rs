// editor modules
mod livecode_text_edit;
mod syntax_highlighting;

use parking_lot::Mutex;

use std::sync;

mod megra_editor;
use megra_editor::{EditorFont, MegraEditor};

use crate::interpreter;
use crate::parser;

use crate::session::Session;

pub fn run_editor<const BUFSIZE: usize, const NCHAN: usize>(
    session: Session<BUFSIZE, NCHAN>,
    base_dir: String,
    create_sketch: bool,
    font: Option<&str>,
    font_size: f32,
    karl_yerkes_mode: bool,
) -> std::result::Result<(), eframe::Error> {
    let globals2 = sync::Arc::clone(&session.globals);
    let base_dir_2 = base_dir.clone();

    let callback_ref: sync::Arc<Mutex<dyn FnMut(&String)>> =
        sync::Arc::new(Mutex::new(move |text: &String| {
            let pfa_in = parser::eval_from_str(
                text,
                &session.functions,
                &globals2,
                session.sample_set.clone(),
                session.output_mode,
            );
            match pfa_in {
                Ok(pfa) => {
                    interpreter::interpret(pfa, session.clone(), base_dir_2.to_string());
                }
                Err(e) => {
                    println!("could not parse this! {text} {e}")
                }
            }
        }));

    let ifont = match font {
        Some("mononoki") => EditorFont::Mononoki,
        Some("ComicMono") => EditorFont::ComicMono,
        Some(path) => EditorFont::Custom(path.to_string()),
        _ => EditorFont::Mononoki,
    };

    // this is super awkward but I need to get around the
    // static lifetime somehow ...
    let fs = sync::Arc::new(font_size);
    let cs = sync::Arc::new(create_sketch);
    let kybr = sync::Arc::new(karl_yerkes_mode);

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Mégra Editor",
        native_options,
        Box::new(|cc| {
            let mut inner_app = MegraEditor::new(cc, base_dir, cs, kybr);
            inner_app.set_font_size(fs);
            inner_app.set_font(ifont);
            inner_app.set_callback(callback_ref);

            Box::new(inner_app)
        }),
    )
}
