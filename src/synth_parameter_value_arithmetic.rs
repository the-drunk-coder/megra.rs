use ruffbox_synth::building_blocks::{mod_env::SegmentInfo, SynthParameterValue};

use crate::event::EventOperation;

pub fn calc_spv(
    a: &SynthParameterValue,
    b: &SynthParameterValue,
    op: EventOperation,
) -> SynthParameterValue {
    match op {
        EventOperation::Replace => b.clone(),
        EventOperation::Add => add_spv(a, b),
        EventOperation::Subtract => subtract_spv(a, b),
        EventOperation::Multiply => multiply_spv(a, b),
        EventOperation::Divide => divide_spv(a, b),
    }
}

// only multiplication by a scalar value is allowed
// (like, multiplying an lfo by an lfo doesn't make that much sense in this context)
pub fn multiply_spv(a: &SynthParameterValue, b: &SynthParameterValue) -> SynthParameterValue {
    if let SynthParameterValue::ScalarF32(incoming) = b {
        match a {
            SynthParameterValue::ScalarF32(original) => {
                SynthParameterValue::ScalarF32(original * incoming)
            }
            SynthParameterValue::LinRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from * incoming, to * incoming, *time, *op)
            }
            SynthParameterValue::LogRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from * incoming, to * incoming, *time, *op)
            }
            SynthParameterValue::ExpRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from * incoming, to * incoming, *time, *op)
            }
            SynthParameterValue::Lfo(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::Lfo(
                    init * incoming,
                    freq.clone(),
                    phase * incoming,
                    Box::new(multiply_spv(amp, b)),
                    add * incoming,
                    *op,
                )
            }
            SynthParameterValue::LFSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFSaw(
                    init * incoming,
                    freq.clone(),
                    phase * incoming,
                    Box::new(multiply_spv(amp, b)),
                    add * incoming,
                    *op,
                )
            }
            SynthParameterValue::LFRSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFRSaw(
                    init * incoming,
                    freq.clone(),
                    phase * incoming,
                    Box::new(multiply_spv(amp, b)),
                    add * incoming,
                    *op,
                )
            }
            SynthParameterValue::LFTri(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFTri(
                    init * incoming,
                    freq.clone(),
                    phase * incoming,
                    Box::new(multiply_spv(amp, b)),
                    add * incoming,
                    *op,
                )
            }
            SynthParameterValue::MultiPointEnvelope(segments, loop_env, op) => {
                let mut seg_new = Vec::new();
                for seg in segments.iter() {
                    seg_new.push(SegmentInfo {
                        from: seg.from * incoming,
                        to: seg.to * incoming,
                        time: seg.time,
                        segment_type: seg.segment_type,
                    });
                }
                SynthParameterValue::MultiPointEnvelope(seg_new, *loop_env, *op)
            }
            _ => a.clone(),
        }
    } else if let SynthParameterValue::ScalarF32(original) = a {
        match b {
            SynthParameterValue::ScalarF32(incoming) => {
                SynthParameterValue::ScalarF32(original * incoming)
            }
            SynthParameterValue::LinRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from * original, to * original, *time, *op)
            }
            SynthParameterValue::LogRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from * original, to * original, *time, *op)
            }
            SynthParameterValue::ExpRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from * original, to * original, *time, *op)
            }
            SynthParameterValue::Lfo(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::Lfo(
                    init * original,
                    freq.clone(),
                    phase * original,
                    Box::new(multiply_spv(a, amp)),
                    add * original,
                    *op,
                )
            }
            SynthParameterValue::LFSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFSaw(
                    init * original,
                    freq.clone(),
                    phase * original,
                    Box::new(multiply_spv(a, amp)),
                    add * original,
                    *op,
                )
            }
            SynthParameterValue::LFRSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFRSaw(
                    init * original,
                    freq.clone(),
                    phase * original,
                    Box::new(multiply_spv(a, amp)),
                    add * original,
                    *op,
                )
            }
            SynthParameterValue::LFTri(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFTri(
                    init * original,
                    freq.clone(),
                    phase * original,
                    Box::new(multiply_spv(a, amp)),
                    add * original,
                    *op,
                )
            }
            SynthParameterValue::MultiPointEnvelope(segments, loop_env, op) => {
                let mut seg_new = Vec::new();
                for seg in segments.iter() {
                    seg_new.push(SegmentInfo {
                        from: seg.from * original,
                        to: seg.to * original,
                        time: seg.time,
                        segment_type: seg.segment_type,
                    });
                }
                SynthParameterValue::MultiPointEnvelope(seg_new, *loop_env, *op)
            }
            _ => a.clone(),
        }
    } else {
        a.clone()
    }
}

pub fn add_spv(a: &SynthParameterValue, b: &SynthParameterValue) -> SynthParameterValue {
    if let SynthParameterValue::ScalarF32(incoming) = b {
        match a {
            SynthParameterValue::ScalarF32(original) => {
                SynthParameterValue::ScalarF32(original + incoming)
            }
            SynthParameterValue::LinRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from + incoming, to + incoming, *time, *op)
            }
            SynthParameterValue::LogRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from + incoming, to + incoming, *time, *op)
            }
            SynthParameterValue::ExpRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from + incoming, to + incoming, *time, *op)
            }
            SynthParameterValue::Lfo(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::Lfo(
                    init + incoming,
                    freq.clone(),
                    phase + incoming,
                    Box::new(add_spv(amp, b)),
                    add + incoming,
                    *op,
                )
            }
            SynthParameterValue::LFSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFSaw(
                    init + incoming,
                    freq.clone(),
                    phase + incoming,
                    Box::new(add_spv(amp, b)),
                    add + incoming,
                    *op,
                )
            }
            SynthParameterValue::LFRSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFRSaw(
                    init + incoming,
                    freq.clone(),
                    phase + incoming,
                    Box::new(add_spv(amp, b)),
                    add + incoming,
                    *op,
                )
            }
            SynthParameterValue::LFTri(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFTri(
                    init + incoming,
                    freq.clone(),
                    phase + incoming,
                    Box::new(add_spv(amp, b)),
                    add + incoming,
                    *op,
                )
            }
            SynthParameterValue::MultiPointEnvelope(segments, loop_env, op) => {
                let mut seg_new = Vec::new();
                for seg in segments.iter() {
                    seg_new.push(SegmentInfo {
                        from: seg.from + incoming,
                        to: seg.to + incoming,
                        time: seg.time,
                        segment_type: seg.segment_type,
                    });
                }
                SynthParameterValue::MultiPointEnvelope(seg_new, *loop_env, *op)
            }
            _ => a.clone(),
        }
    } else if let SynthParameterValue::ScalarF32(original) = a {
        match b {
            SynthParameterValue::ScalarF32(incoming) => {
                SynthParameterValue::ScalarF32(original + incoming)
            }
            SynthParameterValue::LinRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from + original, to + original, *time, *op)
            }
            SynthParameterValue::LogRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from + original, to + original, *time, *op)
            }
            SynthParameterValue::ExpRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from + original, to + original, *time, *op)
            }
            SynthParameterValue::Lfo(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::Lfo(
                    init + original,
                    freq.clone(),
                    phase + original,
                    Box::new(add_spv(a, amp)),
                    add + original,
                    *op,
                )
            }
            SynthParameterValue::LFSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFSaw(
                    init + original,
                    freq.clone(),
                    phase + original,
                    Box::new(add_spv(a, amp)),
                    add + original,
                    *op,
                )
            }
            SynthParameterValue::LFRSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFRSaw(
                    init + original,
                    freq.clone(),
                    phase + original,
                    Box::new(add_spv(a, amp)),
                    add + original,
                    *op,
                )
            }
            SynthParameterValue::LFTri(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFTri(
                    init + original,
                    freq.clone(),
                    phase + original,
                    Box::new(add_spv(a, amp)),
                    add + original,
                    *op,
                )
            }
            SynthParameterValue::MultiPointEnvelope(segments, loop_env, op) => {
                let mut seg_new = Vec::new();
                for seg in segments.iter() {
                    seg_new.push(SegmentInfo {
                        from: seg.from + original,
                        to: seg.to + original,
                        time: seg.time,
                        segment_type: seg.segment_type,
                    });
                }
                SynthParameterValue::MultiPointEnvelope(seg_new, *loop_env, *op)
            }
            _ => a.clone(),
        }
    } else {
        a.clone()
    }
}

pub fn subtract_spv(a: &SynthParameterValue, b: &SynthParameterValue) -> SynthParameterValue {
    if let SynthParameterValue::ScalarF32(incoming) = b {
        match a {
            SynthParameterValue::ScalarF32(original) => {
                SynthParameterValue::ScalarF32(original - incoming)
            }
            SynthParameterValue::LinRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from - incoming, to - incoming, *time, *op)
            }
            SynthParameterValue::LogRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from - incoming, to - incoming, *time, *op)
            }
            SynthParameterValue::ExpRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from - incoming, to - incoming, *time, *op)
            }
            SynthParameterValue::Lfo(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::Lfo(
                    init - incoming,
                    freq.clone(),
                    phase - incoming,
                    Box::new(subtract_spv(amp, b)),
                    add - incoming,
                    *op,
                )
            }
            SynthParameterValue::LFSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFSaw(
                    init - incoming,
                    freq.clone(),
                    phase - incoming,
                    Box::new(subtract_spv(amp, b)),
                    add - incoming,
                    *op,
                )
            }
            SynthParameterValue::LFRSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFRSaw(
                    init - incoming,
                    freq.clone(),
                    phase - incoming,
                    Box::new(subtract_spv(amp, b)),
                    add - incoming,
                    *op,
                )
            }
            SynthParameterValue::LFTri(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFTri(
                    init - incoming,
                    freq.clone(),
                    phase - incoming,
                    Box::new(subtract_spv(amp, b)),
                    add - incoming,
                    *op,
                )
            }
            SynthParameterValue::MultiPointEnvelope(segments, loop_env, op) => {
                let mut seg_new = Vec::new();
                for seg in segments.iter() {
                    seg_new.push(SegmentInfo {
                        from: seg.from - incoming,
                        to: seg.to - incoming,
                        time: seg.time,
                        segment_type: seg.segment_type,
                    });
                }
                SynthParameterValue::MultiPointEnvelope(seg_new, *loop_env, *op)
            }
            _ => a.clone(),
        }
    } else if let SynthParameterValue::ScalarF32(original) = a {
        match b {
            SynthParameterValue::ScalarF32(incoming) => {
                SynthParameterValue::ScalarF32(original - incoming)
            }
            SynthParameterValue::LinRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from - original, to - original, *time, *op)
            }
            SynthParameterValue::LogRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from - original, to - original, *time, *op)
            }
            SynthParameterValue::ExpRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from - original, to - original, *time, *op)
            }
            SynthParameterValue::Lfo(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::Lfo(
                    init - original,
                    freq.clone(),
                    phase - original,
                    Box::new(subtract_spv(a, amp)),
                    add - original,
                    *op,
                )
            }
            SynthParameterValue::LFSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFSaw(
                    init - original,
                    freq.clone(),
                    phase - original,
                    Box::new(subtract_spv(a, amp)),
                    add - original,
                    *op,
                )
            }
            SynthParameterValue::LFRSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFRSaw(
                    init - original,
                    freq.clone(),
                    phase - original,
                    Box::new(subtract_spv(a, amp)),
                    add - original,
                    *op,
                )
            }
            SynthParameterValue::LFTri(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFTri(
                    init - original,
                    freq.clone(),
                    phase - original,
                    Box::new(subtract_spv(a, amp)),
                    add - original,
                    *op,
                )
            }
            SynthParameterValue::MultiPointEnvelope(segments, loop_env, op) => {
                let mut seg_new = Vec::new();
                for seg in segments.iter() {
                    seg_new.push(SegmentInfo {
                        from: seg.from - original,
                        to: seg.to - original,
                        time: seg.time,
                        segment_type: seg.segment_type,
                    });
                }
                SynthParameterValue::MultiPointEnvelope(seg_new, *loop_env, *op)
            }
            _ => a.clone(),
        }
    } else {
        a.clone()
    }
}

pub fn divide_spv(a: &SynthParameterValue, b: &SynthParameterValue) -> SynthParameterValue {
    if let SynthParameterValue::ScalarF32(incoming) = b {
        match a {
            SynthParameterValue::ScalarF32(original) => {
                SynthParameterValue::ScalarF32(original / incoming)
            }
            SynthParameterValue::LinRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from / incoming, to / incoming, *time, *op)
            }
            SynthParameterValue::LogRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from / incoming, to / incoming, *time, *op)
            }
            SynthParameterValue::ExpRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from / incoming, to / incoming, *time, *op)
            }
            SynthParameterValue::Lfo(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::Lfo(
                    init / incoming,
                    freq.clone(),
                    phase / incoming,
                    Box::new(divide_spv(amp, b)),
                    add / incoming,
                    *op,
                )
            }
            SynthParameterValue::LFSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFSaw(
                    init / incoming,
                    freq.clone(),
                    phase / incoming,
                    Box::new(divide_spv(amp, b)),
                    add / incoming,
                    *op,
                )
            }
            SynthParameterValue::LFRSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFRSaw(
                    init / incoming,
                    freq.clone(),
                    phase / incoming,
                    Box::new(divide_spv(amp, b)),
                    add / incoming,
                    *op,
                )
            }
            SynthParameterValue::LFTri(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFTri(
                    init / incoming,
                    freq.clone(),
                    phase / incoming,
                    Box::new(divide_spv(amp, b)),
                    add / incoming,
                    *op,
                )
            }
            SynthParameterValue::MultiPointEnvelope(segments, loop_env, op) => {
                let mut seg_new = Vec::new();
                for seg in segments.iter() {
                    seg_new.push(SegmentInfo {
                        from: seg.from / incoming,
                        to: seg.to / incoming,
                        time: seg.time,
                        segment_type: seg.segment_type,
                    });
                }
                SynthParameterValue::MultiPointEnvelope(seg_new, *loop_env, *op)
            }
            _ => a.clone(),
        }
    } else if let SynthParameterValue::ScalarF32(original) = a {
        match b {
            SynthParameterValue::ScalarF32(incoming) => {
                SynthParameterValue::ScalarF32(original / incoming)
            }
            SynthParameterValue::LinRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from / original, to / original, *time, *op)
            }
            SynthParameterValue::LogRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from / original, to / original, *time, *op)
            }
            SynthParameterValue::ExpRamp(from, to, time, op) => {
                SynthParameterValue::LinRamp(from / original, to / original, *time, *op)
            }
            SynthParameterValue::Lfo(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::Lfo(
                    init / original,
                    freq.clone(),
                    phase / original,
                    Box::new(divide_spv(a, amp)),
                    add / original,
                    *op,
                )
            }
            SynthParameterValue::LFSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFSaw(
                    init / original,
                    freq.clone(),
                    phase / original,
                    Box::new(divide_spv(a, amp)),
                    add / original,
                    *op,
                )
            }
            SynthParameterValue::LFRSaw(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFRSaw(
                    init / original,
                    freq.clone(),
                    phase / original,
                    Box::new(divide_spv(a, amp)),
                    add / original,
                    *op,
                )
            }
            SynthParameterValue::LFTri(init, freq, phase, amp, add, op) => {
                // only scale range, not frequency here ...
                SynthParameterValue::LFTri(
                    init / original,
                    freq.clone(),
                    phase / original,
                    Box::new(divide_spv(a, amp)),
                    add / original,
                    *op,
                )
            }
            SynthParameterValue::MultiPointEnvelope(segments, loop_env, op) => {
                let mut seg_new = Vec::new();
                for seg in segments.iter() {
                    seg_new.push(SegmentInfo {
                        from: seg.from / original,
                        to: seg.to / original,
                        time: seg.time,
                        segment_type: seg.segment_type,
                    });
                }
                SynthParameterValue::MultiPointEnvelope(seg_new, *loop_env, *op)
            }
            _ => a.clone(),
        }
    } else {
        a.clone()
    }
}
