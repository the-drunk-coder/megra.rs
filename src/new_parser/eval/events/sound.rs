use crate::parameter::Parameter;
use ruffbox_synth::ruffbox::synth::SynthParameter;
use crate::event::{Event, EventOperation};
use std::sync;
use crate::event_helpers::map_parameter;
use crate::new_parser::{BuiltIn2, EvaluatedExpr};
use crate::{SampleSet, GlobalParameters};
use crate::music_theory;

pub fn sound(tail: &mut Vec<EvaluatedExpr>,
	     _global_parameters: &sync::Arc<GlobalParameters>,
	     _sample_set: &sync::Arc<sync::Mutex<SampleSet>>) -> Option<EvaluatedExpr> {
    
    let mut tail_drain = tail.drain(..);

    let fname = if let Some(EvaluatedExpr::FunctionName(f)) = tail_drain.next() {
	f
    } else {
	// nothing to do ...
	return None;
    };
    
    let mut ev = match fname.as_str() {
        "sine" => Event::with_name_and_operation("sine".to_string(), EventOperation::Replace),
        "tri" => Event::with_name_and_operation("tri".to_string(), EventOperation::Replace),
        "saw" => Event::with_name_and_operation("saw".to_string(), EventOperation::Replace),
        "sqr" => Event::with_name_and_operation("sqr".to_string(), EventOperation::Replace),
        "cub" => Event::with_name_and_operation("cub".to_string(), EventOperation::Replace),
        "risset" => Event::with_name_and_operation("risset".to_string(), EventOperation::Replace),
        _ => return None
	// TODO sample events !!
    };

    // first arg is always freq ...
    ev.params.insert(
        SynthParameter::PitchFrequency,
        Box::new(match tail_drain.next() {
            Some(EvaluatedExpr::Float(n)) => Parameter::with_value(n),
            //Some(Expr::Constant(Atom::Parameter(pl))) => pl,
	    Some(EvaluatedExpr::Symbol(s)) => Parameter::with_value(music_theory::to_freq(
		music_theory::from_string(&s),
		music_theory::Tuning::EqualTemperament,
	    )),
            _ => Parameter::with_value(100.0),
	}));

    // set some defaults 2
    ev.params
        .insert(SynthParameter::Level, Box::new(Parameter::with_value(0.3)));
    ev.params
        .insert(SynthParameter::Attack, Box::new(Parameter::with_value(2.0)));
    ev.params.insert(
        SynthParameter::Sustain,
        Box::new(Parameter::with_value(48.0)),
    );
    ev.params.insert(
        SynthParameter::Release,
        Box::new(Parameter::with_value(100.0)),
    );
    ev.params.insert(
        SynthParameter::ChannelPosition,
        Box::new(Parameter::with_value(0.00)),
    );

    while let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        if k == "tags" {
            while let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
                ev.tags.insert(s);
            }
        } else {
            ev.params.insert(map_parameter(&k), Box::new(match tail_drain.next() {
		    Some(EvaluatedExpr::Float(n)) => Parameter::with_value(n),
		    //Some(EvaluatedExpr::Parameter(pl)) => pl,
		    _ => Parameter::with_value(0.0),
		}));
        }
    }

    Some(EvaluatedExpr::BuiltIn(BuiltIn2::SoundEvent(ev)))
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::new_parser::*;

    #[test]
    fn test_eval_sound() {
	let snippet = "(risset 4000 :lvl 1.0)";
	let mut functions = FunctionMap::new();
	let sample_set =  sync::Arc::new(sync::Mutex::new(SampleSet::new()));
	
	functions.insert("risset".to_string(), eval::events::sound::sound);
			 
	let globals = sync::Arc::new(GlobalParameters::new());
	
	match eval_from_str2(snippet, &functions, &globals, &sample_set) {
            Ok(res) => {
                assert!(matches!(res, EvaluatedExpr::BuiltIn(BuiltIn2::SoundEvent(_))));
            }
            Err(e) => {
                println!("err {}", e);
                assert!(false)
            }
        }
    }
}
