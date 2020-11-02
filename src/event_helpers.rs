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
	_ => SynthParameter::PitchFrequency,	    
    }
}
