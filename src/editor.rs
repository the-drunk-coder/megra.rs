// editor modules
mod livecode_text_edit;
mod syntax_highlighting;

use parking_lot::Mutex;
use ruffbox_synth::ruffbox::RuffboxControls;
use std::sync;

mod megra_editor;
use megra_editor::{EditorFont, MegraEditor};

use crate::builtin_types::*;
use crate::interpreter;
use crate::parser;
use crate::parser::FunctionMap;
use crate::sample_set::SampleSet;
use crate::session::{OutputMode, Session};

#[allow(clippy::too_many_arguments)]
pub fn run_editor<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    global_parameters: &sync::Arc<GlobalParameters>,
    sample_set: &sync::Arc<Mutex<SampleSet>>,
    parts_store: &sync::Arc<Mutex<PartsStore>>,
    mode: OutputMode,
    font: Option<&str>,
) {
    // Restore editor from file, or create new editor:
    let mut app: MegraEditor = MegraEditor::default();
    match font {
        Some("mononoki") => {
            app.set_font(EditorFont::Mononoki);
        }
        Some("ComicMono") => {
            app.set_font(EditorFont::ComicMono);
        }
        Some(path) => {
            app.set_font(EditorFont::Custom(path.to_string()));
        }
        _ => {}
    }

    let session2 = sync::Arc::clone(session);
    let function_map2 = sync::Arc::clone(function_map);
    let ruffbox2 = sync::Arc::clone(ruffbox);
    let sample_set2 = sync::Arc::clone(sample_set);
    let global_parameters2 = sync::Arc::clone(global_parameters);
    let parts_store2 = sync::Arc::clone(parts_store);

    let callback_ref: sync::Arc<Mutex<dyn FnMut(&String)>> =
        sync::Arc::new(Mutex::new(move |text: &String| {
            let pfa_in = parser::eval_from_str(
                text,
                &function_map2.lock(),
                &global_parameters2,
                &sample_set2,
                mode,
            );
            match pfa_in {
                Ok(pfa) => {
                    interpreter::interpret(
                        pfa,
                        &function_map2,
                        &session2,
                        &ruffbox2,
                        &global_parameters2,
                        &sample_set2,
                        &parts_store2,
                        mode,
                    );
                }
                Err(e) => {
                    println!("could not parse this! {} {}", text, e)
                }
            }
        }));

    app.set_callback(&callback_ref);

    egui_glium::run(Box::new(app), &epi::NativeOptions::default());
}
