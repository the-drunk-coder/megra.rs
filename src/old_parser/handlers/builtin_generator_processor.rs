pub fn collect_generator_processor(
    proc_type: &BuiltInGenProc,
    tail: &mut Vec<Expr>,
) -> GeneratorProcessorOrModifier {
    GeneratorProcessorOrModifier::GeneratorProcessor(match proc_type {
        BuiltInGenProc::Pear => collect_pear(tail),
        BuiltInGenProc::Inhibit => collect_inhibit_exhibit(tail, true, false),
        BuiltInGenProc::Exhibit => collect_inhibit_exhibit(tail, false, true),
        BuiltInGenProc::InExhibit => collect_inhibit_exhibit(tail, true, true),
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
        Some(Expr::Constant(Atom::Symbol(s))) => {
            // check if previous is a keyword ...
            // if not, assume it's a part proxy
            let prev = tail.pop();
            match prev {
                Some(Expr::Constant(Atom::Keyword(_))) => {
                    tail.push(prev.unwrap()); // push back for further processing
                    tail.push(Expr::Constant(Atom::Symbol(s)));
                    Atom::GeneratorProcessorOrModifier(collect_generator_processor(proc_type, tail))
                }
                _ => {
                    tail.push(prev.unwrap()); // push back for further processing
                    Atom::PartProxy(PartProxy::Proxy(
                        s,
                        vec![collect_generator_processor(proc_type, tail)],
                    ))
                }
            }
        }
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
            match gp {
                GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf) => {
                    // if it's a generator modifier function, such as shrink or skip,
                    // push it back as it belongs to the overarching processor
                    tail.push(Expr::Constant(Atom::GeneratorProcessorOrModifier(
                        GeneratorProcessorOrModifier::GeneratorModifierFunction(gmf),
                    )));
                    Atom::GeneratorProcessorOrModifier(collect_generator_processor(proc_type, tail))
                }
                _ => Atom::GeneratorProcessorOrModifierList(vec![
                    gp,
                    collect_generator_processor(proc_type, tail),
                ]),
            }
        }
        Some(Expr::Constant(Atom::GeneratorProcessorOrModifierList(mut l))) => {
            l.push(collect_generator_processor(proc_type, tail));
            Atom::GeneratorProcessorOrModifierList(l)
        }
        // pure modifier lists are handled differently
        Some(Expr::Constant(Atom::GeneratorModifierList(ml))) => {
            tail.push(Expr::Constant(Atom::GeneratorModifierList(ml)));
            Atom::GeneratorProcessorOrModifier(collect_generator_processor(proc_type, tail))
        }
        Some(l) => {
            tail.push(l);
            Atom::GeneratorProcessorOrModifier(collect_generator_processor(proc_type, tail))
        }
        None => Atom::Nothing,
    }
}
