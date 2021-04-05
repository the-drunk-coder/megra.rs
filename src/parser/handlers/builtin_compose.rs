use crate::builtin_types::*;
use crate::generator_processor::*;

fn collect_compose(tail: &mut Vec<Expr>) -> Vec<Box<dyn GeneratorProcessor + Send>> {
    let mut gen_procs = Vec::new();
    let mut tail_drain = tail.drain(..);
    while let Some(Expr::Constant(c)) = tail_drain.next() {
	match c {
	    Atom::GeneratorProcessor(gp) => {
		gen_procs.push(gp);
	    }
	    _ => {}
	}
    }
    gen_procs
}

pub fn handle(tail: &mut Vec<Expr>) -> Atom {
    let last = tail.pop();
    match last {
	Some(Expr::Constant(Atom::Symbol(s))) => Atom::PartProxy(PartProxy::Proxy(
            s,
            collect_compose(tail),
        )),
        Some(Expr::Constant(Atom::PartProxy(PartProxy::Proxy(s, mut proxy_mods)))) => {
            proxy_mods.append(&mut collect_compose(tail));
            Atom::PartProxy(PartProxy::Proxy(s, proxy_mods))
        }
	Some(Expr::Constant(Atom::ProxyList(mut l))) => {
            let gp = collect_compose(tail);
            let mut pdrain = l.drain(..);
            let mut new_list = Vec::new();
            while let Some(PartProxy::Proxy(s, mut proxy_mods)) = pdrain.next() {
                proxy_mods.append(&mut gp.clone());
                new_list.push(PartProxy::Proxy(s, proxy_mods));
            }
            Atom::ProxyList(new_list)
        }
	Some(Expr::Constant(Atom::Generator(mut g))) => {
            g.processors.append(&mut collect_compose(tail));
            Atom::Generator(g)
        }
	Some(Expr::Constant(Atom::GeneratorList(mut gl))) => {
            let gp = collect_compose(tail);
            for gen in gl.iter_mut() {
                gen.processors.append(&mut gp.clone());
            }
            Atom::GeneratorList(gl)
        }        
	Some(l) => {
            tail.push(l);
            Atom::GeneratorProcessorList(collect_compose(tail))
        }
        _ => Atom::Nothing,
    }
}


