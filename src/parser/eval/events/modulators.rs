use crate::parameter::{Parameter, ParameterValue};
use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{GlobalParameters, OutputMode, SampleAndWavematrixSet};
use parking_lot::Mutex;
use ruffbox_synth::building_blocks::mod_env::SegmentType;
use ruffbox_synth::building_blocks::ValOp;
use std::sync;

pub fn lin_ramp_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut from = Parameter::with_value(0.01);
    let mut to = Parameter::with_value(0.5);
    let mut time = Parameter::with_value(0.2);
    let mut op = ValOp::Replace;

    if let Some(f) = tail_drain.next() {
        from = match f {
            EvaluatedExpr::Float(fl) => Parameter::with_value(fl),
            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => p,
            _ => from,
        };
    }

    if let Some(t) = tail_drain.next() {
        to = match t {
            EvaluatedExpr::Float(f) => Parameter::with_value(f),
            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => p,
            _ => to,
        };
    }

    if let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        match k.as_str() {
            "time" | "t" => {
                if let Some(p) = tail_drain.next() {
                    time = match p {
                        EvaluatedExpr::Float(f) => Parameter::with_value(f),
                        EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => p,
                        _ => time,
                    };
                }
            }
            "op" => {
                if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
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
    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(
        ParameterValue::LinRamp(from, to, time, op),
    )))
}

pub fn log_ramp_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut from = Parameter::with_value(0.01);
    let mut to = Parameter::with_value(0.5);
    let mut time = Parameter::with_value(0.2);
    let mut op = ValOp::Replace;

    if let Some(f) = tail_drain.next() {
        from = match f {
            EvaluatedExpr::Float(fl) => Parameter::with_value(fl),
            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => p,
            _ => from,
        };
    }

    if let Some(t) = tail_drain.next() {
        to = match t {
            EvaluatedExpr::Float(f) => Parameter::with_value(f),
            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => p,
            _ => to,
        };
    }

    if let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        match k.as_str() {
            "time" | "t" => {
                if let Some(p) = tail_drain.next() {
                    time = match p {
                        EvaluatedExpr::Float(f) => Parameter::with_value(f),
                        EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => p,
                        _ => time,
                    };
                }
            }
            "op" => {
                if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
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
    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(
        ParameterValue::LogRamp(from, to, time, op),
    )))
}

pub fn exp_ramp_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut from = Parameter::with_value(0.01);
    let mut to = Parameter::with_value(0.5);
    let mut time = Parameter::with_value(0.2);
    let mut op = ValOp::Replace;

    if let Some(f) = tail_drain.next() {
        from = match f {
            EvaluatedExpr::Float(fl) => Parameter::with_value(fl),
            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => p,
            _ => from,
        };
    }

    if let Some(t) = tail_drain.next() {
        to = match t {
            EvaluatedExpr::Float(f) => Parameter::with_value(f),
            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => p,
            _ => to,
        };
    }

    if let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        match k.as_str() {
            "time" | "t" => {
                if let Some(p) = tail_drain.next() {
                    time = match p {
                        EvaluatedExpr::Float(f) => Parameter::with_value(f),
                        EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => p,
                        _ => time,
                    };
                }
            }
            "op" => {
                if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
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
    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(
        ParameterValue::ExpRamp(from, to, time, op),
    )))
}

pub fn multi_point_envelope_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
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
                EvaluatedExpr::Float(f) => levels.push(Parameter::with_value(f)),
                EvaluatedExpr::BuiltIn(BuiltIn::Parameter(ref p)) => levels.push(p.clone()),
                _ => {
                    collect_levels = false;
                }
            }
        }
        if collect_times {
            match c {
                EvaluatedExpr::Float(f) => times.push(Parameter::with_value(f)),
                EvaluatedExpr::BuiltIn(BuiltIn::Parameter(ref p)) => times.push(p.clone()),
                _ => {
                    collect_times = false;
                }
            }
        }

        if collect_segment_types {
            if let EvaluatedExpr::Symbol(ref s) = c {
                types.push(match s.as_str() {
                    "lin" => SegmentType::Lin,
                    "log" => SegmentType::Log,
                    "exp" => SegmentType::Exp,
                    _ => SegmentType::Lin,
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
                    if let Some(EvaluatedExpr::Boolean(b)) = tail_drain.next() {
                        loop_env = b;
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
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

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(
        ParameterValue::MultiPointEnvelope(levels, times, types, loop_env, op),
    )))
}

pub fn lfo_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = Parameter::with_value(1.0);
    let mut freq = Parameter::with_value(1.0);
    let mut eff_phase = Parameter::with_value(0.0);
    let mut amp = Parameter::with_value(1.0);
    let mut add = Parameter::with_value(0.0);
    let mut op = ValOp::Replace;

    // make sure range/phase calc is always consistent
    let mut phase_set = false;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => init = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => freq = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => freq = p,
                            _ => {}
                        }
                    }
                }
                "phase" | "p" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => {
                                eff_phase = Parameter::with_value(f);
                                phase_set = true;
                            }
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => {
                                eff_phase = p;
                                phase_set = true;
                            }
                            _ => {}
                        }
                    }
                }
                "range" | "r" => {
                    if let Some(EvaluatedExpr::Float(f1)) = tail_drain.next() {
                        let a = f1;
                        if let Some(EvaluatedExpr::Float(f2)) = tail_drain.next() {
                            let b = f2;
                            let lamp = (a - b).abs() / 2.0;
                            let ladd = f32::min(a, b) + lamp;

                            // don't overwrite phase if it has been set
                            if !phase_set {
                                eff_phase = Parameter::with_value(a);
                            }

                            //println!("{} {} {} {}", a, b, lamp, ladd);
                            amp = Parameter::with_value(lamp);
                            add = Parameter::with_value(ladd);
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => amp = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => amp = p,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => add = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => add = p,
                            _ => {}
                        }
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
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

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(
        ParameterValue::Lfo(init, freq, eff_phase, amp, add, op),
    )))
}

pub fn lfsaw_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = Parameter::with_value(1.0);
    let mut freq = Parameter::with_value(1.0);
    let mut eff_phase = Parameter::with_value(-1.0);
    let mut amp = Parameter::with_value(1.0);
    let mut add = Parameter::with_value(0.0);
    let mut op = ValOp::Replace;

    // make sure range/phase calc is always consistent
    let mut phase_set = false;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => init = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => freq = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => freq = p,
                            _ => {}
                        }
                    }
                }
                "phase" | "p" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => {
                                eff_phase = Parameter::with_value(f);
                                phase_set = true;
                            }
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => {
                                eff_phase = p;
                                phase_set = true;
                            }
                            _ => {}
                        }
                    }
                }
                "range" | "r" => {
                    if let Some(EvaluatedExpr::Float(f1)) = tail_drain.next() {
                        let a = f1;
                        if let Some(EvaluatedExpr::Float(f2)) = tail_drain.next() {
                            let b = f2;
                            let lamp = (a - b).abs() / 2.0;
                            let ladd = f32::min(a, b) + lamp;

                            // don't overwrite phase if it has been set
                            if !phase_set {
                                eff_phase = Parameter::with_value(a);
                            }

                            amp = Parameter::with_value(lamp);
                            add = Parameter::with_value(ladd);
                            //range = (a,b);
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => amp = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => amp = p,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => add = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => add = p,
                            _ => {}
                        }
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
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

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(
        ParameterValue::LFSaw(init, freq, eff_phase, amp, add, op),
    )))
}

pub fn lfrsaw_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = Parameter::with_value(1.0);
    let mut freq = Parameter::with_value(1.0);
    let mut eff_phase = Parameter::with_value(-1.0);
    let mut amp = Parameter::with_value(1.0);
    let mut add = Parameter::with_value(0.0);
    let mut op = ValOp::Replace;

    // make sure range/phase calc is always consistent
    let mut phase_set = false;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => init = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => freq = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => freq = p,
                            _ => {}
                        }
                    }
                }
                "phase" | "p" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => {
                                eff_phase = Parameter::with_value(f);
                                phase_set = true;
                            }
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => {
                                eff_phase = p;
                                phase_set = true;
                            }
                            _ => {}
                        }
                    }
                }
                "range" | "r" => {
                    if let Some(EvaluatedExpr::Float(f1)) = tail_drain.next() {
                        let a = f1;
                        if let Some(EvaluatedExpr::Float(f2)) = tail_drain.next() {
                            let b = f2;
                            let lamp = (a - b).abs() / 2.0;
                            let ladd = f32::min(a, b) + lamp;

                            // don't overwrite phase if it has been set
                            if !phase_set {
                                eff_phase = Parameter::with_value(a);
                            }

                            amp = Parameter::with_value(lamp);
                            add = Parameter::with_value(ladd);
                            //range = (a,b);
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => amp = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => amp = p,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => add = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => add = p,
                            _ => {}
                        }
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
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

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(
        ParameterValue::LFRSaw(init, freq, eff_phase, amp, add, op),
    )))
}

pub fn lftri_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = Parameter::with_value(1.0);
    let mut freq = Parameter::with_value(1.0);
    let mut eff_phase = Parameter::with_value(0.0);
    let mut amp = Parameter::with_value(1.0);
    let mut add = Parameter::with_value(0.0);
    let mut op = ValOp::Replace;

    // make sure range/phase calc is always consistent
    let mut phase_set = false;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => init = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => freq = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => freq = p,
                            _ => {}
                        }
                    }
                }
                "phase" | "p" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => {
                                eff_phase = Parameter::with_value(f);
                                phase_set = true;
                            }
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => {
                                eff_phase = p;
                                phase_set = true;
                            }
                            _ => {}
                        }
                    }
                }
                "range" | "r" => {
                    if let Some(EvaluatedExpr::Float(f1)) = tail_drain.next() {
                        let a = f1;
                        if let Some(EvaluatedExpr::Float(f2)) = tail_drain.next() {
                            let b = f2;
                            let lamp = (a - b).abs() / 2.0;
                            let ladd = f32::min(a, b) + lamp;

                            // don't overwrite phase if it has been set
                            if !phase_set {
                                eff_phase = Parameter::with_value(a);
                            }

                            amp = Parameter::with_value(lamp);
                            add = Parameter::with_value(ladd);
                            //range = (a,b);
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => amp = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => amp = p,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => add = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => add = p,
                            _ => {}
                        }
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
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

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(
        ParameterValue::LFTri(init, freq, eff_phase, amp, add, op),
    )))
}

pub fn lfsquare_modulator(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleAndWavematrixSet>>,
    _: OutputMode,
) -> Option<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut init = Parameter::with_value(1.0);
    let mut freq = Parameter::with_value(1.0);
    let mut amp = Parameter::with_value(1.0);
    let mut add = Parameter::with_value(0.0);
    let mut pw = Parameter::with_value(0.5);
    let mut op = ValOp::Replace;

    while let Some(e) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = e {
            match k.as_str() {
                "init" | "i" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => init = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => init = p,
                            _ => {}
                        }
                    }
                }
                "freq" | "f" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => freq = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => freq = p,
                            _ => {}
                        }
                    }
                }
                "pw" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(p) => pw = Parameter::with_value(p),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => pw = p,
                            _ => {}
                        }
                    }
                }
                "amp" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => amp = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => amp = p,
                            _ => {}
                        }
                    }
                }
                "add" => {
                    if let Some(p) = tail_drain.next() {
                        match p {
                            EvaluatedExpr::Float(f) => add = Parameter::with_value(f),
                            EvaluatedExpr::BuiltIn(BuiltIn::Parameter(p)) => add = p,
                            _ => {}
                        }
                    }
                }
                "op" => {
                    if let Some(EvaluatedExpr::Symbol(s)) = tail_drain.next() {
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

    Some(EvaluatedExpr::BuiltIn(BuiltIn::Modulator(
        ParameterValue::LFSquare(init, freq, pw, amp, add, op),
    )))
}
