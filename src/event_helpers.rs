use ruffbox_synth::ruffbox::synth::SourceType;
use ruffbox_synth::ruffbox::synth::SynthParameterLabel;

pub fn map_name(name: &str) -> SourceType {
    match name {
        "sine" => SourceType::SineSynth,
        "tri" => SourceType::LFTriangleSynth,
        "saw" => SourceType::LFSawSynth,
        "sqr" => SourceType::LFSquareSynth,
        "cub" => SourceType::LFCubSynth,
        "risset" => SourceType::RissetBell,
        "sampler" => SourceType::Sampler,
        "livesampler" => SourceType::LiveSampler,
        "frozensampler" => SourceType::FrozenSampler,
        _ => SourceType::SineSynth,
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
        _ => SynthParameterLabel::PitchFrequency,
    }
}
