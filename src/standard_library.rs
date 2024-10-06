use crate::parser::{eval, FunctionMap};

/**
 * This is where all the "frontend" functions (that is, the DSL functions)
 * are defined and bound to their Rust equivalents ...  
 */
pub fn define_standard_library() -> FunctionMap {
    let standard_library = FunctionMap::new();
    // session
    standard_library.std_lib.insert("sx".to_string(), eval::session::sync_context::sync_context);

    // constructors
    standard_library.std_lib.insert("nuc".to_string(), eval::constructors::nuc::nuc);
    standard_library.std_lib.insert("fully".to_string(), eval::constructors::fully::fully);
    standard_library.std_lib.insert("friendship".to_string(), eval::constructors::friendship::friendship);
    standard_library.std_lib.insert("lin".to_string(), eval::constructors::linear::linear);
    standard_library.std_lib.insert("linear".to_string(), eval::constructors::linear::linear);
    standard_library.std_lib.insert("loop".to_string(), eval::constructors::r#loop::a_loop);
    standard_library.std_lib.insert("chop".to_string(), eval::constructors::chop::chop);
    standard_library.std_lib.insert("infer".to_string(), eval::constructors::infer::infer);
    standard_library.std_lib.insert("rule".to_string(), eval::constructors::infer::rule);
    standard_library.std_lib.insert("learn".to_string(), eval::constructors::learn::learn);
    standard_library.std_lib.insert("cyc".to_string(), eval::constructors::cyc::cyc);
    standard_library.std_lib.insert("flower".to_string(), eval::constructors::flower::flower);
    standard_library.std_lib.insert("stages".to_string(), eval::constructors::stages::stages);
    standard_library.std_lib.insert("facts".to_string(), eval::constructors::facts::facts);
    standard_library.std_lib.insert("vals".to_string(), eval::constructors::vals::vals);
                        
    // commands
    standard_library.std_lib.insert("freeze".to_string(), eval::commands::freeze_buffer);
    standard_library.std_lib.insert("freeze-add".to_string(), eval::commands::freeze_add_buffer);
    standard_library.std_lib.insert("load-sample".to_string(), eval::commands::load_sample);
    standard_library.std_lib.insert("load-wavematrix".to_string(), eval::commands::load_sample_as_wavematrix);
    standard_library.std_lib.insert("load-sample-sets".to_string(), eval::commands::load_sample_sets);
    standard_library.std_lib.insert("load-sample-set".to_string(), eval::commands::load_sample_set);
    standard_library.std_lib.insert("tmod".to_string(), eval::commands::tmod);
    standard_library.std_lib.insert("latency".to_string(), eval::commands::latency);
    standard_library.std_lib.insert("bpm".to_string(), eval::commands::bpm);
    standard_library.std_lib.insert("default-duration".to_string(), eval::commands::default_duration);
    standard_library.std_lib.insert("globres".to_string(), eval::commands::globres);
    standard_library.std_lib.insert("global-resources".to_string(), eval::commands::globres);
    standard_library.std_lib.insert("reverb".to_string(), eval::commands::reverb);
    standard_library.std_lib.insert("delay".to_string(), eval::commands::delay);
    standard_library.std_lib.insert("export-dot".to_string(), eval::commands::export_dot);
    standard_library.std_lib.insert("once".to_string(), eval::commands::once);
    standard_library.std_lib.insert("step-part".to_string(), eval::commands::step_part);
    standard_library.std_lib.insert("clear".to_string(), eval::commands::clear);
    standard_library.std_lib.insert("connect-visualizer".to_string(), eval::commands::connect_visualizer);
    standard_library.std_lib.insert("rec".to_string(), eval::commands::start_recording);
    standard_library.std_lib.insert("stop-rec".to_string(), eval::commands::stop_recording);
    standard_library.std_lib.insert("import-sample-set".to_string(), eval::commands::import_sample_set);
    standard_library.std_lib.insert("print".to_string(), eval::commands::print);
    standard_library.std_lib.insert("load-file".to_string(), eval::commands::load_file);

    
    // progn and other constructs
    standard_library.std_lib.insert("progn".to_string(), eval::progn::progn);
    standard_library.std_lib.insert("match".to_string(), eval::megra_match::megra_match);
    
    // control event
    standard_library.std_lib.insert("ctrl".to_string(), eval::events::control::control);

    // container structs
    standard_library.std_lib.insert("vec".to_string(), eval::vector::vec);
    standard_library.std_lib.insert("push".to_string(), eval::vector::push);
    standard_library.std_lib.insert("map".to_string(), eval::map::map);
    standard_library.std_lib.insert("insert".to_string(), eval::map::insert);
    
    // matrix is barely usable at this point ...
    standard_library.std_lib.insert("mat".to_string(), eval::matrix::mat);
    
    // sound events (sample events are added as needed)
    standard_library.std_lib.insert("risset".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("saw".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("wsaw".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("fmsaw".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("fmsqr".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("fmtri".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("sqr".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("cub".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("tri".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("sine".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("kpp".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("~".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("silence".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("feedr".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("freezr".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("wtab".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("wmat".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("white".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("brown".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("mosc".to_string(), eval::events::sound::sound);
    standard_library.std_lib.insert("blit".to_string(), eval::events::sound::sound);

    // note event
    standard_library.std_lib.insert("note".to_string(), eval::events::note::event_note);

    // modulators
    standard_library.std_lib.insert("lfo~".to_string(), eval::events::modulators::lfo_modulator);
    standard_library.std_lib.insert("lfsaw~".to_string(), eval::events::modulators::lfsaw_modulator);
    standard_library.std_lib.insert("lfrsaw~".to_string(), eval::events::modulators::lfrsaw_modulator);
    standard_library.std_lib.insert("lfsqr~".to_string(), eval::events::modulators::lfsquare_modulator);
    standard_library.std_lib.insert("lftri~".to_string(), eval::events::modulators::lftri_modulator);
    standard_library.std_lib.insert("linramp~".to_string(), eval::events::modulators::lin_ramp_modulator);
    standard_library.std_lib.insert("logramp~".to_string(), eval::events::modulators::log_ramp_modulator);
    standard_library.std_lib.insert("expramp~".to_string(), eval::events::modulators::exp_ramp_modulator);
    standard_library.std_lib.insert("env~".to_string(), eval::events::modulators::multi_point_envelope_modulator);
    
    // parameter events

    // symbolic type paramerters
    standard_library.std_lib.insert("lpt".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("hpt".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("atkt".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dect".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("relt".to_string(), eval::events::parameters::parameter);
    
    standard_library.std_lib.insert("pitch".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pitch-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pitch-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pitch-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pitch-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("midi-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("midi-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("midi-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("midi-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("freq".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("freq-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("freq-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("freq-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("freq-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("lvl".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lvl-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lvl-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lvl-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lvl-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("lpf".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpf-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpf-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpf-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpf-div".to_string(), eval::events::parameters::parameter);
    
    standard_library.std_lib.insert("lpd".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpd-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpd-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpd-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpd-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("lpq".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpq-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpq-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpq-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("lpq-div".to_string(), eval::events::parameters::parameter);
    
    standard_library.std_lib.insert("pff".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pff-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pff-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pff-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pff-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("pfq".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pfq-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pfq-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pfq-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pfq-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("pfg".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pfg-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pfg-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pfg-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pfg-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("hpf".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("hpf-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("hpf-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("hpf-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("hpf-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("hpq".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("hpq-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("hpq-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("hpq-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("hpq-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("azi".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("azi-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("azi-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("azi-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("azi-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("ele".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("ele-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("ele-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("ele-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("ele-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("atk".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("atk-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("atk-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("atk-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("atk-div".to_string(), eval::events::parameters::parameter);

    // attack peak
    standard_library.std_lib.insert("atkp".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("atkp-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("atkp-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("atkp-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("atkp-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("sus".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("sus-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("sus-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("sus-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("sus-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("dec".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dec-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dec-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dec-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dec-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("rel".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rel-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rel-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rel-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rel-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("pos".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pos-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pos-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pos-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pos-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("dur".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dur-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dur-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dur-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dur-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("del".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("del-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("del-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("del-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("del-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("rev".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rev-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rev-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rev-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rev-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("pw".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pw-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pw-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pw-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("pw-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("start".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("start-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("start-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("start-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("start-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("rate".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rate-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rate-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rate-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("rate-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("gain".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("gain-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("gain-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("gain-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("gain-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("amp".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("amp-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("amp-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("amp-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("amp-div".to_string(), eval::events::parameters::parameter);
    
    standard_library.std_lib.insert("dist".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dist-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dist-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dist-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("dist-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("bcbits".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcbits-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcbits-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcbits-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcbits-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("bcdown".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcdown-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcdown-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcdown-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcdown-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("bcmix".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcmix-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcmix-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcmix-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("bcmix-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("bcmode".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("nharm".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("nharm-add".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("nharm-mul".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("nharm-sub".to_string(), eval::events::parameters::parameter);
    standard_library.std_lib.insert("nharm-div".to_string(), eval::events::parameters::parameter);

    standard_library.std_lib.insert("keys".to_string(), eval::events::parameters::sample_keys);
    standard_library.std_lib.insert("keys-add".to_string(), eval::events::parameters::sample_keys);
    standard_library.std_lib.insert("keys-sub".to_string(), eval::events::parameters::sample_keys);

    standard_library.std_lib.insert("sample-number".to_string(), eval::events::parameters::sample_number);
    standard_library.std_lib.insert("sample-number-add".to_string(), eval::events::parameters::sample_number);
    standard_library.std_lib.insert("sample-number-mul".to_string(), eval::events::parameters::sample_number);
    standard_library.std_lib.insert("sample-number-sub".to_string(), eval::events::parameters::sample_number);
    standard_library.std_lib.insert("sample-number-div".to_string(), eval::events::parameters::sample_number);

    standard_library.std_lib.insert("sno".to_string(), eval::events::parameters::sample_number);
    standard_library.std_lib.insert("sno-add".to_string(), eval::events::parameters::sample_number);
    standard_library.std_lib.insert("sno-mul".to_string(), eval::events::parameters::sample_number);
    standard_library.std_lib.insert("sno-sub".to_string(), eval::events::parameters::sample_number);
    standard_library.std_lib.insert("sno-div".to_string(), eval::events::parameters::sample_number);

    standard_library.std_lib.insert("random-sample".to_string(), eval::events::parameters::random_sample);
    standard_library.std_lib.insert("rands".to_string(), eval::events::parameters::random_sample);
        
    // some shorthands 
    standard_library.std_lib.insert("transpose".to_string(), eval::events::parameters::transpose);
    standard_library.std_lib.insert("tpo".to_string(), eval::events::parameters::transpose);

    // dynpars
    standard_library.std_lib.insert("bounce".to_string(), eval::dynpar::bounce);
    standard_library.std_lib.insert("brownian".to_string(), eval::dynpar::brownian);
    standard_library.std_lib.insert("randr".to_string(), eval::dynpar::randrange);
    standard_library.std_lib.insert("env".to_string(), eval::dynpar::env);
    standard_library.std_lib.insert("fade".to_string(), eval::dynpar::fade);

    // generator processors
    standard_library.std_lib.insert("pear".to_string(), eval::generator_processor::eval_pear);
    standard_library.std_lib.insert("apple".to_string(), eval::generator_processor::eval_apple);
    standard_library.std_lib.insert("every".to_string(), eval::generator_processor::eval_every);
    standard_library.std_lib.insert("mapper".to_string(), eval::generator_processor::eval_mapper);
    standard_library.std_lib.insert("life".to_string(), eval::generator_processor::eval_lifemodel);
    standard_library.std_lib.insert("inhibit".to_string(), eval::generator_processor::eval_inhibit);
    standard_library.std_lib.insert("exhibit".to_string(), eval::generator_processor::eval_exhibit);

    // composition
    standard_library.std_lib.insert("cmp".to_string(), eval::compose::compose);
    standard_library.std_lib.insert("compose".to_string(), eval::compose::compose);
    standard_library.std_lib.insert("ls".to_string(), eval::generator_list::generator_list);
    standard_library.std_lib.insert("list".to_string(), eval::generator_list::generator_list);
    standard_library.std_lib.insert("spread".to_string(), eval::generator_list::spread_list);

    // multiplyer
    standard_library.std_lib.insert("xspread".to_string(), eval::multiplyer::eval_xspread);
    standard_library.std_lib.insert("xdup".to_string(), eval::multiplyer::eval_xdup);
    
    // generator modifiers
    standard_library.std_lib.insert("haste".to_string(), eval::generator_modifier::eval_haste);
    standard_library.std_lib.insert("shift".to_string(), eval::generator_modifier::eval_shift);
    standard_library.std_lib.insert("relax".to_string(), eval::generator_modifier::eval_relax);
    standard_library.std_lib.insert("grow".to_string(), eval::generator_modifier::eval_grow);
    standard_library.std_lib.insert("grown".to_string(), eval::generator_modifier::eval_grown);
    standard_library.std_lib.insert("shrink".to_string(), eval::generator_modifier::eval_shrink);
    standard_library.std_lib.insert("solidify".to_string(), eval::generator_modifier::eval_solidify);
    standard_library.std_lib.insert("blur".to_string(), eval::generator_modifier::eval_blur);
    standard_library.std_lib.insert("sharpen".to_string(), eval::generator_modifier::eval_sharpen);
    standard_library.std_lib.insert("shake".to_string(), eval::generator_modifier::eval_shake);
    standard_library.std_lib.insert("skip".to_string(), eval::generator_modifier::eval_skip);
    standard_library.std_lib.insert("rewind".to_string(), eval::generator_modifier::eval_rewind);
    standard_library.std_lib.insert("rnd".to_string(), eval::generator_modifier::eval_rnd);
    standard_library.std_lib.insert("rep".to_string(), eval::generator_modifier::eval_rep);
    standard_library.std_lib.insert("reverse".to_string(), eval::generator_modifier::eval_reverse);
    standard_library.std_lib.insert("keep".to_string(), eval::generator_modifier::eval_keep);

    // arithmetic
    standard_library.std_lib.insert("add".to_string(), eval::arithmetic::add);
    standard_library.std_lib.insert("mul".to_string(), eval::arithmetic::mul);
    standard_library.std_lib.insert("sub".to_string(), eval::arithmetic::sub);
    standard_library.std_lib.insert("div".to_string(), eval::arithmetic::div);
    standard_library.std_lib.insert("mod".to_string(), eval::arithmetic::modulo);
    standard_library.std_lib.insert("pow".to_string(), eval::arithmetic::pow);
    standard_library.std_lib.insert("max".to_string(), eval::arithmetic::max);
    standard_library.std_lib.insert("min".to_string(), eval::arithmetic::min);
    standard_library.std_lib.insert("round".to_string(), eval::arithmetic::round);
    standard_library.std_lib.insert("floor".to_string(), eval::arithmetic::floor);
    standard_library.std_lib.insert("ceil".to_string(), eval::arithmetic::ceil);
    
    // comparison
    standard_library.std_lib.insert(">=".to_string(), eval::comparison::greater_equal);
    standard_library.std_lib.insert(">".to_string(), eval::comparison::greater);
    standard_library.std_lib.insert("==".to_string(), eval::comparison::equal);
    standard_library.std_lib.insert("<=".to_string(), eval::comparison::lesser_equal);
    standard_library.std_lib.insert("<".to_string(), eval::comparison::lesser);
    standard_library.std_lib.insert("istype".to_string(), eval::comparison::is_type);
    
    // midi helpers
    standard_library.std_lib.insert("mtof".to_string(), eval::midi_helpers::mtof);
    standard_library.std_lib.insert("mtosym".to_string(), eval::midi_helpers::mtosym);
    standard_library.std_lib.insert("veltodyn".to_string(), eval::midi_helpers::veltodyn);
    standard_library.std_lib.insert("symtof".to_string(), eval::midi_helpers::symtofreq);
    standard_library.std_lib.insert("mtovex".to_string(), eval::midi_helpers::mtovex);
    
    // string helpers
    standard_library.std_lib.insert("concat".to_string(), eval::string_helpers::concat);

    // osc
    standard_library.std_lib.insert("osc-sender".to_string(), eval::osc::osc_define_sender);
    standard_library.std_lib.insert("osc-send".to_string(), eval::osc::osc_send);
    standard_library.std_lib.insert("osc-receiver".to_string(), eval::osc::osc_start_receiver);

    // midi
    standard_library.std_lib.insert("list-midi-ports".to_string(), eval::midi::eval_list_midi_ports);
    standard_library.std_lib.insert("open-midi-port".to_string(), eval::midi::open_midi_port);
        
    // types for osc and other stuff
    standard_library.std_lib.insert("f64".to_string(), eval::types::double);
    standard_library.std_lib.insert("i32".to_string(), eval::types::int);
    standard_library.std_lib.insert("i64".to_string(), eval::types::long);
    standard_library.std_lib.insert("pair".to_string(), eval::types::pair);
    standard_library.std_lib.insert("to-string".to_string(), eval::types::to_string);

    // event getters
    standard_library.std_lib.insert("ev-param".to_string(), eval::event_getters::event_param);
    standard_library.std_lib.insert("ev-tag".to_string(), eval::event_getters::event_tag);
    standard_library.std_lib.insert("ev-name".to_string(), eval::event_getters::event_name);

    standard_library
}
