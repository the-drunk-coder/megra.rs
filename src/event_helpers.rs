use ruffbox_synth::building_blocks::{
    FilterType, OscillatorType, SynthParameterAddress, SynthParameterLabel, SynthParameterValue,
};
use ruffbox_synth::synths::SynthType;
use std::collections::HashMap;

//
pub fn map_synth_type(
    name: &str,
    params: &HashMap<SynthParameterAddress, SynthParameterValue>,
) -> SynthType {
    match name {
        "sine" | "tri" | "sqr" | "saw" | "rsaw" | "wsaw" | "fmsqr" | "fmsaw" | "fmtri" | "cub"
        | "white" | "brown" => SynthType::SingleOscillator(
            match name {
                "sine" => OscillatorType::Sine,
                "tri" => OscillatorType::LFTri,
                "sqr" => OscillatorType::LFSquare,
                "saw" => OscillatorType::LFSaw,
                "rsaw" => OscillatorType::LFRsaw,
                "wsaw" => OscillatorType::WTSaw,
                "fmsqr" => OscillatorType::FMSquare,
                "fmsaw" => OscillatorType::FMSaw,
                "fmtri" => OscillatorType::FMTri,
                "cub" => OscillatorType::LFCub,
                "white" => OscillatorType::WhiteNoise,
                "brown" => OscillatorType::BrownNoise,
                _ => OscillatorType::Sine,
            },
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::LowpassFilterType.into())
            {
                *t
            } else {
                FilterType::Lpf18
            },
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::HighpassFilterType.into())
            {
                *t
            } else {
                FilterType::Dummy
            },
        ),
        "risset" => SynthType::RissetBell,
        "sampler" => {
            if params.contains_key(&SynthParameterLabel::AmbisonicAzimuth.into())
                || params.contains_key(&SynthParameterLabel::AmbisonicElevation.into())
            {
                SynthType::AmbisonicSampler(
                    // assemble sampler
                    if let Some(SynthParameterValue::FilterType(t)) =
                        params.get(&SynthParameterLabel::HighpassFilterType.into())
                    {
                        *t
                    } else {
                        FilterType::BiquadHpf12dB
                    },
                    if params
                        .get(&SynthParameterLabel::PeakFrequency.with_index(0))
                        .is_some()
                    {
                        FilterType::PeakEQ
                    } else {
                        FilterType::Dummy
                    },
                    if params
                        .get(&SynthParameterLabel::PeakFrequency.with_index(1))
                        .is_some()
                    {
                        FilterType::PeakEQ
                    } else {
                        FilterType::Dummy
                    },
                    if let Some(SynthParameterValue::FilterType(t)) =
                        params.get(&SynthParameterLabel::LowpassFilterType.into())
                    {
                        *t
                    } else {
                        FilterType::Lpf18
                    },
                )
            } else {
                SynthType::Sampler(
                    // assemble sampler
                    if let Some(SynthParameterValue::FilterType(t)) =
                        params.get(&SynthParameterLabel::HighpassFilterType.into())
                    {
                        *t
                    } else {
                        FilterType::BiquadHpf12dB
                    },
                    if params
                        .get(&SynthParameterLabel::PeakFrequency.with_index(0))
                        .is_some()
                    {
                        FilterType::PeakEQ
                    } else {
                        FilterType::Dummy
                    },
                    if params
                        .get(&SynthParameterLabel::PeakFrequency.with_index(1))
                        .is_some()
                    {
                        FilterType::PeakEQ
                    } else {
                        FilterType::Dummy
                    },
                    if let Some(SynthParameterValue::FilterType(t)) =
                        params.get(&SynthParameterLabel::LowpassFilterType.into())
                    {
                        *t
                    } else {
                        FilterType::Lpf18
                    },
                )
            }
        }
        "livesampler" => SynthType::LiveSampler(
            // assemble sampler
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::HighpassFilterType.into())
            {
                *t
            } else {
                FilterType::BiquadHpf12dB
            },
            if params
                .get(&SynthParameterLabel::PeakFrequency.with_index(0))
                .is_some()
            {
                FilterType::PeakEQ
            } else {
                FilterType::Dummy
            },
            if params
                .get(&SynthParameterLabel::PeakFrequency.with_index(1))
                .is_some()
            {
                FilterType::PeakEQ
            } else {
                FilterType::Dummy
            },
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::LowpassFilterType.into())
            {
                *t
            } else {
                FilterType::Lpf18
            },
        ),
        "frozensampler" => SynthType::FrozenSampler(
            // assemble sampler
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::HighpassFilterType.into())
            {
                *t
            } else {
                FilterType::BiquadHpf12dB
            },
            if params
                .get(&SynthParameterLabel::PeakFrequency.with_index(0))
                .is_some()
            {
                FilterType::PeakEQ
            } else {
                FilterType::Dummy
            },
            if params
                .get(&SynthParameterLabel::PeakFrequency.with_index(1))
                .is_some()
            {
                FilterType::PeakEQ
            } else {
                FilterType::Dummy
            },
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::LowpassFilterType.into())
            {
                *t
            } else {
                FilterType::Lpf18
            },
        ),
        "wavetable" => SynthType::SingleOscillator(
            OscillatorType::Wavetable,
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::LowpassFilterType.into())
            {
                *t
            } else {
                FilterType::Lpf18
            },
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::HighpassFilterType.into())
            {
                *t
            } else {
                FilterType::BiquadHpf12dB
            },
        ),
        "wavematrix" => SynthType::SingleOscillator(
            OscillatorType::Wavematrix,
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::LowpassFilterType.into())
            {
                *t
            } else {
                FilterType::Lpf18
            },
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::HighpassFilterType.into())
            {
                *t
            } else {
                FilterType::BiquadHpf12dB
            },
        ),
        _ => SynthType::SingleOscillator(
            OscillatorType::Sine,
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::LowpassFilterType.into())
            {
                *t
            } else {
                FilterType::Lpf18
            },
            if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::HighpassFilterType.into())
            {
                *t
            } else {
                FilterType::Dummy
            },
        ),
    }
}

pub fn map_parameter(name: &str) -> SynthParameterAddress {
    match name {
        "freq" => SynthParameterLabel::PitchFrequency.into(),
        "note" => SynthParameterLabel::PitchNote.into(),
        "atk" => SynthParameterLabel::Attack.into(),
        "atkt" => SynthParameterLabel::AttackType.into(),
        "atkp" => SynthParameterLabel::AttackPeakLevel.into(),
        "dec" => SynthParameterLabel::Decay.into(),
        "dect" => SynthParameterLabel::DecayType.into(),
        "rel" => SynthParameterLabel::Release.into(),
        "relt" => SynthParameterLabel::ReleaseType.into(),
        "sus" => SynthParameterLabel::Sustain.into(),
        "env" => SynthParameterLabel::Envelope.into(),
        "pos" => SynthParameterLabel::ChannelPosition.into(),
        "lvl" => SynthParameterLabel::EnvelopeLevel.into(),
        "amp" => SynthParameterLabel::OscillatorAmplitude.into(),
        "gain" => SynthParameterLabel::OscillatorAmplitude.into(), // a bit of a compromise.into(), for legacy reasons ...
        "dur" => SynthParameterLabel::Duration.into(),
        "lpf" => SynthParameterLabel::LowpassCutoffFrequency.into(),
        "lpd" => SynthParameterLabel::LowpassFilterDistortion.into(),
        "lpq" => SynthParameterLabel::LowpassQFactor.into(),
        "lpt" => SynthParameterLabel::LowpassFilterType.into(),
        "hpf" => SynthParameterLabel::HighpassCutoffFrequency.into(),
        "hpq" => SynthParameterLabel::HighpassQFactor.into(),
        "hpt" => SynthParameterLabel::HighpassFilterType.into(),
        "pff" => SynthParameterAddress {
            label: SynthParameterLabel::PeakFrequency,
            idx: Some(0),
        },
        "pfbw" => SynthParameterAddress {
            label: SynthParameterLabel::PeakBandwidth,
            idx: Some(0),
        },
        "pfg" => SynthParameterAddress {
            label: SynthParameterLabel::PeakGain,
            idx: Some(0),
        },
        "pff1" => SynthParameterAddress {
            label: SynthParameterLabel::PeakFrequency,
            idx: Some(0),
        },
        "pfbw1" => SynthParameterAddress {
            label: SynthParameterLabel::PeakBandwidth,
            idx: Some(0),
        },
        "pfg1" => SynthParameterAddress {
            label: SynthParameterLabel::PeakGain,
            idx: Some(0),
        },
        "pff2" => SynthParameterAddress {
            label: SynthParameterLabel::PeakFrequency,
            idx: Some(1),
        },
        "pfg2" => SynthParameterAddress {
            label: SynthParameterLabel::PeakGain,
            idx: Some(1),
        },

        "pfbw2" => SynthParameterAddress {
            label: SynthParameterLabel::PeakBandwidth,
            idx: Some(1),
        },

        "pw" => SynthParameterLabel::Pulsewidth.into(),
        "rate" => SynthParameterLabel::PlaybackRate.into(),
        "start" => SynthParameterLabel::PlaybackStart.into(),
        "loop" => SynthParameterLabel::PlaybackLoop.into(),
        "bufnum" => SynthParameterLabel::SampleBufferNumber.into(),
        "rev" => SynthParameterLabel::ReverbMix.into(),
        "del" => SynthParameterLabel::DelayMix.into(),
        "azi" => SynthParameterLabel::AmbisonicAzimuth.into(),
        "ele" => SynthParameterLabel::AmbisonicElevation.into(),
        "wt" | "wavetable" => SynthParameterLabel::Wavetable.into(),
        "wm" | "wavematrix" => SynthParameterLabel::Wavematrix.into(),
        "ti" | "tableindex" => SynthParameterLabel::WavematrixTableIndex.into(),
        "dist" => SynthParameterLabel::WaveshaperMix.into(),
        _ => SynthParameterLabel::PitchFrequency.into(),
    }
}
