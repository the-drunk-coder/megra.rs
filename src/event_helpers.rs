use ruffbox_synth::building_blocks::bitcrusher::BitcrusherMode;
use ruffbox_synth::building_blocks::{
    EffectType, FilterType, OscillatorType, SynthParameterAddress, SynthParameterLabel,
    SynthParameterValue,
};
use ruffbox_synth::synths::{SynthDescription, SynthType};
use std::collections::HashMap;

/// generate the ruffbox synth type from available data ...
pub fn map_synth_type(
    name: &str,
    params: &HashMap<SynthParameterAddress, SynthParameterValue>,
) -> SynthType {
    // check whether we have a specified bitcrusher mode ...
    let bitcrusher_mode = if let Some(SynthParameterValue::BitcrusherMode(m)) =
        params.get(&SynthParameterLabel::BitcrusherMode.into())
    {
        *m
    } else {
        BitcrusherMode::Cast
    };

    // assemble pre-filter effects
    let mut pre_filter_effects: Vec<EffectType> = params
        .iter()
        .filter_map(|(k, _)| {
            let SynthParameterAddress { label, idx: _ } = k;
            match label {
                SynthParameterLabel::BitcrusherBits
                | SynthParameterLabel::BitcrusherMix
                | SynthParameterLabel::BitcrusherDownsampling => {
                    Some(EffectType::Bitcrusher(bitcrusher_mode))
                }
                SynthParameterLabel::WaveshaperMix => Some(EffectType::Waveshaper),
                _ => None,
            }
        })
        .collect();

    pre_filter_effects.dedup();

    match name {
        "sine" | "tri" | "sqr" | "saw" | "rsaw" | "wsaw" | "fmsqr" | "fmsaw" | "fmtri" | "cub"
        | "white" | "brown" | "blit" => SynthType::SingleOscillator(SynthDescription {
            oscillator_types: vec![match name {
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
                "blit" => OscillatorType::NaiveBlit,
                _ => OscillatorType::Sine,
            }],
            filters: vec![
                // highpass first ...
                if let Some(SynthParameterValue::FilterType(t)) =
                    params.get(&SynthParameterLabel::HighpassFilterType.into())
                {
                    *t
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
            ],
            pre_filter_effects,
        }),
        "kpp" => SynthType::KarPlusPlus(SynthDescription {
            oscillator_types: vec![if let Some(SynthParameterValue::OscillatorType(t)) =
                params.get(&SynthParameterLabel::OscillatorType.into())
            {
                *t
            } else {
                OscillatorType::WhiteNoise
            }],
            filters: vec![
                if let Some(SynthParameterValue::FilterType(t)) =
                    params.get(&SynthParameterLabel::DelayDampeningFilterType.into())
                {
                    *t
                } else {
                    FilterType::Dummy
                },
                if let Some(SynthParameterValue::FilterType(t)) =
                    params.get(&SynthParameterLabel::FilterType.into())
                {
                    *t
                } else {
                    FilterType::Dummy
                },
            ],
            pre_filter_effects,
        }),
        "risset" => SynthType::RissetBell,
        "sampler" => {
            if params.contains_key(&SynthParameterLabel::AmbisonicAzimuth.into())
                || params.contains_key(&SynthParameterLabel::AmbisonicElevation.into())
            {
                SynthType::AmbisonicSampler(SynthDescription {
                    oscillator_types: vec![],
                    filters: vec![
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
                    ],
                    pre_filter_effects,
                })
            } else {
                SynthType::Sampler(SynthDescription {
                    oscillator_types: vec![],
                    filters: vec![
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
                    ],
                    pre_filter_effects,
                })
            }
        }
        "livesampler" => SynthType::LiveSampler(SynthDescription {
            oscillator_types: vec![],
            filters: vec![
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
            ],
            pre_filter_effects,
        }),
        "frozensampler" => SynthType::FrozenSampler(SynthDescription {
            oscillator_types: vec![],
            filters: vec![
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
            ],
            pre_filter_effects,
        }),
        "wavetable" => SynthType::SingleOscillator(SynthDescription {
            oscillator_types: vec![OscillatorType::Wavetable],
            filters: vec![
                // highpass first
                if let Some(SynthParameterValue::FilterType(t)) =
                    params.get(&SynthParameterLabel::HighpassFilterType.into())
                {
                    *t
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
            ],
            pre_filter_effects,
        }),
        "wavematrix" => SynthType::SingleOscillator(SynthDescription {
            oscillator_types: vec![OscillatorType::Wavematrix],
            filters: vec![
                // highpass first
                if let Some(SynthParameterValue::FilterType(t)) =
                    params.get(&SynthParameterLabel::HighpassFilterType.into())
                {
                    *t
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
            ],
            pre_filter_effects,
        }),
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

            SynthType::MultiOscillator(SynthDescription {
                oscillator_types: osc_types,
                filters: vec![
                    // highpass first
                    if let Some(SynthParameterValue::FilterType(t)) =
                        params.get(&SynthParameterLabel::HighpassFilterType.into())
                    {
                        *t
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
                ],
                pre_filter_effects,
            })
        }
        _ => SynthType::SingleOscillator(SynthDescription {
            oscillator_types: vec![OscillatorType::Sine],
            filters: vec![
                // highpass first
                if let Some(SynthParameterValue::FilterType(t)) =
                    params.get(&SynthParameterLabel::HighpassFilterType.into())
                {
                    *t
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
            ],
            pre_filter_effects,
        }),
    }
}

pub fn map_parameter(name: &str) -> SynthParameterAddress {
    let mut id_str = "".to_string();
    let mut idx_str = "".to_string();

    // split into index and id ...
    // yes, this COULD be handled through the parser ...
    // but that's more complicated ...
    for c in name.chars() {
        if c.is_ascii_digit() {
            idx_str.push(c);
        } else {
            id_str.push(c);
        }
    }

    let label = match id_str.as_str() {
        "freq" => SynthParameterLabel::PitchFrequency,
        "osc" => SynthParameterLabel::OscillatorType,
        "note" => SynthParameterLabel::PitchNote,
        "atk" => SynthParameterLabel::Attack,
        "atkt" => SynthParameterLabel::AttackType,
        "atkp" => SynthParameterLabel::AttackPeakLevel,
        "dec" => SynthParameterLabel::Decay,
        "dect" => SynthParameterLabel::DecayType,
        "rel" => SynthParameterLabel::Release,
        "relt" => SynthParameterLabel::ReleaseType,
        "sus" => SynthParameterLabel::Sustain,
        "env" => SynthParameterLabel::Envelope,
        "pos" => SynthParameterLabel::ChannelPosition,
        "lvl" => SynthParameterLabel::EnvelopeLevel,
        "amp" => SynthParameterLabel::OscillatorAmplitude,
        "amp1" => SynthParameterLabel::OscillatorAmplitude,
        "amp2" => SynthParameterLabel::OscillatorAmplitude,
        "amp3" => SynthParameterLabel::OscillatorAmplitude,
        "amp4" => SynthParameterLabel::OscillatorAmplitude,
        "gain" => SynthParameterLabel::OscillatorAmplitude, // a bit of a compromise, for legacy reasons ...
        "dur" => SynthParameterLabel::Duration,
        "lpf" => SynthParameterLabel::LowpassCutoffFrequency,
        "lpd" => SynthParameterLabel::LowpassFilterDistortion,
        "lpq" => SynthParameterLabel::LowpassQFactor,
        "lpt" => SynthParameterLabel::LowpassFilterType,
        "hpf" => SynthParameterLabel::HighpassCutoffFrequency,
        "hpq" => SynthParameterLabel::HighpassQFactor,
        "hpt" => SynthParameterLabel::HighpassFilterType,
        "pff" => SynthParameterLabel::PeakFrequency,
        "pfbw" => SynthParameterLabel::PeakBandwidth,
        "pfg" => SynthParameterLabel::PeakGain,
        "pw" => SynthParameterLabel::Pulsewidth,
        "rate" => SynthParameterLabel::PlaybackRate,
        "start" => SynthParameterLabel::PlaybackStart,
        "loop" => SynthParameterLabel::PlaybackLoop,
        "bufnum" => SynthParameterLabel::SampleBufferNumber,
        "rev" => SynthParameterLabel::ReverbMix,
        "del" => SynthParameterLabel::DelayMix,
        "azi" => SynthParameterLabel::AmbisonicAzimuth,
        "ele" => SynthParameterLabel::AmbisonicElevation,
        "wt" | "wavetable" => SynthParameterLabel::Wavetable,
        "wm" | "wavematrix" => SynthParameterLabel::Wavematrix,
        "ti" | "tableindex" => SynthParameterLabel::WavematrixTableIndex,
        "dist" => SynthParameterLabel::WaveshaperMix,
        "delfb" => SynthParameterLabel::DelayFeedback,
        "deldf" => SynthParameterLabel::DelayDampeningFrequency,
        "delft" => SynthParameterLabel::DelayDampeningFilterType,
        "ft" => SynthParameterLabel::FilterType,
        "osct" => SynthParameterLabel::OscillatorType,
        "bcmix" => SynthParameterLabel::BitcrusherMix,
        "bcbits" => SynthParameterLabel::BitcrusherBits,
        "bcdown" => SynthParameterLabel::BitcrusherDownsampling,
        "bcmode" => SynthParameterLabel::BitcrusherMode,
        _ => SynthParameterLabel::PitchFrequency,
    };

    if let Ok(idx) = idx_str.parse::<usize>() {
        if idx > 0 {
            // we start counting at one in this case,
            // zero is the same as no index ...
            label.with_index(idx - 1)
        } else {
            label.into()
        }
    } else {
        label.into()
    }
}
