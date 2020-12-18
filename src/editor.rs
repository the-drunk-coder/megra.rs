use std::sync;
use parking_lot::Mutex;

use kas::event::*;
use kas::macros::make_widget;
use kas::widget::Window;
use ruffbox_synth::ruffbox::Ruffbox;

mod custom_editbox;
use custom_editbox::EditBox;

use crate::session::OutputMode;

pub fn run_editor<const BUFSIZE:usize, const NCHAN:usize>(ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE, NCHAN>>>, mode: OutputMode) -> Result<(), anyhow::Error> {

    let doc = r"(sx 'ga #t (infer 'tral :events 'a (saw 200) :rules (rule 'a 'a 100 400)))";
        
    let window = Window::new(
        "Mégra",
        make_widget! {
            #[layout(grid)]
            #[handler(msg = VoidMsg)]
            struct<const BUFSIZE:usize, const NCHAN:usize> {
                #[widget(row=0, col=0)]
		editor: EditBox<BUFSIZE, NCHAN> = EditBox::new(doc, ruffbox, mode),
            }	    
        },
    );

    let theme = kas_theme::FlatTheme::new().with_colours("dark").with_font_size(7.0);
    kas_wgpu::Toolkit::new(theme)?.with(window)?.run()    
}
