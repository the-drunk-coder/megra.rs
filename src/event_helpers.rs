use ruffbox_synth::ruffbox::synth::SourceType;
use ruffbox_synth::ruffbox::synth::SynthParameter;

pub fn map_name(name: &str) -> SourceType {
    match name {
	"sine" => SourceType::SineSynth,
	"saw" => SourceType::LFSawSynth,
	_ => SourceType::SineSynth,	    
    }
}

pub fn map_parameter(name: &str) -> SynthParameter {
    match name {
	"freq" => SynthParameter::PitchFrequency,
	"atk" => SynthParameter::Attack,
	"rel" => SynthParameter::Release,
	"sus" => SynthParameter::Sustain,
	"pos" => SynthParameter::StereoPosition,
	"lvl" => SynthParameter::Level,	
	_ => SynthParameter::PitchFrequency,	    
    }
}
