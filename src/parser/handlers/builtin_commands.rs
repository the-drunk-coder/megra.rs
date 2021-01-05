use crate::builtin_types::*;
use crate::parser::parser_helpers::*;

fn handle_load_part(tail: &mut Vec<Expr>) -> Atom {
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

fn handle_load_sample(tail: &mut Vec<Expr>) -> Atom {

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

fn handle_load_sample_set(tail: &mut Vec<Expr>) -> Atom {

    let mut tail_drain = tail.drain(..);
    let path = if let Expr::Constant(Atom::Description(n)) = tail_drain.next().unwrap() {
	n
    } else {
	"".to_string()
    };
	
    Atom::Command(Command::LoadSampleSet(path.to_string()))
}

pub fn handle(cmd: BuiltInCommand, tail: &mut Vec<Expr>) -> Atom {
    match cmd {
	BuiltInCommand::Clear => Atom::Command(Command::Clear),
	BuiltInCommand::LoadSample => handle_load_sample(tail),
	BuiltInCommand::LoadSampleSet => handle_load_sample_set(tail),
	BuiltInCommand::LoadPart => handle_load_part(tail),
    }
}
