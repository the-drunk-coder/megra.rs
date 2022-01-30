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
    standard_library.fmap.insert("flower".to_string(), eval::constructors::cyc::cyc);
    standard_library.fmap.insert("stages".to_string(), eval::constructors::cyc::cyc);

    // commands
    standard_library.fmap.insert("load-part".to_string(), eval::commands::load_part);
    standard_library.fmap.insert("freeze".to_string(), eval::commands::freeze_buffer);
    standard_library.fmap.insert("load-sample".to_string(), eval::commands::load_sample);
    standard_library.fmap.insert("load-sample-sets".to_string(), eval::commands::load_sample_sets);
    standard_library.fmap.insert("load-sample-set".to_string(), eval::commands::load_sample_set);
    standard_library.fmap.insert("tmod".to_string(), eval::commands::tmod);
    standard_library.fmap.insert("latency".to_string(), eval::commands::latency);
    standard_library.fmap.insert("bpm".to_string(), eval::commands::bpm);
    standard_library.fmap.insert("default-duration".to_string(), eval::commands::default_duration);
    standard_library.fmap.insert("globres".to_string(), eval::commands::globres);
    standard_library.fmap.insert("global-resources".to_string(), eval::commands::globres);
    standard_library.fmap.insert("reverb".to_string(), eval::commands::reverb);
    standard_library.fmap.insert("delay".to_string(), eval::commands::delay);
    standard_library.fmap.insert("export-dot".to_string(), eval::commands::export_dot);
    standard_library.fmap.insert("once".to_string(), eval::commands::once);
    standard_library.fmap.insert("step-part".to_string(), eval::commands::step_part);
    standard_library.fmap.insert("clear".to_string(), eval::commands::clear);
    
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
