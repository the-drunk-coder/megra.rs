use crate::builtin_types::{Comparable, TypedEntity};
use crate::parameter::{DynVal, ParameterValue};
use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{GlobalVariables, OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use ruffbox_synth::building_blocks::EnvelopeSegmentType;
use ruffbox_synth::building_blocks::ValOp;
use std::sync;

pub fn lin_ramp_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut from = DynVal::with_value(0.01);
    let mut to = DynVal::with_value(0.5);
    let mut time = DynVal::with_value(0.2);
    let mut op = ValOp::Replace;

    if let Some(f) = tail_drain.next() {
        from = match f {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(fl))) => {
                DynVal::with_value(fl)
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => p,
            _ => from,
        };
    }

    if let Some(t) = tail_drain.next() {
        to = match t {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                DynVal::with_value(f)
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => p,
            _ => to,
        };
    }

    if let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        match k.as_str() {
            "time" | "t" => {
                if let Some(p) = tail_drain.next() {
                    time = match p {
                        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                            DynVal::with_value(f)
                        }
                        EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => p,
                        _ => time,
                    };
                }
            }
            "op" => {
                if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) =
                    tail_drain.next()
                {
                    match s.as_str() {
                        "add" => op = ValOp::Add,
                        "sub" => op = ValOp::Subtract,
                        "div" => op = ValOp::Divide,
                        "mul" => op = ValOp::Multiply,
                        _ => op = ValOp::Replace,
                    }
                }
            }
            _ => {}
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(
        ParameterValue::LinRamp(from, to, time, op),
    )))
}

pub fn log_ramp_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut from = DynVal::with_value(0.01);
    let mut to = DynVal::with_value(0.5);
    let mut time = DynVal::with_value(0.2);
    let mut op = ValOp::Replace;

    if let Some(f) = tail_drain.next() {
        from = match f {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(fl))) => {
                DynVal::with_value(fl)
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => p,
            _ => from,
        };
    }

    if let Some(t) = tail_drain.next() {
        to = match t {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                DynVal::with_value(f)
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => p,
            _ => to,
        };
    }

    if let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        match k.as_str() {
            "time" | "t" => {
                if let Some(p) = tail_drain.next() {
                    time = match p {
                        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                            DynVal::with_value(f)
                        }
                        EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => p,
                        _ => time,
                    };
                }
            }
            "op" => {
                if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) =
                    tail_drain.next()
                {
                    match s.as_str() {
                        "add" => op = ValOp::Add,
                        "sub" => op = ValOp::Subtract,
                        "div" => op = ValOp::Divide,
                        "mul" => op = ValOp::Multiply,
                        _ => op = ValOp::Replace,
                    }
                }
            }
            _ => {}
        }
    }
    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(
        ParameterValue::LogRamp(from, to, time, op),
    )))
}

pub fn exp_ramp_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut from = DynVal::with_value(0.01);
    let mut to = DynVal::with_value(0.5);
    let mut time = DynVal::with_value(0.2);
    let mut op = ValOp::Replace;

    if let Some(f) = tail_drain.next() {
        from = match f {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(fl))) => {
                DynVal::with_value(fl)
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => p,
            _ => from,
        };
    }

    if let Some(t) = tail_drain.next() {
        to = match t {
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                DynVal::with_value(f)
            }
            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => p,
            _ => to,
        };
    }

    if let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        match k.as_str() {
            "time" | "t" => {
                if let Some(p) = tail_drain.next() {
                    time = match p {
                        EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                            DynVal::with_value(f)
                        }
                        EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => p,
                        _ => time,
                    };
                }
            }
            "op" => {
                if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) =
                    tail_drain.next()
                {
                    match s.as_str() {
                        "add" => op = ValOp::Add,
                        "sub" => op = ValOp::Subtract,
                        "div" => op = ValOp::Divide,
                        "mul" => op = ValOp::Multiply,
                        _ => op = ValOp::Replace,
                    }
                }
            }
            _ => {}
        }
    }
    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(
        ParameterValue::ExpRamp(from, to, time, op),
    )))
}

pub fn multi_point_envelope_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut levels = Vec::new();
    let mut times = Vec::new();
    let mut types = Vec::new();
    let mut op = ValOp::Replace;
    let mut loop_env = false;

    let mut collect_levels = false;
    let mut collect_times = false;
    let mut collect_segment_types = false;

    while let Some(c) = tail_drain.next() {
        if collect_levels {
            match c {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                    levels.push(DynVal::with_value(f))
                }
                EvaluatedExpr::Typed(TypedEntity::Parameter(ref p)) => levels.push(p.clone()),
                _ => {
                    collect_levels = false;
                }
            }
        }
        if collect_times {
            match c {
                EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                    times.push(DynVal::with_value(f))
                }
                EvaluatedExpr::Typed(TypedEntity::Parameter(ref p)) => times.push(p.clone()),
                _ => {
                    collect_times = false;
                }
            }
        }

        if collect_segment_types {
            if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(ref s))) = c {
                types.push(match s.as_str() {
                    "lin" => EnvelopeSegmentType::Lin,
                    "log" => EnvelopeSegmentType::Log,
                    "exp" => EnvelopeSegmentType::Exp,
                    _ => EnvelopeSegmentType::Lin,
                });
            } else {
                collect_segment_types = false;
            }
        }

        if let EvaluatedExpr::Keyword(k) = c {
            match k.as_str() {
                "times" | "t" => {
                    collect_times = true;
                }
                "levels" | "l" => {
                    collect_levels = true;
                }
                "types" | "ty" => {
                    collect_segment_types = true;
                }
                "loop" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Boolean(b),
                    ))) = tail_drain.next()
                    {
                        loop_env = b;
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Symbol(s),
                    ))) = tail_drain.next()
                    {
                        match s.as_str() {
                            "add" => op = ValOp::Add,
                            "sub" => op = ValOp::Subtract,
                            "div" => op = ValOp::Divide,
                            "mul" => op = ValOp::Multiply,
                            _ => op = ValOp::Replace,
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(
        ParameterValue::MultiPointEnvelope(levels, times, types, loop_env, op),
    )))
}

pub fn lfo_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = DynVal::with_value(1.0);
    let mut freq = ParameterValue::Scalar(DynVal::with_value(1.0));
    let mut eff_phase = DynVal::with_value(0.0);
    let mut amp = ParameterValue::Scalar(DynVal::with_value(1.0));
    let mut add = DynVal::with_value(0.0);
    let mut op = ValOp::Replace;

    // make sure range/phase calc is always consistent
    let mut phase_set = false;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                init = DynVal::with_value(f)
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                freq = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                freq = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => freq = m,
                            _ => {}
                        }
                    }
                }
                "phase" | "p" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                eff_phase = DynVal::with_value(f);
                                phase_set = true;
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                eff_phase = p;
                                phase_set = true;
                            }
                            _ => {}
                        }
                    }
                }
                "range" | "r" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        f1,
                    )))) = tail_drain.next()
                    {
                        let a = f1;
                        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                            Comparable::Float(f2),
                        ))) = tail_drain.next()
                        {
                            let b = f2;
                            let lamp = (a - b).abs() / 2.0;
                            let ladd = f32::min(a, b) + lamp;

                            // don't overwrite phase if it has been set
                            if !phase_set {
                                eff_phase = DynVal::with_value(a);
                            }

                            //println!("{} {} {} {}", a, b, lamp, ladd);
                            amp = ParameterValue::Scalar(DynVal::with_value(lamp));
                            add = DynVal::with_value(ladd);
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                amp = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                amp = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => amp = m,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                add = DynVal::with_value(f)
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => add = p,
                            _ => {}
                        }
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Symbol(s),
                    ))) = tail_drain.next()
                    {
                        match s.as_str() {
                            "add" => op = ValOp::Add,
                            "sub" => op = ValOp::Subtract,
                            "div" => op = ValOp::Divide,
                            "mul" => op = ValOp::Multiply,
                            _ => op = ValOp::Replace,
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(
        ParameterValue::Lfo(init, Box::new(freq), eff_phase, Box::new(amp), add, op),
    )))
}

pub fn lfsaw_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = DynVal::with_value(1.0);
    let mut freq = ParameterValue::Scalar(DynVal::with_value(1.0));
    let mut eff_phase = DynVal::with_value(-1.0);
    let mut amp = ParameterValue::Scalar(DynVal::with_value(1.0));
    let mut add = DynVal::with_value(0.0);
    let mut op = ValOp::Replace;

    // make sure range/phase calc is always consistent
    let mut phase_set = false;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                init = DynVal::with_value(f)
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                freq = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                freq = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => freq = m,
                            _ => {}
                        }
                    }
                }
                "phase" | "p" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                eff_phase = DynVal::with_value(f);
                                phase_set = true;
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                eff_phase = p;
                                phase_set = true;
                            }
                            _ => {}
                        }
                    }
                }
                "range" | "r" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        f1,
                    )))) = tail_drain.next()
                    {
                        let a = f1;
                        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                            Comparable::Float(f2),
                        ))) = tail_drain.next()
                        {
                            let b = f2;
                            let lamp = (a - b).abs() / 2.0;
                            let ladd = f32::min(a, b) + lamp;

                            // don't overwrite phase if it has been set
                            if !phase_set {
                                eff_phase = DynVal::with_value(a);
                            }

                            amp = ParameterValue::Scalar(DynVal::with_value(lamp));
                            add = DynVal::with_value(ladd);
                            //range = (a,b);
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                amp = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                amp = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => amp = m,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                amp = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                amp = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => amp = m,
                            _ => {}
                        }
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Symbol(s),
                    ))) = tail_drain.next()
                    {
                        match s.as_str() {
                            "add" => op = ValOp::Add,
                            "sub" => op = ValOp::Subtract,
                            "div" => op = ValOp::Divide,
                            "mul" => op = ValOp::Multiply,
                            _ => op = ValOp::Replace,
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(
        ParameterValue::LFSaw(init, Box::new(freq), eff_phase, Box::new(amp), add, op),
    )))
}

pub fn lfrsaw_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = DynVal::with_value(1.0);
    let mut freq = ParameterValue::Scalar(DynVal::with_value(1.0));
    let mut eff_phase = DynVal::with_value(-1.0);
    let mut amp = ParameterValue::Scalar(DynVal::with_value(1.0));
    let mut add = DynVal::with_value(0.0);
    let mut op = ValOp::Replace;

    // make sure range/phase calc is always consistent
    let mut phase_set = false;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                init = DynVal::with_value(f)
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                freq = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                freq = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => freq = m,
                            _ => {}
                        }
                    }
                }
                "phase" | "p" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                eff_phase = DynVal::with_value(f);
                                phase_set = true;
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                eff_phase = p;
                                phase_set = true;
                            }
                            _ => {}
                        }
                    }
                }
                "range" | "r" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        f1,
                    )))) = tail_drain.next()
                    {
                        let a = f1;
                        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                            Comparable::Float(f2),
                        ))) = tail_drain.next()
                        {
                            let b = f2;
                            let lamp = (a - b).abs() / 2.0;
                            let ladd = f32::min(a, b) + lamp;

                            // don't overwrite phase if it has been set
                            if !phase_set {
                                eff_phase = DynVal::with_value(a);
                            }

                            amp = ParameterValue::Scalar(DynVal::with_value(lamp));
                            add = DynVal::with_value(ladd);
                            //range = (a,b);
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                amp = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                amp = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => amp = m,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                add = DynVal::with_value(f)
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => add = p,
                            _ => {}
                        }
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Symbol(s),
                    ))) = tail_drain.next()
                    {
                        match s.as_str() {
                            "add" => op = ValOp::Add,
                            "sub" => op = ValOp::Subtract,
                            "div" => op = ValOp::Divide,
                            "mul" => op = ValOp::Multiply,
                            _ => op = ValOp::Replace,
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(
        ParameterValue::LFRSaw(init, Box::new(freq), eff_phase, Box::new(amp), add, op),
    )))
}

pub fn lftri_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = DynVal::with_value(1.0);
    let mut freq = ParameterValue::Scalar(DynVal::with_value(1.0));
    let mut eff_phase = DynVal::with_value(0.0);
    let mut amp = ParameterValue::Scalar(DynVal::with_value(1.0));
    let mut add = DynVal::with_value(0.0);
    let mut op = ValOp::Replace;

    // make sure range/phase calc is always consistent
    let mut phase_set = false;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                init = DynVal::with_value(f)
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                freq = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                freq = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => freq = m,
                            _ => {}
                        }
                    }
                }
                "phase" | "p" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                eff_phase = DynVal::with_value(f);
                                phase_set = true;
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                eff_phase = p;
                                phase_set = true;
                            }
                            _ => {}
                        }
                    }
                }
                "range" | "r" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        f1,
                    )))) = tail_drain.next()
                    {
                        let a = f1;
                        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                            Comparable::Float(f2),
                        ))) = tail_drain.next()
                        {
                            let b = f2;
                            let lamp = (a - b).abs() / 2.0;
                            let ladd = f32::min(a, b) + lamp;

                            // don't overwrite phase if it has been set
                            if !phase_set {
                                eff_phase = DynVal::with_value(a);
                            }

                            amp = ParameterValue::Scalar(DynVal::with_value(lamp));
                            add = DynVal::with_value(ladd);
                            //range = (a,b);
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                amp = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                amp = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => amp = m,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                amp = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                amp = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => amp = m,
                            _ => {}
                        }
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Symbol(s),
                    ))) = tail_drain.next()
                    {
                        match s.as_str() {
                            "add" => op = ValOp::Add,
                            "sub" => op = ValOp::Subtract,
                            "div" => op = ValOp::Divide,
                            "mul" => op = ValOp::Multiply,
                            _ => op = ValOp::Replace,
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(
        ParameterValue::LFTri(init, Box::new(freq), eff_phase, Box::new(amp), add, op),
    )))
}

pub fn lfsquare_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = DynVal::with_value(1.0);
    let mut freq = ParameterValue::Scalar(DynVal::with_value(1.0));
    let mut amp = ParameterValue::Scalar(DynVal::with_value(1.0));
    let mut add = DynVal::with_value(0.0);
    let mut pw = DynVal::with_value(0.5);
    let mut op = ValOp::Replace;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                init = DynVal::with_value(f)
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                freq = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                freq = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => freq = m,
                            _ => {}
                        }
                    }
                }
                "range" | "r" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        f1,
                    )))) = tail_drain.next()
                    {
                        let a = f1;
                        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                            Comparable::Float(f2),
                        ))) = tail_drain.next()
                        {
                            let b = f2;
                            let lamp = (a - b).abs() / 2.0;
                            let ladd = f32::min(a, b) + lamp;

                            amp = ParameterValue::Scalar(DynVal::with_value(lamp));
                            add = DynVal::with_value(ladd);
                        }
                    }
                }
                "pw" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(p))) => {
                                pw = DynVal::with_value(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => pw = p,
                            _ => {}
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                amp = ParameterValue::Scalar(DynVal::with_value(f))
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => {
                                amp = ParameterValue::Scalar(p)
                            }
                            EvaluatedExpr::Typed(TypedEntity::ParameterValue(m)) => amp = m,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                                add = DynVal::with_value(f)
                            }
                            EvaluatedExpr::Typed(TypedEntity::Parameter(p)) => add = p,
                            _ => {}
                        }
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Symbol(s),
                    ))) = tail_drain.next()
                    {
                        match s.as_str() {
                            "add" => op = ValOp::Add,
                            "sub" => op = ValOp::Subtract,
                            "div" => op = ValOp::Divide,
                            "mul" => op = ValOp::Multiply,
                            _ => op = ValOp::Replace,
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(
        ParameterValue::LFSquare(init, Box::new(freq), pw, Box::new(amp), add, op),
    )))
}
