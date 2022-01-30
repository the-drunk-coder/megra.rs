use crate::parser::{eval, FunctionMap};

pub fn define_standard_library() -> FunctionMap {
    let mut standard_library = FunctionMap::new();
    // session
    standard_library.fmap.insert("sx".to_string(), eval::session::sync_context::sync_context);

    // constructors
    standard_library.fmap.insert("nuc".to_string(), eval::constructors::nuc::nuc);
    standard_library.fmap.insert("fully".to_string(), eval::constructors::fully::fully);
    standard_library.fmap.insert("friendship".to_string(), eval::constructors::friendship::friendship);
    standard_library.fmap.insert("linear".to_string(), eval::constructors::linear::linear);
    standard_library.fmap.insert("loop".to_string(), eval::constructors::r#loop::a_loop);
    standard_library.fmap.insert("chop".to_string(), eval::constructors::chop::chop);
    standard_library.fmap.insert("infer".to_string(), eval::constructors::infer::infer);
    standard_library.fmap.insert("rule".to_string(), eval::constructors::infer::rule);
    standard_library.fmap.insert("learn".to_string(), eval::constructors::learn::learn);
    standard_library.fmap.insert("cyc".to_string(), eval::constructors::cyc::cyc);
    
    // sound events (sample events are added as needed)
    standard_library.fmap.insert("risset".to_string(), eval::events::sound::sound);
    standard_library.fmap.insert("saw".to_string(), eval::events::sound::sound);
    standard_library.fmap.insert("sqr".to_string(), eval::events::sound::sound);
    standard_library.fmap.insert("cub".to_string(), eval::events::sound::sound);
    standard_library.fmap.insert("tri".to_string(), eval::events::sound::sound);
    standard_library.fmap.insert("sine".to_string(), eval::events::sound::sound);

    // control event
    standard_library.fmap.insert("ctrl".to_string(), eval::events::control::control);

    standard_library
}
