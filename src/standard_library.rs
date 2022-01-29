use crate::new_parser::{eval, FunctionMap};

pub fn define_standard_library() -> FunctionMap {
    let mut standard_library = FunctionMap::new();
    // session
    standard_library.insert("sx".to_string(), eval::session::sync_context::sync_context);

    // constructors
    standard_library.insert("nuc".to_string(), eval::constructors::nuc::nuc);
    standard_library.insert("fully".to_string(), eval::constructors::fully::fully);

    // sound events
    standard_library.insert("risset".to_string(), eval::events::sound::sound);
    standard_library.insert("saw".to_string(), eval::events::sound::sound);
    standard_library.insert("sqr".to_string(), eval::events::sound::sound);
    standard_library.insert("cub".to_string(), eval::events::sound::sound);
    standard_library.insert("tri".to_string(), eval::events::sound::sound);
    standard_library.insert("sine".to_string(), eval::events::sound::sound);

    // control event
    standard_library.insert("ctrl".to_string(), eval::events::control::control);

    standard_library
}