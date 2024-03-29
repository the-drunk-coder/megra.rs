use ruffbox_synth::building_blocks::SynthParameterLabel;
use std::collections::{BTreeSet, HashMap};
use std::sync;

use crate::builtin_types::*;
use crate::event::{Event, EventOperation};
use crate::generator::Generator;
use crate::generator_processor::{GeneratorWrapperProcessor, PearProcessor};
use crate::parameter::{DynVal, ParameterValue};

use crate::parser::{EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleAndWavematrixSet};

use super::resolver::resolve_globals;

pub type GenSpreader = fn(&mut [Generator], &OutputMode);

pub fn spread_gens(gens: &mut [Generator], out_mode: &OutputMode) {
    let positions = match out_mode {
        OutputMode::Stereo => {
            if gens.len() == 1 {
                vec![0.0]
            } else {
                let mut p = Vec::new();
                for i in 0..gens.len() {
                    let val = (i as f32 * (2.0 / (gens.len() as f32 - 1.0))) - 1.0;
                    p.push(val);
                }
                p
            }
        }
        OutputMode::FourChannel => {
            if gens.len() == 1 {
                vec![0.0]
            } else {
                let mut p = Vec::new();
                for i in 0..gens.len() {
                    let val = 1.0 + (i as f32 * (3.0 / (gens.len() as f32 - 1.0)));
                    p.push(val);
                }
                p
            }
        }
        OutputMode::EightChannel => {
            if gens.len() == 1 {
                vec![0.0]
            } else {
                let mut p = Vec::new();
                for i in 0..gens.len() {
                    let val = 1.0 + (i as f32 * (7.0 / (gens.len() as f32 - 1.0)));
                    p.push(val);
                }
                p
            }
        }
        OutputMode::SixteenChannel => {
            if gens.len() == 1 {
                vec![0.0]
            } else {
                let mut p = Vec::new();
                for i in 0..gens.len() {
                    let val = 1.0 + (i as f32 * (15.0 / (gens.len() as f32 - 1.0)));
                    p.push(val);
                }
                p
            }
        }
    };

    for i in 0..gens.len() {
        let mut p = PearProcessor::new();
        let mut ev = Event::with_name_and_operation("pos".to_string(), EventOperation::Replace);
        ev.params.insert(
            SynthParameterLabel::ChannelPosition.into(),
            ParameterValue::Scalar(DynVal::with_value(positions[i])),
        );
        let mut filtered_events = HashMap::new();
        filtered_events.insert(vec!["".to_string()], (true, vec![ev]));
        p.events_to_be_applied
            .push((DynVal::with_value(100.0), filtered_events));
        gens[i].processors.push(Box::new(p));
    }
}

pub fn eval_multiplyer(
    gen_spread: GenSpreader,
    tail: &mut Vec<EvaluatedExpr>,
    out_mode: OutputMode,
    globals: &std::sync::Arc<GlobalVariables>,
) -> Option<EvaluatedExpr> {
    let last = tail.pop(); // generator or generator list ...

    let mut gen_proc_list_list = Vec::new();

    for c in tail.drain(..).skip(1) {
        // ignore function name
        match c {
            EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifierList(gpl)) => {
                gen_proc_list_list.push(gpl);
            }
            EvaluatedExpr::Typed(TypedEntity::GeneratorModifierList(gml)) => {
                gen_proc_list_list.push(gml);
            }
            EvaluatedExpr::Typed(TypedEntity::GeneratorProcessorOrModifier(gp)) => {
                let gpl = vec![gp];
                gen_proc_list_list.push(gpl);
            }
            EvaluatedExpr::Typed(TypedEntity::Generator(g)) => {
                gen_proc_list_list.push(vec![GeneratorProcessorOrModifier::GeneratorProcessor(
                    Box::new(GeneratorWrapperProcessor::with_generator(g)),
                )]);
            }
            _ => {
                println!("can't multiply {c:?} {last:?}");
            }
        }
    }

    Some(match last {
        Some(EvaluatedExpr::Typed(TypedEntity::Generator(g))) => {
            let mut gens = Vec::new();
            let mut idx: usize = 0;

            // multiply into duplicates by cloning ...
            for mut gpl in gen_proc_list_list.drain(..) {
                let mut pclone = g.clone();

                // this isn't super elegant but hey ...
                for i in idx..100 {
                    let tag = format!("mpx-{i}");
                    if !pclone.id_tags.contains(&tag) {
                        pclone.id_tags.insert(tag);
                        idx = i + 1;
                        break;
                    }
                }

                for gpom in gpl.drain(..) {
                    match gpom {
                        GeneratorProcessorOrModifier::GeneratorProcessor(gp) => {
                            pclone.processors.push(gp)
                        }
                        GeneratorProcessorOrModifier::GeneratorModifierFunction((
                            fun,
                            pos,
                            named,
                        )) => fun(&mut pclone, &pos, &named, globals),
                    }
                }

                gens.push(pclone);
            }
            gens.push(g);
            gen_spread(&mut gens, &out_mode);
            EvaluatedExpr::Typed(TypedEntity::GeneratorList(gens))
        }
        Some(EvaluatedExpr::Typed(TypedEntity::GeneratorList(mut gl))) => {
            let mut gens = Vec::new();

            // collect tags ... make sure the multiplying process leaves
            // each generator individually, but deterministically tagged ...
            let mut all_tags: BTreeSet<String> = BTreeSet::new();

            for gen in gl.iter() {
                all_tags.append(&mut gen.id_tags.clone());
            }

            let mut idx: usize = 0;
            for gen in gl.drain(..) {
                // multiply into duplicates by cloning ...
                for gpl in gen_proc_list_list.iter() {
                    let mut pclone = gen.clone();

                    // this isn't super elegant but hey ...
                    for i in idx..100 {
                        let tag = format!("mpx-{i}");
                        if !all_tags.contains(&tag) {
                            pclone.id_tags.insert(tag);
                            idx = i + 1;
                            break;
                        }
                    }

                    let mut gpl_clone = gpl.clone();
                    for gpom in gpl_clone.drain(..) {
                        match gpom {
                            GeneratorProcessorOrModifier::GeneratorProcessor(gp) => {
                                pclone.processors.push(gp)
                            }
                            GeneratorProcessorOrModifier::GeneratorModifierFunction((
                                fun,
                                pos,
                                named,
                            )) => fun(&mut pclone, &pos, &named, globals),
                        }
                    }

                    gens.push(pclone);
                }
                gens.push(gen);
                gen_spread(&mut gens, &out_mode);
            }
            EvaluatedExpr::Typed(TypedEntity::GeneratorList(gens))
        }
        _ => return None,
    })
}

pub fn eval_xspread(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    out_mode: OutputMode,
) -> Option<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    eval_multiplyer(spread_gens, tail, out_mode, globals)
}

pub fn eval_xdup(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    globals: &sync::Arc<GlobalVariables>,
    _: SampleAndWavematrixSet,
    out_mode: OutputMode,
) -> Option<EvaluatedExpr> {
    resolve_globals(&mut tail[1..], globals);
    eval_multiplyer(|_, _| {}, tail, out_mode, globals)
}
