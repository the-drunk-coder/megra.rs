use std::collections::HashMap;
//use ruffbox_synth::ruffbox::synth::SynthParameter;

use crate::builtin_types::*;
use crate::event_helpers::*;

use crate::generator_processor::*;
use crate::parameter::Parameter;
use crate::parser::parser_helpers::*;

fn collect_every(tail: &mut Vec<Expr>) -> Box<EveryProcessor> {
    let mut tail_drain = tail.drain(..);
    let mut proc = EveryProcessor::new();

    let mut last_filters = vec!["".to_string()];

    let mut cur_step = Parameter::with_value(1.0); // if nothing is specified, it's always applied
    let mut gen_mod_funs = Vec::new();
    let mut events = Vec::new();
    let mut collect_filters = false;

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf),
            ) => {
                gen_mod_funs.push(gmf);
                collect_filters = false;
            }
            Atom::SoundEvent(e) => {
                events.push(e);
                collect_filters = false;
            }
            Atom::Symbol(s) => {
                if collect_filters {
                    last_filters.push(s)
                }
            }
            Atom::Keyword(k) => {
                match k.as_str() {
                    "for" => {
                        if !events.is_empty() || !gen_mod_funs.is_empty() {
                            let mut n_mods = Vec::new();
                            n_mods.append(&mut gen_mod_funs);

                            let mut filtered_events = HashMap::new();
                            let mut n_evs = Vec::new();
                            let mut n_filters = Vec::new();
                            n_evs.append(&mut events);
                            n_filters.append(&mut last_filters);
                            filtered_events.insert(n_filters, n_evs);

                            proc.things_to_be_applied.push((
                                cur_step.clone(),
                                filtered_events,
                                n_mods,
                            ));
                        }
                        // collect new filters
                        collect_filters = true;
                    }
                    "n" => {
                        if !events.is_empty() || !gen_mod_funs.is_empty() {
                            let mut n_mods = Vec::new();
                            n_mods.append(&mut gen_mod_funs);

                            let mut filtered_events = HashMap::new();
                            let mut n_evs = Vec::new();
                            let mut n_filters = Vec::new();
                            n_evs.append(&mut events);
                            n_filters.append(&mut last_filters);
                            filtered_events.insert(n_filters, n_evs);

                            proc.things_to_be_applied.push((
                                cur_step.clone(),
                                filtered_events,
                                n_mods,
                            ));
                        }
                        // grab new probability
                        cur_step = get_next_param(&mut tail_drain, 1.0);
                        collect_filters = false;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // save last context
    if !events.is_empty() || !gen_mod_funs.is_empty() {
        let mut filtered_events = HashMap::new();
        filtered_events.insert(last_filters, events);
        proc.things_to_be_applied
            .push((cur_step, filtered_events, gen_mod_funs));
    }

    Box::new(proc)
}

fn collect_pear(tail: &mut Vec<Expr>) -> Box<PearProcessor> {
    let mut tail_drain = tail.drain(..);
    let mut proc = PearProcessor::new();

    let mut last_filters = vec!["".to_string()];

    let mut evs = Vec::new();
    let mut collect_filters = false;
    let mut cur_prob = Parameter::with_value(100.0); // if nothing is specified, it's always or prob 100

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::SoundEvent(e) => {
                evs.push(e);
                if collect_filters {
                    collect_filters = false;
                }
            }
            Atom::Keyword(k) => {
                match k.as_str() {
                    "p" => {
                        // save current context, if something has been found
                        if !evs.is_empty() {
                            let mut filtered_events = HashMap::new();
                            let mut n_evs = Vec::new();
                            let mut n_filters = Vec::new();
                            n_evs.append(&mut evs);
                            n_filters.extend_from_slice(&last_filters);
                            filtered_events.insert(n_filters, n_evs);
                            proc.events_to_be_applied
                                .push((cur_prob.clone(), filtered_events));
                        }
                        // grab new probability
                        cur_prob = get_next_param(&mut tail_drain, 100.0);
                        collect_filters = false;
                    }
                    "for" => {
                        if !evs.is_empty() {
                            let mut filtered_events = HashMap::new();
                            let mut n_evs = Vec::new();
                            let mut n_filters = Vec::new();
                            n_evs.append(&mut evs);
                            n_filters.append(&mut last_filters);
                            filtered_events.insert(n_filters, n_evs);
                            proc.events_to_be_applied
                                .push((cur_prob.clone(), filtered_events));
                        }
                        // collect new filters
                        collect_filters = true;
                    }
                    _ => {}
                }
            }
            Atom::Symbol(s) => {
                if collect_filters {
                    last_filters.push(s)
                }
            }
            _ => {}
        }
    }

    // save last context
    if !evs.is_empty() {
        let mut filtered_events = HashMap::new();
        filtered_events.insert(last_filters, evs);
        proc.events_to_be_applied.push((cur_prob, filtered_events));
    }
    Box::new(proc)
}

fn collect_apple(tail: &mut Vec<Expr>) -> Box<AppleProcessor> {
    let mut tail_drain = tail.drain(..);
    let mut proc = AppleProcessor::new();

    let mut cur_prob = Parameter::with_value(100.0); // if nothing is specified, it's always or prob 100
    let mut gen_mod_funs = Vec::new();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf),
            ) => {
                gen_mod_funs.push(gmf);
            }
            Atom::Keyword(k) => {
                if k == "p" {
                    if !gen_mod_funs.is_empty() {
                        let mut new_mods = Vec::new();
                        new_mods.append(&mut gen_mod_funs);
                        proc.modifiers_to_be_applied
                            .push((cur_prob.clone(), new_mods));
                    }
                    // grab new probability
                    cur_prob = get_next_param(&mut tail_drain, 100.0);
                }
            }
            _ => {}
        }
    }

    // save last context
    if !gen_mod_funs.is_empty() {
        proc.modifiers_to_be_applied.push((cur_prob, gen_mod_funs));
    }

    Box::new(proc)
}

fn collect_lifemodel(tail: &mut Vec<Expr>) -> Box<LifemodelProcessor> {
    let mut tail_drain = tail.drain(..);
    let mut proc = LifemodelProcessor::new();

    // positional args: growth cycle, lifespan, variance
    if let Some(growth_cycle) = get_float_from_expr_opt(&tail_drain.next()) {
        proc.growth_cycle = growth_cycle as usize;
    }

    if let Some(lifespan) = get_float_from_expr_opt(&tail_drain.next()) {
        proc.node_lifespan = lifespan as usize;
    }

    if let Some(variance) = get_float_from_expr_opt(&tail_drain.next()) {
        proc.variance = variance;
    }

    let mut collect_durations = false;
    let mut collect_keeps = false;

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        if collect_durations {
            match c {
                Atom::Float(f) => proc.durations.push(Parameter::with_value(f)),
                Atom::Parameter(ref p) => proc.durations.push(p.clone()),
                _ => {
                    collect_durations = false;
                }
            }
        }

        if collect_keeps {
            match c {
                Atom::Symbol(ref s) => {
                    proc.keep_param.insert(map_parameter(&s));
                }
                _ => {
                    collect_keeps = false;
                }
            }
        }

        if let Atom::Keyword(k) = c {
            match k.as_str() {
                "durs" => {
                    collect_durations = true;
                }
                "keep" => {
                    collect_keeps = true;
                }
                "apoptosis" => {
                    if let Expr::Constant(Atom::Boolean(b)) = tail_drain.next().unwrap() {
                        proc.apoptosis = b;
                    }
                }
                "method" => {
                    if let Expr::Constant(Atom::Keyword(k)) = tail_drain.next().unwrap() {
                        proc.growth_method = k;
                    }
                }
                "autophagia" => {
                    if let Expr::Constant(Atom::Boolean(b)) = tail_drain.next().unwrap() {
                        proc.autophagia = b;
                    }
                }
                "lifespan-variance" => {
                    if let Expr::Constant(Atom::Float(f)) = tail_drain.next().unwrap() {
                        proc.node_lifespan_variance = f;
                    }
                }
                "apoptosis-regain" => {
                    if let Expr::Constant(Atom::Float(f)) = tail_drain.next().unwrap() {
                        proc.apoptosis_regain = f;
                    }
                }
                "autophagia-regain" => {
                    if let Expr::Constant(Atom::Float(f)) = tail_drain.next().unwrap() {
                        proc.autophagia_regain = f;
                    }
                }
                "local-resources" => {
                    if let Expr::Constant(Atom::Float(f)) = tail_drain.next().unwrap() {
                        proc.local_resources = f;
                    }
                }
                "cost" => {
                    if let Expr::Constant(Atom::Float(f)) = tail_drain.next().unwrap() {
                        proc.growth_cost = f;
                    }
                }
                "global-contrib" => {
                    if let Expr::Constant(Atom::Boolean(b)) = tail_drain.next().unwrap() {
                        proc.global_contrib = b;
                    }
                }
                _ => {}
            }
        }
    }

    Box::new(proc)
}

pub fn collect_generator_processor(
    proc_type: &BuiltInGenProc,
    tail: &mut Vec<Expr>,
) -> GeneratorProcessorOrModifier {
    GeneratorProcessorOrModifier::GeneratorProcessor(match proc_type {
        BuiltInGenProc::Pear => collect_pear(tail),
        BuiltInGenProc::Apple => collect_apple(tail),
        BuiltInGenProc::Every => collect_every(tail),
        BuiltInGenProc::Lifemodel => collect_lifemodel(tail),
    })
}

// store list of genProcs in a vec if there's no root gen ???
pub fn handle(proc_type: &BuiltInGenProc, tail: &mut Vec<Expr>) -> Atom {
    let last = tail.pop();
    match last {
        Some(Expr::Constant(Atom::Generator(mut g))) => {
            if let GeneratorProcessorOrModifier::GeneratorProcessor(gp) =
                collect_generator_processor(proc_type, tail)
            {
                g.processors.push(gp);
            }
            Atom::Generator(g)
        }
        Some(Expr::Constant(Atom::Symbol(s))) => Atom::PartProxy(PartProxy::Proxy(
            s,
            vec![collect_generator_processor(proc_type, tail)],
        )),
        Some(Expr::Constant(Atom::PartProxy(PartProxy::Proxy(s, mut proxy_mods)))) => {
            proxy_mods.push(collect_generator_processor(proc_type, tail));
            Atom::PartProxy(PartProxy::Proxy(s, proxy_mods))
        }
        Some(Expr::Constant(Atom::ProxyList(mut l))) => {
            let gp = collect_generator_processor(proc_type, tail);
            let mut pdrain = l.drain(..);
            let mut new_list = Vec::new();
            while let Some(PartProxy::Proxy(s, mut proxy_mods)) = pdrain.next() {
                proxy_mods.push(gp.clone());
                new_list.push(PartProxy::Proxy(s, proxy_mods));
            }
            Atom::ProxyList(new_list)
        }
        Some(Expr::Constant(Atom::GeneratorList(mut gl))) => {
            if let GeneratorProcessorOrModifier::GeneratorProcessor(gp) =
                collect_generator_processor(proc_type, tail)
            {
                for gen in gl.iter_mut() {
                    gen.processors.push(gp.clone());
                }
            }
            Atom::GeneratorList(gl)
        }
        Some(Expr::Constant(Atom::GeneratorProcessorOrModifier(gp))) => {
            Atom::GeneratorProcessorOrModifierList(vec![
                gp,
                collect_generator_processor(proc_type, tail),
            ])
        }
        Some(Expr::Constant(Atom::GeneratorProcessorOrModifierList(mut l))) => {
            l.push(collect_generator_processor(proc_type, tail));
            Atom::GeneratorProcessorOrModifierList(l)
        }
        Some(l) => {
            tail.push(l);
            Atom::GeneratorProcessorOrModifier(collect_generator_processor(proc_type, tail))
        }
        None => Atom::Nothing,
    }
}
