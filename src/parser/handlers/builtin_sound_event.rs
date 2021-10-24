use crate::builtin_types::*;
use crate::event::Event;
use crate::parameter::Parameter;
use crate::parser::parser_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;

pub fn handle(event_type: &BuiltInSoundEvent, tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);

    let mut ev = match event_type {
        BuiltInSoundEvent::Sine(o) => Event::with_name_and_operation("sine".to_string(), *o),
        BuiltInSoundEvent::Tri(o) => Event::with_name_and_operation("tri".to_string(), *o),
        BuiltInSoundEvent::Saw(o) => Event::with_name_and_operation("saw".to_string(), *o),
        BuiltInSoundEvent::Square(o) => Event::with_name_and_operation("sqr".to_string(), *o),
        BuiltInSoundEvent::Cub(o) => Event::with_name_and_operation("cub".to_string(), *o),
        BuiltInSoundEvent::RissetBell(o) => {
            Event::with_name_and_operation("risset".to_string(), *o)
        }
    };

    
    
    // first arg is always freq ...
    ev.params.insert(
        SynthParameter::PitchFrequency,
        Box::new(get_next_pitch_param(&mut tail_drain, 100.0)),
    );

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

    get_keyword_params(&mut ev.params, &mut tail_drain);

    Atom::SoundEvent(ev)
}
