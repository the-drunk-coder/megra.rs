use crate::builtin_types::*;
use crate::generator::*;

pub fn handle(gen_mod: &BuiltInGenModFun, tail: &mut Vec<Expr>, _parts_store: &PartsStore) -> Atom {

    let last = tail.pop();
    match last {
	Some(Expr::Constant(Atom::Generator(mut g))) => {
	    let mut tail_drain = tail.drain(..); 	    
	    let mut args = Vec::new();

	    while let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
		args.push(f);
	    }

	    match gen_mod {
		BuiltInGenModFun::Haste => haste(&mut g.root_generator, &mut g.time_mods, &args),
		BuiltInGenModFun::Relax => relax(&mut g.root_generator, &mut g.time_mods, &args),
		BuiltInGenModFun::Grow => grow(&mut g.root_generator, &mut g.time_mods, &args),
	    }
	    Atom::Generator(g)
	},	
	
	Some(l) => {
	    tail.push(l);

	    let mut tail_drain = tail.drain(..); 	    
	    let mut args = Vec::new();

	    while let Some(Expr::Constant(Atom::Float(f))) = tail_drain.next() {
		args.push(f);
	    }
    
	    Atom::GeneratorModifierFunction (match gen_mod {
		BuiltInGenModFun::Haste => (haste, args),
		BuiltInGenModFun::Relax => (relax, args),
		BuiltInGenModFun::Grow => (grow, args),
	    })
	},
	None => {
	    Atom::Nothing
	}
    } 
}
