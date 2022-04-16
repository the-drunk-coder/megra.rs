use ruffbox_synth::building_blocks::SynthParameterLabel;
use ruffbox_synth::building_blocks::SynthType;

pub fn map_name(name: &str) -> SynthType {
    match name {
        "sine" => SynthType::SineSynth,
        "tri" => SynthType::LFTriangleSynth,
        "saw" => SynthType::LFSawSynth,
        "sqr" => SynthType::LFSquareSynth,
        "cub" => SynthType::LFCubSynth,
        "risset" => SynthType::RissetBell,
        "sampler" => SynthType::Sampler,
        "livesampler" => SynthType::LiveSampler,
        "frozensampler" => SynthType::FrozenSampler,
        "wavetable" => SynthType::Wavetable,
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
        "lvl" => SynthParameterLabel::Level,
        "dur" => SynthParameterLabel::Duration,
        "lpf" => SynthParameterLabel::LowpassCutoffFrequency,
        "lpd" => SynthParameterLabel::LowpassFilterDistortion,
        "lpq" => SynthParameterLabel::LowpassQFactor,
        "hpf" => SynthParameterLabel::HighpassCutoffFrequency,
        "hpq" => SynthParameterLabel::HighpassQFactor,
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
        "wt" => SynthParameterLabel::Wavetable,
        _ => SynthParameterLabel::PitchFrequency,
    }
}
