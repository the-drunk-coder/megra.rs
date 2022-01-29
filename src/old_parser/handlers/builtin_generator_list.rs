use crate::builtin_types::*;

#[allow(clippy::mut_range_bound)]
pub fn handle(tail: &mut Vec<Expr>) -> Atom {
    let mut gen_list = Vec::new();

    let mut tail_drain = tail.drain(..);
    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::Generator(g) => {
                gen_list.push(g);
            }
            Atom::GeneratorList(mut gl) => {
                gen_list.append(&mut gl);
            }
            _ => {
                println!("u can't list this ...");
            }
        }
    }

    Atom::GeneratorList(gen_list)
}
