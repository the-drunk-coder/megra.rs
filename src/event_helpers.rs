use ruffbox_synth::building_blocks::{
    FilterType, OscillatorType, SynthParameterAddress, SynthParameterLabel, SynthParameterValue,
};
use ruffbox_synth::synths::SynthType;
use std::collections::HashMap;

/// generate the ruffbox synth type from available data ...
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
        "mosc" => {
            let mut osc_types = Vec::new();
            if let Some(SynthParameterValue::OscillatorType(o)) =
                params.get(&SynthParameterLabel::OscillatorType.with_index(0))
            {
                if osc_types.is_empty() {
                    osc_types.push(*o);
                } else {
                    osc_types[0] = *o;
                }
            }
            if let Some(SynthParameterValue::OscillatorType(o)) =
                params.get(&SynthParameterLabel::OscillatorType.with_index(1))
            {
                if osc_types.len() < 2 {
                    for _ in osc_types.len()..2 {
                        osc_types.push(OscillatorType::Sine);
                    }
                }
                osc_types[1] = *o;
            }
            if let Some(SynthParameterValue::OscillatorType(o)) =
                params.get(&SynthParameterLabel::OscillatorType.with_index(2))
            {
                if osc_types.len() < 3 {
                    for _ in osc_types.len()..3 {
                        osc_types.push(OscillatorType::Sine);
                    }
                }
                osc_types[2] = *o;
            }
            if let Some(SynthParameterValue::OscillatorType(o)) =
                params.get(&SynthParameterLabel::OscillatorType.with_index(3))
            {
                if osc_types.len() < 4 {
                    for _ in osc_types.len()..4 {
                        osc_types.push(OscillatorType::Sine);
                    }
                }
                osc_types[3] = *o;
            }
            let lp_type = if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::LowpassFilterType.into())
            {
                *t
            } else {
                FilterType::Lpf18
            };
            let hp_type = if let Some(SynthParameterValue::FilterType(t)) =
                params.get(&SynthParameterLabel::HighpassFilterType.into())
            {
                *t
            } else {
                FilterType::BiquadHpf12dB
            };
            SynthType::MultiOscillator(osc_types, lp_type, hp_type)
        }
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
        "freq1" => SynthParameterLabel::PitchFrequency.with_index(0),
        "freq2" => SynthParameterLabel::PitchFrequency.with_index(1),
        "freq3" => SynthParameterLabel::PitchFrequency.with_index(2),
        "freq4" => SynthParameterLabel::PitchFrequency.with_index(3),
        "osc1" => SynthParameterLabel::OscillatorType.with_index(0),
        "osc2" => SynthParameterLabel::OscillatorType.with_index(1),
        "osc3" => SynthParameterLabel::OscillatorType.with_index(2),
        "osc4" => SynthParameterLabel::OscillatorType.with_index(3),
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
        "amp1" => SynthParameterLabel::OscillatorAmplitude.with_index(0),
        "amp2" => SynthParameterLabel::OscillatorAmplitude.with_index(1),
        "amp3" => SynthParameterLabel::OscillatorAmplitude.with_index(2),
        "amp4" => SynthParameterLabel::OscillatorAmplitude.with_index(3),
        "gain" => SynthParameterLabel::OscillatorAmplitude.into(), // a bit of a compromise.into(), for legacy reasons ...
        "dur" => SynthParameterLabel::Duration.into(),
        "lpf" => SynthParameterLabel::LowpassCutoffFrequency.into(),
        "lpd" => SynthParameterLabel::LowpassFilterDistortion.into(),
        "lpq" => SynthParameterLabel::LowpassQFactor.into(),
        "lpt" => SynthParameterLabel::LowpassFilterType.into(),
        "hpf" => SynthParameterLabel::HighpassCutoffFrequency.into(),
        "hpq" => SynthParameterLabel::HighpassQFactor.into(),
        "hpt" => SynthParameterLabel::HighpassFilterType.into(),
        "pff" | "pff1" => SynthParameterLabel::PeakFrequency.with_index(0),
        "pfbw" | "pfbw1" => SynthParameterLabel::PeakBandwidth.with_index(0),
        "pfg" | "pfg1" => SynthParameterLabel::PeakGain.with_index(0),
        "pff2" => SynthParameterLabel::PeakFrequency.with_index(1),
        "pfbw2" => SynthParameterLabel::PeakBandwidth.with_index(1),
        "pfg2" => SynthParameterLabel::PeakGain.with_index(1),
        "pw" => SynthParameterLabel::Pulsewidth.into(),
        "pw1" => SynthParameterLabel::Pulsewidth.with_index(0),
        "pw2" => SynthParameterLabel::Pulsewidth.with_index(1),
        "pw3" => SynthParameterLabel::Pulsewidth.with_index(2),
        "pw4" => SynthParameterLabel::Pulsewidth.with_index(3),
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
