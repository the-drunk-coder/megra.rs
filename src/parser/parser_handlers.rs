use std::collections::HashSet;

use crate::builtin_types::*;
use crate::event::*;
use crate::session::SyncContext;
use crate::generator::Generator;

use crate::parser::parser_helpers::*;

pub fn handle_load_sample(tail: &mut Vec<Expr>) -> Atom {

    let mut tail_drain = tail.drain(..);
    
    let mut collect_keywords = false;
    
    let mut keywords:Vec<String> = Vec::new();
    let mut path:String = "".to_string();
    let mut set:String = "".to_string();
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {

	if collect_keywords {
	    if let Atom::Symbol(ref s) = c {
		keywords.push(s.to_string());
		continue;
	    } else {
		collect_keywords = false;
	    }				    
	}
	
	match c {
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "keywords" => {
			collect_keywords = true;
			continue;	
		    },
		    "set" => {
			if let Expr::Constant(Atom::Symbol(n)) = tail_drain.next().unwrap() {
			    set = n.to_string();
			}
		    },
		    "path" => {
			if let Expr::Constant(Atom::Description(n)) = tail_drain.next().unwrap() {
			    path = n.to_string();
			}
		    },
		    _ => println!("{}", k)
		}
	    }
	    _ => println!{"ignored"}
	}
    }
    
    Atom::Command(Command::LoadSample((set, keywords, path)))
}

pub fn handle_sync_context(tail: &mut Vec<Expr>, parts_store: &PartsStore) -> Atom {
    let mut tail_drain = tail.drain(..);
    
    // name is the first symbol
    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();
    let active = get_bool_from_expr(&tail_drain.next().unwrap()).unwrap();

    if !active {
	return Atom::SyncContext(SyncContext {
	    name: name,
	    generators: Vec::new(),	    
	    sync_to: None,
	    active: false,
	})
    }

    let mut gens: Vec<Generator> = Vec::new();
    let mut sync_to = None;
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {		
	match c {
	    Atom::Keyword(k) => {
		match k.as_str() {
		    "sync" => {
			if let Expr::Constant(Atom::Symbol(sync)) = tail_drain.next().unwrap() {
			    sync_to = Some(sync);
			}			
		    }
		    _ => {} // ignore
		}
	    },
	    Atom::Symbol(s) => {
		if let Some(kl) = parts_store.get(&s) {
		    let mut klc = kl.clone();
		    for k in klc.iter_mut() {
			k.id_tags.insert(name.clone());
		    }
		    gens.append(&mut klc);
		} else {
		    println!("warning: '{} not defined!", s);
		}
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
	sync_to: sync_to,
	active: true,
    })
}

pub fn handle_control_event(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let mut sync_contexts = Vec::new();

    while let Some(Expr::Constant(Atom::SyncContext(s))) = tail_drain.next() {
	sync_contexts.push(s);
    }

    Atom::ControlEvent(ControlEvent {
	tags: HashSet::new(),
	ctx: if sync_contexts.is_empty() { None } else { Some(sync_contexts) },
    })
}

pub fn handle_load_part(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);
    let mut gens = Vec::new();

    let name: String = get_string_from_expr(&tail_drain.next().unwrap()).unwrap();
    
    while let Some(Expr::Constant(c)) = tail_drain.next() {
	match c {
	    Atom::Generator(g) => gens.push(g),
	    Atom::GeneratorList(mut gl) => gens.append(&mut gl),
	    _ => {}
	}
    }
    
    Atom::Command(Command::LoadPart((name, gens)))
}
