use nom::{branch::alt, bytes::complete::tag, combinator::map, error::VerboseError, IResult};

use crate::builtin_types::*;
use crate::event::EventOperation;

pub fn parse_pitch_frequency_event<'a>(
    i: &'a str,
) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("freq-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PitchFrequency(EventOperation::Add))
        }),
        map(tag("freq-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PitchFrequency(
                EventOperation::Multiply,
            ))
        }),
        map(tag("freq-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PitchFrequency(
                EventOperation::Subtract,
            ))
        }),
        map(tag("freq-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PitchFrequency(
                EventOperation::Divide,
            ))
        }),
        map(tag("freq"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PitchFrequency(
                EventOperation::Replace,
            ))
        }),
    ))(i)
}

pub fn parse_level_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("lvl-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Level(EventOperation::Add))
        }),
        map(tag("lvl-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Level(EventOperation::Multiply))
        }),
        map(tag("lvl-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Level(EventOperation::Subtract))
        }),
        map(tag("lvl-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Level(EventOperation::Divide))
        }),
        map(tag("lvl"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Level(EventOperation::Replace))
        }),
    ))(i)
}

pub fn parse_duration_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("dur-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Duration(EventOperation::Add))
        }),
        map(tag("dur-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Duration(EventOperation::Multiply))
        }),
        map(tag("dur-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Duration(EventOperation::Subtract))
        }),
        map(tag("dur-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Duration(EventOperation::Divide))
        }),
        map(tag("dur"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Duration(EventOperation::Replace))
        }),
    ))(i)
}

pub fn parse_reverb_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("rev-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Reverb(EventOperation::Add))
        }),
        map(tag("rev-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Reverb(EventOperation::Multiply))
        }),
        map(tag("rev-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Reverb(EventOperation::Subtract))
        }),
        map(tag("rev-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Reverb(EventOperation::Divide))
        }),
        map(tag("rev"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Reverb(EventOperation::Replace))
        }),
    ))(i)
}

pub fn parse_attack_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("atk"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Attack(EventOperation::Replace))
        }),
        map(tag("atk-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Attack(EventOperation::Add))
        }),
        map(tag("atk-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Attack(EventOperation::Multiply))
        }),
        map(tag("atk-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Attack(EventOperation::Subtract))
        }),
        map(tag("atk-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Attack(EventOperation::Divide))
        }),
    ))(i)
}

pub fn parse_sustain_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("sus-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Sustain(EventOperation::Add))
        }),
        map(tag("sus-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Sustain(EventOperation::Multiply))
        }),
        map(tag("sus-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Sustain(EventOperation::Subtract))
        }),
        map(tag("sus-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Sustain(EventOperation::Divide))
        }),
        map(tag("sus"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Sustain(EventOperation::Replace))
        }),
    ))(i)
}

pub fn parse_release_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("rel-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Release(EventOperation::Add))
        }),
        map(tag("rel-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Release(EventOperation::Multiply))
        }),
        map(tag("rel-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Release(EventOperation::Subtract))
        }),
        map(tag("rel-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Release(EventOperation::Divide))
        }),
        map(tag("rel"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Release(EventOperation::Replace))
        }),
    ))(i)
}

pub fn parse_channel_position_event<'a>(
    i: &'a str,
) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("pos-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::ChannelPosition(EventOperation::Add))
        }),
        map(tag("pos-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::ChannelPosition(
                EventOperation::Multiply,
            ))
        }),
        map(tag("pos-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::ChannelPosition(
                EventOperation::Subtract,
            ))
        }),
        map(tag("pos-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::ChannelPosition(
                EventOperation::Divide,
            ))
        }),
        map(tag("pos"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::ChannelPosition(
                EventOperation::Replace,
            ))
        }),
    ))(i)
}

pub fn parse_delay_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("del"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Delay(EventOperation::Replace))
        }),
        map(tag("del-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Delay(EventOperation::Add))
        }),
        map(tag("del-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Delay(EventOperation::Multiply))
        }),
        map(tag("del-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Delay(EventOperation::Subtract))
        }),
        map(tag("del-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Delay(EventOperation::Divide))
        }),
    ))(i)
}

pub fn parse_lp_freq_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("lpf-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpFreq(EventOperation::Add))
        }),
        map(tag("lpf-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpFreq(EventOperation::Multiply))
        }),
        map(tag("lpf-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpFreq(EventOperation::Subtract))
        }),
        map(tag("lpf-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpFreq(EventOperation::Divide))
        }),
        map(tag("lpf"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpFreq(EventOperation::Replace))
        }),
    ))(i)
}

pub fn parse_lp_q_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("lpq-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpQ(EventOperation::Add))
        }),
        map(tag("lpq-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpQ(EventOperation::Multiply))
        }),
        map(tag("lpq-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpQ(EventOperation::Subtract))
        }),
        map(tag("lpq-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpQ(EventOperation::Divide))
        }),
        map(tag("lpq"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpQ(EventOperation::Replace))
        }),
    ))(i)
}

pub fn parse_lp_dist_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("lpd"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpDist(EventOperation::Replace))
        }),
        map(tag("lpd-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpDist(EventOperation::Add))
        }),
        map(tag("lpd-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpDist(EventOperation::Multiply))
        }),
        map(tag("lpd-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpDist(EventOperation::Subtract))
        }),
        map(tag("lpd-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::LpDist(EventOperation::Divide))
        }),
    ))(i)
}

pub fn parse_pf_freq_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("pff-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakFreq(EventOperation::Add))
        }),
        map(tag("pff-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakFreq(EventOperation::Multiply))
        }),
        map(tag("pff-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakFreq(EventOperation::Subtract))
        }),
        map(tag("pff-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakFreq(EventOperation::Divide))
        }),
        map(tag("pff"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakFreq(EventOperation::Replace))
        }),
    ))(i)
}

pub fn parse_pf_q_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("pfq"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakQ(EventOperation::Replace))
        }),
        map(tag("pfq-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakQ(EventOperation::Add))
        }),
        map(tag("pfq-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakQ(EventOperation::Multiply))
        }),
        map(tag("pfq-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakQ(EventOperation::Subtract))
        }),
        map(tag("pfq-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakQ(EventOperation::Divide))
        }),
    ))(i)
}

pub fn parse_pf_gain_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("pfg"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakGain(EventOperation::Replace))
        }),
        map(tag("pfg-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakGain(EventOperation::Add))
        }),
        map(tag("pfg-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakGain(EventOperation::Multiply))
        }),
        map(tag("pfg-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakGain(EventOperation::Subtract))
        }),
        map(tag("pfg-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PeakGain(EventOperation::Divide))
        }),
    ))(i)
}

pub fn parse_pw_event<'a>(i: &'a str) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("pw"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Pulsewidth(EventOperation::Replace))
        }),
        map(tag("pw-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Pulsewidth(EventOperation::Add))
        }),
        map(tag("pw-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Pulsewidth(EventOperation::Multiply))
        }),
        map(tag("pw-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Pulsewidth(EventOperation::Subtract))
        }),
        map(tag("pw-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::Pulsewidth(EventOperation::Divide))
        }),
    ))(i)
}

pub fn parse_playback_start_event<'a>(
    i: &'a str,
) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("start-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PlaybackStart(EventOperation::Add))
        }),
        map(tag("start-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PlaybackStart(
                EventOperation::Multiply,
            ))
        }),
        map(tag("start-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PlaybackStart(
                EventOperation::Subtract,
            ))
        }),
        map(tag("start-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PlaybackStart(EventOperation::Divide))
        }),
        map(tag("start"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PlaybackStart(
                EventOperation::Replace,
            ))
        }),
    ))(i)
}

pub fn parse_playback_rate_event<'a>(
    i: &'a str,
) -> IResult<&'a str, BuiltIn, VerboseError<&'a str>> {
    alt((
        map(tag("rate-add"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PlaybackRate(EventOperation::Add))
        }),
        map(tag("rate-mul"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PlaybackRate(
                EventOperation::Multiply,
            ))
        }),
        map(tag("rate-sub"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PlaybackRate(
                EventOperation::Subtract,
            ))
        }),
        map(tag("rate-div"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PlaybackRate(EventOperation::Divide))
        }),
        map(tag("rate"), |_| {
            BuiltIn::ParameterEvent(BuiltInParameterEvent::PlaybackRate(EventOperation::Replace))
        }),
    ))(i)
}
