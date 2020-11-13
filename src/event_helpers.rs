use ruffbox_synth::ruffbox::synth::SourceType;
use ruffbox_synth::ruffbox::synth::SynthParameter;

pub fn map_name(name: &str) -> SourceType {
    match name {
	"sine" => SourceType::SineSynth,
	"saw" => SourceType::LFSawSynth,
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
	"pos" => SynthParameter::StereoPosition,
	"lvl" => SynthParameter::Level,
	"dur" => SynthParameter::Duration,
	"lp-freq" => SynthParameter::LowpassCutoffFrequency,
	"lp-dist" => SynthParameter::LowpassFilterDistortion,
	"lp-q" => SynthParameter::LowpassQFactor,
	"pf-freq" => SynthParameter::PeakFrequency,
	"pf-q" => SynthParameter::PeakQFactor,
	"pf-gain" => SynthParameter::PeakGain,
	"pw" => SynthParameter::Pulsewidth,
	"rate" => SynthParameter::PlaybackRate,
	"start" => SynthParameter::PlaybackStart,
	"loop" => SynthParameter::PlaybackLoop,
	"bufnum" => SynthParameter::SampleBufferNumber,
	_ => SynthParameter::PitchFrequency,	    
    }
}
