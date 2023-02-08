// editor modules
mod livecode_text_edit;
mod syntax_highlighting;

use parking_lot::Mutex;
use ruffbox_synth::ruffbox::RuffboxControls;
use std::collections::HashMap;
use std::sync;

mod megra_editor;
use megra_editor::{EditorFont, MegraEditor};

use crate::builtin_types::*;
use crate::interpreter;
use crate::parser;
use crate::parser::FunctionMap;
use crate::sample_set::SampleAndWavematrixSet;
use crate::session::{OutputMode, Session};

#[allow(clippy::too_many_arguments)]
pub fn run_editor<const BUFSIZE: usize, const NCHAN: usize>(
    function_map: &sync::Arc<Mutex<FunctionMap>>,
    midi_callback_map: &sync::Arc<Mutex<HashMap<u8, Command>>>,
    session: &sync::Arc<Mutex<Session<BUFSIZE, NCHAN>>>,
    ruffbox: &sync::Arc<RuffboxControls<BUFSIZE, NCHAN>>,
    global_parameters: &sync::Arc<GlobalParameters>,
    sample_set: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    parts_store: &sync::Arc<Mutex<PartsStore>>,
    base_dir: String,
    create_sketch: bool,
    mode: OutputMode,
    font: Option<&str>,
    font_size: f32,
) -> std::result::Result<(), eframe::Error> {
    let session2 = sync::Arc::clone(session);
    let function_map2 = sync::Arc::clone(function_map);
    let midi_callback_map2 = sync::Arc::clone(midi_callback_map);
    let ruffbox2 = sync::Arc::clone(ruffbox);
    let sample_set2 = sync::Arc::clone(sample_set);
    let global_parameters2 = sync::Arc::clone(global_parameters);
    let parts_store2 = sync::Arc::clone(parts_store);
    let base_dir_2 = base_dir.clone();

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
                        &midi_callback_map2,
                        &session2,
                        &ruffbox2,
                        &global_parameters2,
                        &sample_set2,
                        &parts_store2,
                        mode,
                        base_dir_2.to_string(),
                    );
                }
                Err(e) => {
                    println!("could not parse this! {} {}", text, e)
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

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Mégra Editor",
        native_options,
        Box::new(|cc| {
            let mut inner_app = MegraEditor::new(cc, base_dir, cs);
            inner_app.set_font_size(fs);
            inner_app.set_font(ifont);
            inner_app.set_callback(callback_ref);
            Box::new(inner_app)
        }),
    )
}
