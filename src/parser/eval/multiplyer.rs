use ruffbox_synth::ruffbox::synth::SynthParameterLabel;
use std::collections::{BTreeSet, HashMap};
use std::sync;

use crate::builtin_types::*;
use crate::event::{Event, EventOperation};
use crate::generator::Generator;
use crate::generator_processor::PearProcessor;
use crate::parameter::{Parameter, ParameterValue};

use crate::parser::{BuiltIn, EvaluatedExpr, FunctionMap};
use crate::{OutputMode, SampleSet};
use parking_lot::Mutex;

pub type GenSpreader = fn(&mut [Generator], &OutputMode);
pub type ProxySpreader = fn(&mut [PartProxy], &OutputMode);

fn spread_gens(gens: &mut [Generator], out_mode: &OutputMode) {
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
    };

    for i in 0..gens.len() {
        let mut p = PearProcessor::new();
        let mut ev = Event::with_name_and_operation("pos".to_string(), EventOperation::Replace);
        ev.params.insert(
            SynthParameterLabel::ChannelPosition,
            ParameterValue::Scalar(Parameter::with_value(positions[i])),
        );
        let mut filtered_events = HashMap::new();
        filtered_events.insert(vec!["".to_string()], (true, vec![ev]));
        p.events_to_be_applied
            .push((Parameter::with_value(100.0), filtered_events));
        gens[i].processors.push(Box::new(p));
    }
}

fn spread_proxies(proxies: &mut [PartProxy], out_mode: &OutputMode) {
    let positions = match out_mode {
        OutputMode::Stereo => {
            if proxies.len() == 1 {
                vec![0.0]
            } else {
                let mut p = Vec::new();
                for i in 0..proxies.len() {
                    let val = (i as f32 * (2.0 / (proxies.len() as f32 - 1.0))) - 1.0;
                    p.push(val);
                }
                p
            }
        }
        OutputMode::FourChannel => {
            if proxies.len() == 1 {
                vec![0.0]
            } else {
                let mut p = Vec::new();
                for i in 0..proxies.len() {
                    let val = 1.0 + (i as f32 * (3.0 / (proxies.len() as f32 - 1.0)));
                    p.push(val);
                }
                p
            }
        }
        OutputMode::EightChannel => {
            if proxies.len() == 1 {
                vec![0.0]
            } else {
                let mut p = Vec::new();
                for i in 0..proxies.len() {
                    let val = 1.0 + (i as f32 * (7.0 / (proxies.len() as f32 - 1.0)));
                    p.push(val);
                }
                p
            }
        }
    };

    for (i, prox) in proxies.iter_mut().enumerate() {
        let mut p = PearProcessor::new();
        let mut ev = Event::with_name_and_operation("pos".to_string(), EventOperation::Replace);
        ev.params.insert(
            SynthParameterLabel::ChannelPosition,
            ParameterValue::Scalar(Parameter::with_value(positions[i])),
        );
        let mut filtered_events = HashMap::new();
        filtered_events.insert(vec!["".to_string()], (true, vec![ev]));
        p.events_to_be_applied
            .push((Parameter::with_value(100.0), filtered_events));
        match prox {
            PartProxy::Proxy(_, ref mut procs) => procs.push(
                GeneratorProcessorOrModifier::GeneratorProcessor(Box::new(p)),
            ),
        }
    }
}

pub fn eval_multiplyer(
    gen_spread: GenSpreader,
    proxy_spread: ProxySpreader,
    tail: &mut Vec<EvaluatedExpr>,
    out_mode: OutputMode,
) -> Option<EvaluatedExpr> {
    let last = tail.pop(); // generator or generator list ...

    let mut gen_proc_list_list = Vec::new();

    for c in tail.drain(..).skip(1) {
        // ignore function name
        match c {
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifierList(gpl)) => {
                gen_proc_list_list.push(gpl);
            }
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorModifierList(gml)) => {
                gen_proc_list_list.push(gml);
            }
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorProcessorOrModifier(gp)) => {
                let gpl = vec![gp];
                gen_proc_list_list.push(gpl);
            }
            _ => {
                println!("can't multiply {:?} {:?}", c, last);
            }
        }
    }

    Some(match last {
        // create a proxy ...
        Some(EvaluatedExpr::Symbol(s)) => {
            println!("create proxy {}", s);
            let mut proxies = Vec::new();
            for gpl in gen_proc_list_list.drain(..) {
                proxies.push(PartProxy::Proxy(s.clone(), gpl));
            }
            proxies.push(PartProxy::Proxy(s, Vec::new()));
            // return early, this will be resolved in session handling !
            proxy_spread(&mut proxies, &out_mode);
            EvaluatedExpr::BuiltIn(BuiltIn::ProxyList(proxies))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::PartProxy(PartProxy::Proxy(s, procs)))) => {
            println!("create proxy list from proxy {}", s);
            let mut proxies = Vec::new();
            for mut gpl in gen_proc_list_list.drain(..) {
                let mut ngpl = procs.clone();
                ngpl.append(&mut gpl);
                proxies.push(PartProxy::Proxy(s.clone(), ngpl));
            }
            proxies.push(PartProxy::Proxy(s, procs));
            proxy_spread(&mut proxies, &out_mode);
            EvaluatedExpr::BuiltIn(BuiltIn::ProxyList(proxies))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::ProxyList(mut l))) => {
            println!("propagate proxy list");
            let mut proxies = Vec::new();
            for prox in l.iter() {
                match prox {
                    PartProxy::Proxy(s, procs) => {
                        for gpl in gen_proc_list_list.iter() {
                            let mut ngpl = procs.clone();
                            ngpl.append(&mut gpl.clone());
                            proxies.push(PartProxy::Proxy(s.clone(), ngpl));
                        }
                    }
                }
            }
            proxies.append(&mut l);
            proxy_spread(&mut proxies, &out_mode);
            EvaluatedExpr::BuiltIn(BuiltIn::ProxyList(proxies))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::Generator(g))) => {
            let mut gens = Vec::new();
            let mut idx: usize = 0;

            // multiply into duplicates by cloning ...
            for mut gpl in gen_proc_list_list.drain(..) {
                let mut pclone = g.clone();

                // this isn't super elegant but hey ...
                for i in idx..100 {
                    let tag = format!("mpx-{}", i);
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
                        )) => fun(&mut pclone, &pos, &named),
                    }
                }

                gens.push(pclone);
            }
            gens.push(g);
            gen_spread(&mut gens, &out_mode);
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorList(gens))
        }
        Some(EvaluatedExpr::BuiltIn(BuiltIn::GeneratorList(mut gl))) => {
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
                        let tag = format!("mpx-{}", i);
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
                            )) => fun(&mut pclone, &pos, &named),
                        }
                    }

                    gens.push(pclone);
                }
                gens.push(gen);
                gen_spread(&mut gens, &out_mode);
            }
            EvaluatedExpr::BuiltIn(BuiltIn::GeneratorList(gens))
        }
        _ => return None,
    })
}

pub fn eval_xspread(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    out_mode: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_multiplyer(spread_gens, spread_proxies, tail, out_mode)
}

pub fn eval_xdup(
    _: &FunctionMap,
    tail: &mut Vec<EvaluatedExpr>,
    _: &sync::Arc<GlobalParameters>,
    _: &sync::Arc<Mutex<SampleSet>>,
    out_mode: OutputMode,
) -> Option<EvaluatedExpr> {
    eval_multiplyer(|_, _| {}, |_, _| {}, tail, out_mode)
}
