// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Markdown parsing demo


use kas::event::*;
use kas::macros::make_widget;
use kas::widget::Window;

mod custom_editbox;
use custom_editbox::{EditBox, EditBoxVoid};

pub fn run_editor() -> Result<(), anyhow::Error> {

    let doc = r"Hello";

    let window = Window::new(
        "Markdown parser",
        make_widget! {
            #[layout(grid)]
            #[handler(msg = VoidMsg)]
            struct {
                #[widget(row=0, col=0)] editor: EditBoxVoid = EditBox::new(doc).multi_line(true),                
            }	    
        },
    );

    let theme = kas_theme::FlatTheme::new().with_colours("dark").with_font_size(7.0);
    kas_wgpu::Toolkit::new(theme)?.with(window)?.run()    
}
