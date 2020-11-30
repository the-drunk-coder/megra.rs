use ruffbox_synth::ruffbox::synth::SourceType;
use ruffbox_synth::ruffbox::synth::SynthParameter;

pub fn map_name(name: &str) -> SourceType {
    match name {
	"sine" => SourceType::SineSynth,
	"saw" => SourceType::LFSawSynth,
	"sqr" => SourceType::LFSquareSynth,
	"sampler" => SourceType::Sampler,
	_ => SourceType::SineSynth,	    
    }
}

pub fn map_parameter(name: &str) -> SynthParameter {
    match name {
	"freq" => SynthParameter::PitchFrequency,
	"note" => SynthParameter::PitchNote,
	"atk" => SynthParameter::Attack,
	"rel" => SynthParameter::Release,
	"sus" => SynthParameter::Sustain,
	"pos" => SynthParameter::ChannelPosition,
	"lvl" => SynthParameter::Level,
	"dur" => SynthParameter::Duration,
	"lpf" => SynthParameter::LowpassCutoffFrequency,
	"lpd" => SynthParameter::LowpassFilterDistortion,
	"lpq" => SynthParameter::LowpassQFactor,
	"pf" => SynthParameter::PeakFrequency,
	"pfq" => SynthParameter::PeakQFactor,
	"pfg" => SynthParameter::PeakGain,
	"pw" => SynthParameter::Pulsewidth,
	"rate" => SynthParameter::PlaybackRate,
	"start" => SynthParameter::PlaybackStart,
	"loop" => SynthParameter::PlaybackLoop,
	"bufnum" => SynthParameter::SampleBufferNumber,
	"rev" => SynthParameter::ReverbMix,
	"del" => SynthParameter::DelayMix,
	_ => SynthParameter::PitchFrequency,	    
    }
}
