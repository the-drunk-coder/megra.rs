use std::sync;
use parking_lot::Mutex;

use crate::session::OutputMode;

use ruffbox_synth::ruffbox::Ruffbox;

mod megra_editor;
use megra_editor::MegraEditor;


pub fn run_editor<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>, mode: OutputMode) {
    let title = "Mégra Editor";

    // Persist app state to file:
    let storage = egui_glium::storage::FileStorage::from_path(".megra_edit.json");
    
    // Restore editor from file, or create new editor:
    let app: MegraEditor = egui::app::get_value(&storage, egui::app::APP_KEY).unwrap_or_default();

    egui_glium::run(title, Box::new(storage), Box::new(app));
}
