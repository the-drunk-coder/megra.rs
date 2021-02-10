use crate::builtin_types::*;
use crate::session::SyncContext;
use crate::generator::Generator;
use crate::parser::parser_helpers::*;

pub fn handle(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    // name is the first symbol
    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();
    let active = get_bool_from_expr(&tail_drain.next().unwrap()).unwrap();

    if !active {
	return Atom::SyncContext(SyncContext {
	    name: name,
	    generators: Vec::new(),
	    part_proxies: Vec::new(),	    
	    sync_to: None,
	    active: false,
	    shift: 0
	})
    }

    let mut gens: Vec<Generator> = Vec::new();
    let mut proxies: Vec<PartProxy> = Vec::new();
    let mut sync_to = None;
    let mut shift:i32 = 0;
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {		
	match c {
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "sync" => {
			if let Expr::Constant(Atom::Symbol(sync)) = tail_drain.next().unwrap() {
			    sync_to = Some(sync);
			}			
		    },
		    "shift" => {
			if let Expr::Constant(Atom::Float(f)) = tail_drain.next().unwrap() {
			    shift = f as i32;
			}			
		    }
		    _ => {} // ignore
		}
	    },
	    Atom::Symbol(s) => {
		// part proxy without additional modifiers		
		proxies.push(PartProxy::Proxy(s, Vec::new()));
	    },
	    Atom::PartProxy(p) => {
		// part proxy without additional modifiers 
		proxies.push(p);
	    },
	    Atom::ProxyList(mut l) => {		
		// part proxy without additional modifiers 
		proxies.append(&mut l);
	    },
	    Atom::Generator(mut k) => {
		k.id_tags.insert(name.clone());
		gens.push(k);
	    },
	    Atom::GeneratorList(mut kl) => {
		for k in kl.iter_mut() {
		    k.id_tags.insert(name.clone());
		}
		gens.append(&mut kl);
	    }
	    _ => println!{"ignored"}
	}
    }
    
    Atom::SyncContext(SyncContext {
	name: name,
	generators: gens,
	part_proxies: proxies,
	sync_to: sync_to,
	active: true,
	shift: shift
    })
}
