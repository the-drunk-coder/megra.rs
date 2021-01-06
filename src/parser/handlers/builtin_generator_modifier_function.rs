use std::collections::HashMap;
use crate::builtin_types::*;
use crate::generator::*;

pub fn handle(gen_mod: &BuiltInGenModFun, tail: &mut Vec<Expr>, _parts_store: &PartsStore) -> Atom {

    let last = tail.pop();
    match last {
	Some(Expr::Constant(Atom::Generator(mut g))) => {
	    let mut tail_drain = tail.drain(..); 	    
	    let mut pos_args = Vec::new();
	    let mut named_args = HashMap::new();
	    
	    while let Some(Expr::Constant(c)) = tail_drain.next() {
		match c {
		    Atom::Float(f) => pos_args.push(ConfigParameter::Numeric(f)),
		    Atom::Keyword(k) => {
			named_args.insert(k, match tail_drain.next() {
			    Some(Expr::Constant(Atom::Float(f))) => ConfigParameter::Numeric(f),
			    Some(Expr::Constant(Atom::Symbol(s))) => ConfigParameter::Symbolic(s),
			    _ => ConfigParameter::Numeric(0.0) // dumb placeholder			    
			});
		    },
		    _ => {}
		} 
	    }

	    match gen_mod {
		BuiltInGenModFun::Haste => haste(&mut g.root_generator, &mut g.time_mods, &pos_args, &named_args),
		BuiltInGenModFun::Relax => relax(&mut g.root_generator, &mut g.time_mods, &pos_args, &named_args),
		BuiltInGenModFun::Grow => grow(&mut g.root_generator, &mut g.time_mods, &pos_args, &named_args),
		BuiltInGenModFun::Shrink => shrink(&mut g.root_generator, &mut g.time_mods, &pos_args, &named_args),
		BuiltInGenModFun::Blur => blur(&mut g.root_generator, &mut g.time_mods, &pos_args, &named_args),
		BuiltInGenModFun::Sharpen => sharpen(&mut g.root_generator, &mut g.time_mods, &pos_args, &named_args),
		BuiltInGenModFun::Shake => shake(&mut g.root_generator, &mut g.time_mods, &pos_args, &named_args),
		BuiltInGenModFun::Skip => skip(&mut g.root_generator, &mut g.time_mods, &pos_args, &named_args),
		BuiltInGenModFun::Rewind => rewind(&mut g.root_generator, &mut g.time_mods, &pos_args, &named_args),
	    }
	    Atom::Generator(g)
	},	
	
	Some(l) => {
	    tail.push(l);

	    let mut tail_drain = tail.drain(..); 	    
	    let mut pos_args = Vec::new();
	    let mut named_args = HashMap::new();
	    
	    while let Some(Expr::Constant(c)) = tail_drain.next() {
		match c {
		    Atom::Float(f) => pos_args.push(ConfigParameter::Numeric(f)),
		    Atom::Keyword(k) => {
			named_args.insert(k, match tail_drain.next() {
			    Some(Expr::Constant(Atom::Float(f))) => ConfigParameter::Numeric(f),
			    Some(Expr::Constant(Atom::Symbol(s))) => ConfigParameter::Symbolic(s),
			    _ => ConfigParameter::Numeric(0.0) // dumb placeholder			    
			});
		    },
		    _ => {}
		} 
	    }
    
	    Atom::GeneratorModifierFunction (match gen_mod {
		BuiltInGenModFun::Haste => (haste, pos_args, named_args),
		BuiltInGenModFun::Relax => (relax, pos_args, named_args),
		BuiltInGenModFun::Grow => (grow, pos_args, named_args),
		BuiltInGenModFun::Shrink => (shrink, pos_args, named_args),
		BuiltInGenModFun::Blur => (blur, pos_args, named_args),
		BuiltInGenModFun::Sharpen => (sharpen, pos_args, named_args),
		BuiltInGenModFun::Shake => (shake, pos_args, named_args),
		BuiltInGenModFun::Skip => (skip, pos_args, named_args),
		BuiltInGenModFun::Rewind => (rewind, pos_args, named_args),
	    })
	},
	None => {
	    Atom::Nothing
	}
    } 
}
