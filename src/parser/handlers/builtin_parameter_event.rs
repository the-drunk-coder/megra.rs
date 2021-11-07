use crate::builtin_types::*;
use crate::event::*;
use crate::parser::parser_helpers::*;
use ruffbox_synth::ruffbox::synth::SynthParameter;

pub fn handle(event_type: &BuiltInParameterEvent, tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);

    let mut ev = match event_type {
        BuiltInParameterEvent::PitchFrequency(o) => {
            Event::with_name_and_operation("freq".to_string(), *o)
        }
        BuiltInParameterEvent::Attack(o) => Event::with_name_and_operation("atk".to_string(), *o), // milliseconds, not seconds
        BuiltInParameterEvent::Release(o) => Event::with_name_and_operation("rel".to_string(), *o), // milliseconds, not seconds
        BuiltInParameterEvent::Sustain(o) => Event::with_name_and_operation("sus".to_string(), *o), // milliseconds, not seconds
        BuiltInParameterEvent::ChannelPosition(o) => {
            Event::with_name_and_operation("pos".to_string(), *o)
        }
        BuiltInParameterEvent::Level(o) => Event::with_name_and_operation("lvl".to_string(), *o),
        BuiltInParameterEvent::Duration(o) => Event::with_name_and_operation("dur".to_string(), *o),
        BuiltInParameterEvent::Reverb(o) => Event::with_name_and_operation("rev".to_string(), *o),
        BuiltInParameterEvent::Delay(o) => Event::with_name_and_operation("del".to_string(), *o),
        BuiltInParameterEvent::LpFreq(o) => Event::with_name_and_operation("lpf".to_string(), *o),
        BuiltInParameterEvent::LpQ(o) => Event::with_name_and_operation("lpq".to_string(), *o),
        BuiltInParameterEvent::LpDist(o) => Event::with_name_and_operation("lpd".to_string(), *o),
        BuiltInParameterEvent::HpFreq(o) => Event::with_name_and_operation("hpf".to_string(), *o),
        BuiltInParameterEvent::HpQ(o) => Event::with_name_and_operation("hpq".to_string(), *o),
        BuiltInParameterEvent::PeakFreq(o) => Event::with_name_and_operation("pff".to_string(), *o),
        BuiltInParameterEvent::PeakQ(o) => Event::with_name_and_operation("pfq".to_string(), *o),
        BuiltInParameterEvent::PeakGain(o) => Event::with_name_and_operation("pfg".to_string(), *o),
        BuiltInParameterEvent::Pulsewidth(o) => {
            Event::with_name_and_operation("pw".to_string(), *o)
        }
        BuiltInParameterEvent::PlaybackStart(o) => {
            Event::with_name_and_operation("start".to_string(), *o)
        }
        BuiltInParameterEvent::PlaybackRate(o) => {
            Event::with_name_and_operation("rate".to_string(), *o)
        }
    };

    let param_key = match event_type {
        BuiltInParameterEvent::PitchFrequency(_) => SynthParameter::PitchFrequency,
        BuiltInParameterEvent::Attack(_) => SynthParameter::Attack,
        BuiltInParameterEvent::Release(_) => SynthParameter::Release,
        BuiltInParameterEvent::Sustain(_) => SynthParameter::Sustain,
        BuiltInParameterEvent::ChannelPosition(_) => SynthParameter::ChannelPosition,
        BuiltInParameterEvent::Level(_) => SynthParameter::Level,
        BuiltInParameterEvent::Duration(_) => SynthParameter::Duration,
        BuiltInParameterEvent::Reverb(_) => SynthParameter::ReverbMix,
        BuiltInParameterEvent::Delay(_) => SynthParameter::DelayMix,
        BuiltInParameterEvent::LpFreq(_) => SynthParameter::LowpassCutoffFrequency,
        BuiltInParameterEvent::LpQ(_) => SynthParameter::LowpassQFactor,
        BuiltInParameterEvent::LpDist(_) => SynthParameter::LowpassFilterDistortion,
        BuiltInParameterEvent::HpFreq(_) => SynthParameter::HighpassCutoffFrequency,
        BuiltInParameterEvent::HpQ(_) => SynthParameter::HighpassQFactor,
        BuiltInParameterEvent::PeakFreq(_) => SynthParameter::PeakFrequency,
        BuiltInParameterEvent::PeakQ(_) => SynthParameter::PeakQFactor,
        BuiltInParameterEvent::PeakGain(_) => SynthParameter::PeakGain,
        BuiltInParameterEvent::Pulsewidth(_) => SynthParameter::Pulsewidth,
        BuiltInParameterEvent::PlaybackStart(_) => SynthParameter::PlaybackStart,
        BuiltInParameterEvent::PlaybackRate(_) => SynthParameter::PlaybackRate,
    };

    ev.params
        .insert(param_key, Box::new(get_next_param(&mut tail_drain, 0.0)));

    Atom::SoundEvent(ev)
}
