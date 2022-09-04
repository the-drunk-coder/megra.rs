use ruffbox_synth::building_blocks::{FilterType, SynthParameterLabel, SynthParameterValue};
use ruffbox_synth::synths::SynthType;
use std::collections::HashMap;

//
pub fn map_synth_type(
    name: &str,
    params: &HashMap<SynthParameterLabel, SynthParameterValue>,
) -> SynthType {
    match name {
        "sine" => SynthType::SineSynth,
        "tri" => SynthType::LFTriangleSynth,
        "saw" => SynthType::LFSawSynth,
        "wsaw" => SynthType::WTSawSynth,
        "fmsaw" => SynthType::FMSawSynth,
        "fmsqr" => SynthType::FMSquareSynth,
	"fmtri" => SynthType::FMTriSynth,
        "sqr" => SynthType::LFSquareSynth,
        "cub" => SynthType::LFCubSynth,
        "risset" => SynthType::RissetBell,
        "sampler" => SynthType::Sampler(
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::HighpassFilterType)
            {
                *t
            } else {
                FilterType::BiquadHpf12dB
            },
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::LowpassFilterType)
            {
                *t
            } else {
                FilterType::Lpf18
            },
        ),
        "livesampler" => SynthType::LiveSampler,
        "frozensampler" => SynthType::FrozenSampler,
        "wavetable" => SynthType::Wavetable,
        "wavematrix" => SynthType::Wavematrix,
        _ => SynthType::SineSynth,
    }
}

pub fn map_parameter(name: &str) -> SynthParameterLabel {
    match name {
        "freq" => SynthParameterLabel::PitchFrequency,
        "note" => SynthParameterLabel::PitchNote,
        "atk" => SynthParameterLabel::Attack,
        "rel" => SynthParameterLabel::Release,
        "sus" => SynthParameterLabel::Sustain,
        "pos" => SynthParameterLabel::ChannelPosition,
        "lvl" => SynthParameterLabel::EnvelopeLevel,
        "amp" => SynthParameterLabel::OscillatorAmplitude,
        "dur" => SynthParameterLabel::Duration,
        "lpf" => SynthParameterLabel::LowpassCutoffFrequency,
        "lpd" => SynthParameterLabel::LowpassFilterDistortion,
        "lpq" => SynthParameterLabel::LowpassQFactor,
        "lpt" => SynthParameterLabel::LowpassFilterType,
        "hpf" => SynthParameterLabel::HighpassCutoffFrequency,
        "hpq" => SynthParameterLabel::HighpassQFactor,
        "hpt" => SynthParameterLabel::HighpassFilterType,
        "pff" => SynthParameterLabel::PeakFrequency,
        "pfq" => SynthParameterLabel::PeakQFactor,
        "pfg" => SynthParameterLabel::PeakGain,
        "pw" => SynthParameterLabel::Pulsewidth,
        "rate" => SynthParameterLabel::PlaybackRate,
        "start" => SynthParameterLabel::PlaybackStart,
        "loop" => SynthParameterLabel::PlaybackLoop,
        "bufnum" => SynthParameterLabel::SampleBufferNumber,
        "rev" => SynthParameterLabel::ReverbMix,
        "del" => SynthParameterLabel::DelayMix,
        "wt" | "wavetable" => SynthParameterLabel::Wavetable,
        "wm" | "wavematrix" => SynthParameterLabel::Wavematrix,
        "ti" | "tableindex" => SynthParameterLabel::WavematrixTableIndex,
        _ => SynthParameterLabel::PitchFrequency,
    }
}
