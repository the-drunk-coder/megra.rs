use std::collections::HashMap;

use crate::builtin_types::*;
use crate::parameter::*;

use std::collections::BTreeSet;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use ruffbox_synth::building_blocks::SynthParameterLabel;

use crate::eval::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

use std::sync;

use super::resolver::resolve_globals;

pub fn import_sample_set(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1).peekable();

    let mut url: Option<String> = None;
    let mut file: Option<String> = None;
    let mut checksum: Option<String> = None;

    // handle named sample sets ...
    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) =
        tail_drain.peek()
    {
        // hard-coded tutorial sample set with checksum ...
        if s.as_str() == "tutorial" {
            url = Some("https://github.com/the-drunk-coder/megra-public-samples/archive/refs/heads/master.zip".to_string());
            checksum = Some(
                "7a339e8672511be64fa46961bbfdb3d6f797ebbd9572fc3adf551b737d3c4dcd".to_string(),
            );
        }
    } else {
        while let Some(c) = tail_drain.next() {
            if let EvaluatedExpr::Keyword(k) = c {
                if k.as_str() == "checksum" {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::String(s),
                    ))) = tail_drain.peek()
                    {
                        checksum = Some(s.to_string());
                    }
                }
                if k.as_str() == "url" {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::String(s),
                    ))) = tail_drain.peek()
                    {
                        url = Some(s.to_string());
                    }
                }
                if k.as_str() == "file" {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::String(s),
                    ))) = tail_drain.peek()
                    {
                        file = Some(s.to_string());
                    }
                }
            }
        }
    }

    // url has priority in case someone provided both ..
    if let Some(url_string) = url {
        Ok(EvaluatedExpr::Command(Command::ImportSampleSet(
            SampleResource::Url(url_string, checksum),
        )))
    } else {
        file.map(|path_string| {
            EvaluatedExpr::Command(Command::ImportSampleSet(SampleResource::File(
                path_string,
                checksum,
            )))
        })
        .ok_or(anyhow!("import-sample-set - missing or invalid file name"))
    }
}

#[allow(clippy::unnecessary_unwrap)]
pub fn load_sample_as_wavematrix(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut key: Option<String> = None;
    let mut path: Option<String> = None;
    let mut method: Option<String> = Some("zerocrossing_fixed_stretch_inverse".to_string());
    let mut matrix_size: Option<(usize, usize)> = None;
    let mut start: Option<f32> = Some(0.0);

    while let Some(c) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = c {
            if k.as_str() == "key" {
                // default is zero ...
                if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(n)))) =
                    tail_drain.next()
                {
                    key = Some(n);
                }
            }
            if k.as_str() == "start" {
                // default is zero ...
                if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) =
                    tail_drain.next()
                {
                    start = Some(f);
                }
            }
            if k.as_str() == "path" {
                // default is zero ...
                if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) =
                    tail_drain.next()
                {
                    path = Some(s);
                }
            }
            if k.as_str() == "method" {
                // default is zero ...
                if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) =
                    tail_drain.next()
                {
                    method = Some(s);
                }
            }
            if k.as_str() == "size" {
                // default is zero ...
                if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(x)))) =
                    tail_drain.next()
                {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        y,
                    )))) = tail_drain.next()
                    {
                        matrix_size = Some((x as usize, y as usize));
                    }
                }
            }
        }
    }
    if key.is_some() && path.is_some() && matrix_size.is_some() {
        Ok(EvaluatedExpr::Command(Command::LoadSampleAsWavematrix(
            key.unwrap(),
            path.unwrap(),
            method.unwrap(),
            matrix_size.unwrap(),
            start.unwrap(),
        )))
    } else {
        Err(anyhow!("can't load sample {key:?} {path:?} as wavematrix"))
    }
}

pub fn freeze_buffer(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    // on the user side,
    // both input- and freeze buffers are counted
    // starting at 1
    let mut inbuf: usize = 0;
    let mut freezbuf: usize = 0;

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Keyword(k) => {
                if k.as_str() == "in" {
                    // default is zero ...
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.next()
                    {
                        if n as usize > 0 {
                            inbuf = n as usize - 1;
                        }
                    }
                }
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                if f as usize > 0 {
                    freezbuf = f as usize - 1;
                }
            }
            _ => {}
        }
    }

    Ok(EvaluatedExpr::Command(Command::FreezeBuffer(
        freezbuf, inbuf,
    )))
}

pub fn freeze_add_buffer(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    // on the user side,
    // both input- and freeze buffers are counted
    // starting at 1
    let mut inbuf: usize = 0;
    let mut freezbuf: usize = 0;

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Keyword(k) => {
                if k.as_str() == "in" {
                    // default is zero ...
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        n,
                    )))) = tail_drain.next()
                    {
                        if n as usize > 0 {
                            inbuf = n as usize - 1;
                        }
                    }
                }
            }
            EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f))) => {
                if f as usize > 0 {
                    freezbuf = f as usize - 1;
                }
            }
            _ => {}
        }
    }

    Ok(EvaluatedExpr::Command(Command::FreezeAddBuffer(
        freezbuf, inbuf,
    )))
}

pub fn load_sample(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    let mut collect_keywords = false;

    let mut keywords: Vec<String> = Vec::new();
    let mut path: String = "".to_string();
    let mut set: String = "".to_string();
    let mut downmix_stereo = false;

    while let Some(c) = tail_drain.next() {
        if collect_keywords {
            if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(ref s))) = c {
                keywords.push(s.to_string());
                continue;
            } else {
                collect_keywords = false;
            }
        }

        if let EvaluatedExpr::Keyword(k) = c {
            match k.as_str() {
                "keywords" => {
                    collect_keywords = true;
                    continue;
                }
                "set" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Symbol(n),
                    ))) = tail_drain.next()
                    {
                        set = n.to_string();
                    }
                }
                "path" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::String(n),
                    ))) = tail_drain.next()
                    {
                        path = n.to_string();
                    }
                }
                "use-stereo" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Boolean(b),
                    ))) = tail_drain.next()
                    {
                        downmix_stereo = !b;
                    }
                }
                _ => println!("{k}"),
            }
        }
    }

    Ok(EvaluatedExpr::Command(Command::LoadSample(
        set,
        keywords,
        path,
        downmix_stereo,
    )))
}

pub fn load_sample_sets(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    let path = if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(n))) =
        tail_drain.next().unwrap()
    {
        n
    } else {
        bail!("load-sample-sets - path missing");
    };

    let mut downmix_stereo = false;
    if let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        if k == "use-stereo" {
            if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(b)))) =
                tail_drain.next()
            {
                downmix_stereo = !b
            }
        }
    }

    Ok(EvaluatedExpr::Command(Command::LoadSampleSets(
        path,
        downmix_stereo,
    )))
}

pub fn load_sample_set(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    let path = if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(n))) =
        tail_drain.next().unwrap()
    {
        n
    } else {
        bail!("load-sample-set - path missing");
    };

    let mut downmix_stereo = false;
    if let Some(EvaluatedExpr::Keyword(k)) = tail_drain.next() {
        if k == "use-stereo" {
            if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(b)))) =
                tail_drain.next()
            {
                downmix_stereo = !b
            }
        }
    }

    Ok(EvaluatedExpr::Command(Command::LoadSampleSet(
        path,
        downmix_stereo,
    )))
}

pub fn tmod(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    Ok(EvaluatedExpr::Command(Command::Tmod(
        match tail_drain.next() {
            Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => p,
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => {
                DynVal::with_value(f)
            }
            _ => DynVal::with_value(1.0),
        },
    )))
}

pub fn latency(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    Ok(EvaluatedExpr::Command(Command::Latency(
        match tail_drain.next() {
            Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => p,
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => {
                DynVal::with_value(f)
            }
            _ => DynVal::with_value(0.05),
        },
    )))
}

pub fn bpm(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    Ok(EvaluatedExpr::Command(Command::Bpm(
        match tail_drain.next() {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => {
                60000.0 / f
            }
            _ => 200.0,
        },
    )))
}

pub fn default_duration(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    Ok(EvaluatedExpr::Command(Command::DefaultDuration(
        match tail_drain.next() {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => f,
            _ => 200.0,
        },
    )))
}

pub fn globres(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    Ok(EvaluatedExpr::Command(Command::GlobRes(
        match tail_drain.next() {
            Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => f,
            _ => 400000.0,
        },
    )))
}

pub fn reverb(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    let mut param_map = HashMap::new();

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "damp" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        f,
                    )))) = tail_drain.next()
                    {
                        param_map.insert(
                            SynthParameterLabel::ReverbDampening,
                            ParameterValue::Scalar(DynVal::with_value(f)),
                        );
                    }
                }
                "mix" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        f,
                    )))) = tail_drain.next()
                    {
                        param_map.insert(
                            SynthParameterLabel::ReverbMix,
                            ParameterValue::Scalar(DynVal::with_value(f.clamp(0.01, 0.99))),
                        );
                    }
                }
                "roomsize" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        f,
                    )))) = tail_drain.next()
                    {
                        param_map.insert(
                            SynthParameterLabel::ReverbRoomsize,
                            ParameterValue::Scalar(DynVal::with_value(f.clamp(0.01, 0.99))),
                        );
                    }
                }

                _ => println!("{k}"),
            },
            _ => println! {"ignored"},
        }
    }

    Ok(EvaluatedExpr::Command(Command::GlobalRuffboxParams(
        param_map,
    )))
}

pub fn delay(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    let mut param_map = HashMap::new();

    while let Some(c) = tail_drain.next() {
        match c {
            EvaluatedExpr::Keyword(k) => match k.as_str() {
                "damp-freq" => match tail_drain.next() {
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => {
                        param_map.insert(
                            SynthParameterLabel::DelayDampeningFrequency,
                            ParameterValue::Scalar(DynVal::with_value(f.clamp(20.0, 18000.0))),
                        );
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => {
                        param_map.insert(
                            SynthParameterLabel::DelayDampeningFrequency,
                            ParameterValue::Scalar(p),
                        );
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(m))) => {
                        param_map.insert(SynthParameterLabel::DelayDampeningFrequency, m);
                    }
                    _ => {}
                },
                "feedback" | "fb" => match tail_drain.next() {
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => {
                        param_map.insert(
                            SynthParameterLabel::DelayFeedback,
                            ParameterValue::Scalar(DynVal::with_value(f.clamp(0.01, 0.99))),
                        );
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => {
                        param_map.insert(
                            SynthParameterLabel::DelayFeedback,
                            ParameterValue::Scalar(p),
                        );
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(m))) => {
                        param_map.insert(SynthParameterLabel::DelayFeedback, m);
                    }
                    _ => {}
                },
                "mix" => {
                    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(
                        f,
                    )))) = tail_drain.next()
                    {
                        param_map.insert(
                            SynthParameterLabel::DelayMix,
                            ParameterValue::Scalar(DynVal::with_value(f.clamp(0.01, 0.99))),
                        );
                    }
                }
                "time" | "t" => match tail_drain.next() {
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => {
                        param_map.insert(
                            SynthParameterLabel::DelayTime,
                            ParameterValue::Scalar(DynVal::with_value(
                                (f / 1000.0).clamp(0.01, 1.99),
                            )),
                        );
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => {
                        param_map.insert(SynthParameterLabel::DelayTime, ParameterValue::Scalar(p));
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(m))) => {
                        param_map.insert(SynthParameterLabel::DelayTime, m);
                    }
                    _ => {}
                },
                "rate" | "r" => match tail_drain.next() {
                    Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Float(f)))) => {
                        param_map.insert(
                            SynthParameterLabel::DelayRate,
                            ParameterValue::Scalar(DynVal::with_value(
                                (f / 1000.0).clamp(0.01, 1.99),
                            )),
                        );
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::Parameter(p))) => {
                        param_map.insert(SynthParameterLabel::DelayRate, ParameterValue::Scalar(p));
                    }
                    Some(EvaluatedExpr::Typed(TypedEntity::ParameterValue(m))) => {
                        param_map.insert(SynthParameterLabel::DelayRate, m);
                    }
                    _ => {}
                },
                _ => println!("{k}"),
            },
            _ => println! {"ignored"},
        }
    }

    Ok(EvaluatedExpr::Command(Command::GlobalRuffboxParams(
        param_map,
    )))
}

pub fn export_dot(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);

    // filename
    let filename =
        if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) =
            tail_drain.next()
        {
            s
        } else {
            bail!("export-dot - missing filename");
        };

    match tail_drain.next() {
        Some(EvaluatedExpr::Typed(TypedEntity::Generator(g))) => Ok(EvaluatedExpr::Command(
            Command::ExportDotStatic(filename, g),
        )),
        Some(EvaluatedExpr::Keyword(k)) => {
            match k.as_str() {
                "live" => {
                    let mut id_tags = BTreeSet::new();
                    // expect more tags
                    while let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(
                        Comparable::Symbol(si),
                    ))) = tail_drain.next()
                    {
                        id_tags.insert(si);
                    }
                    // collect next symbols
                    Ok(EvaluatedExpr::Command(Command::ExportDotRunning((
                        filename, id_tags,
                    ))))
                }
                _ => Err(anyhow!("export-dot - keyword arg {k} invalid")),
            }
        }
        _ => Err(anyhow!("export-dot - invalid argument")),
    }
}

pub fn once(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let tail_drain = tail.drain(..).skip(1);
    let mut sound_events = Vec::new();
    let mut control_events = Vec::new();

    for c in tail_drain {
        match c {
            EvaluatedExpr::Typed(TypedEntity::SoundEvent(mut e)) => {
                sound_events.push(e.get_static(globals));
            }
            EvaluatedExpr::Typed(TypedEntity::ControlEvent(c)) => control_events.push(c),
            _ => {}
        }
    }

    Ok(EvaluatedExpr::Command(Command::Once(
        sound_events,
        control_events,
    )))
}

pub fn step_part(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    match tail_drain.next() {
        Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s)))) => {
            Ok(EvaluatedExpr::Command(Command::StepPart(s)))
        }
        Some(EvaluatedExpr::Identifier(s)) => Ok(EvaluatedExpr::Command(Command::StepPart(s))),
        _ => Err(anyhow!("step-part - no part to step ...")),
    }
}

pub fn clear(
    _: &FunctionMap,
    _: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    Ok(EvaluatedExpr::Command(Command::Clear))
}

pub fn connect_visualizer(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let tail_drain = tail.drain(..).skip(1);

    let mut exclusion_list: BTreeSet<String> = BTreeSet::new();

    let mut collect_excludes = false;
    for c in tail_drain {
        if let EvaluatedExpr::Keyword(ref k) = c {
            if k.as_str() == "exclude" {
                collect_excludes = true;
                continue;
            }
        }
        if collect_excludes {
            if let EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Symbol(s))) = c {
                exclusion_list.insert(s);
            }
        }
    }

    Ok(EvaluatedExpr::Command(Command::ConnectVisualizer(
        exclusion_list,
    )))
}

pub fn start_recording(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    let prefix = if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) =
        tail_drain.next()
    {
        Some(s)
    } else {
        None
    };

    let mut rec_input = false;
    while let Some(c) = tail_drain.next() {
        if let EvaluatedExpr::Keyword(k) = c {
            if k.as_str() == "input" {
                // default is zero ...
                if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::Boolean(b)))) =
                    tail_drain.next()
                {
                    rec_input = b;
                }
            }
        }
    }

    Ok(EvaluatedExpr::Command(Command::StartRecording(
        prefix, rec_input,
    )))
}

pub fn stop_recording(
    _: &FunctionMap,
    _: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    Ok(EvaluatedExpr::Command(Command::StopRecording))
}

pub fn load_file(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    let mut tail_drain = tail.drain(..).skip(1);
    if let Some(EvaluatedExpr::Typed(TypedEntity::Comparable(Comparable::String(s)))) =
        tail_drain.next()
    {
        Ok(EvaluatedExpr::Command(Command::LoadFile(s)))
    } else {
        bail!("load-file - missing or invalid filename")
    }
}

pub fn print(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    _: OutputMode,
) -> Result<EvaluatedExpr> {
    // ignore function name
    resolve_globals(&mut tail[1..], globals);
    let mut tail_drain = tail.drain(1..);

    if let Some(EvaluatedExpr::Typed(t)) = tail_drain.next() {
        Ok(EvaluatedExpr::Command(Command::Print(t)))
    } else {
        bail!("print - missing or invalid entity")
    }
}
