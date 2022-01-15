use crate::builtin_types::*;
use crate::generator::*;
use std::collections::HashMap;

pub fn handle(gen_mod: &BuiltInGenModFun, tail: &mut Vec<Expr>) -> Atom {
    //println!("handle schrink?");
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
                        named_args.insert(
                            k,
                            match tail_drain.next() {
                                Some(Expr::Constant(Atom::Float(f))) => ConfigParameter::Numeric(f),
                                Some(Expr::Constant(Atom::Symbol(s))) => {
                                    ConfigParameter::Symbolic(s)
                                }
                                _ => ConfigParameter::Numeric(0.0), // dumb placeholder
                            },
                        );
                    }
                    _ => {}
                }
            }

            match gen_mod {
                BuiltInGenModFun::Haste => haste(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Relax => relax(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Grow => grow(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Shrink => shrink(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Solidify => solidify(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Blur => blur(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Sharpen => sharpen(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Shake => shake(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Skip => skip(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Rewind => rewind(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Rnd => rnd(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Rep => rep(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
                BuiltInGenModFun::Reverse => reverse(
                    &mut g.root_generator,
                    &mut g.time_mods,
                    &pos_args,
                    &named_args,
                ),
            }
            Atom::Generator(g)
        }
        Some(Expr::Constant(Atom::GeneratorProcessorOrModifier(gpom))) => {
            let mut tail_drain = tail.drain(..);
            let mut pos_args = Vec::new();
            let mut named_args = HashMap::new();

            while let Some(Expr::Constant(c)) = tail_drain.next() {
                match c {
                    Atom::Float(f) => pos_args.push(ConfigParameter::Numeric(f)),
                    Atom::Keyword(k) => {
                        named_args.insert(
                            k,
                            match tail_drain.next() {
                                Some(Expr::Constant(Atom::Float(f))) => ConfigParameter::Numeric(f),
                                Some(Expr::Constant(Atom::Symbol(s))) => {
                                    ConfigParameter::Symbolic(s)
                                }
                                _ => ConfigParameter::Numeric(0.0), // dumb placeholder
                            },
                        );
                    }
                    _ => {}
                }
            }
            let gm = GeneratorProcessorOrModifier::GeneratorModifierFunction(match gen_mod {
                BuiltInGenModFun::Haste => (haste, pos_args, named_args),
                BuiltInGenModFun::Relax => (relax, pos_args, named_args),
                BuiltInGenModFun::Grow => (grow, pos_args, named_args),
                BuiltInGenModFun::Shrink => (shrink, pos_args, named_args),
                BuiltInGenModFun::Solidify => (solidify, pos_args, named_args),
                BuiltInGenModFun::Blur => (blur, pos_args, named_args),
                BuiltInGenModFun::Sharpen => (sharpen, pos_args, named_args),
                BuiltInGenModFun::Shake => (shake, pos_args, named_args),
                BuiltInGenModFun::Skip => (skip, pos_args, named_args),
                BuiltInGenModFun::Rewind => (rewind, pos_args, named_args),
                BuiltInGenModFun::Rnd => (rnd, pos_args, named_args),
                BuiltInGenModFun::Rep => (rep, pos_args, named_args),
                BuiltInGenModFun::Reverse => (reverse, pos_args, named_args),
            });
            match gpom {
                GeneratorProcessorOrModifier::GeneratorProcessor(_) => {
                    Atom::GeneratorProcessorOrModifierList(vec![gpom, gm])
                }
                GeneratorProcessorOrModifier::GeneratorModifierFunction(_) => {
                    Atom::GeneratorModifierList(vec![gpom, gm])
                }
            }
        }
        Some(Expr::Constant(Atom::GeneratorProcessorOrModifierList(mut gpoml))) => {
            let mut tail_drain = tail.drain(..);
            let mut pos_args = Vec::new();
            let mut named_args = HashMap::new();

            while let Some(Expr::Constant(c)) = tail_drain.next() {
                match c {
                    Atom::Float(f) => pos_args.push(ConfigParameter::Numeric(f)),
                    Atom::Keyword(k) => {
                        named_args.insert(
                            k,
                            match tail_drain.next() {
                                Some(Expr::Constant(Atom::Float(f))) => ConfigParameter::Numeric(f),
                                Some(Expr::Constant(Atom::Symbol(s))) => {
                                    ConfigParameter::Symbolic(s)
                                }
                                _ => ConfigParameter::Numeric(0.0), // dumb placeholder
                            },
                        );
                    }
                    _ => {}
                }
            }
            let gm = GeneratorProcessorOrModifier::GeneratorModifierFunction(match gen_mod {
                BuiltInGenModFun::Haste => (haste, pos_args, named_args),
                BuiltInGenModFun::Relax => (relax, pos_args, named_args),
                BuiltInGenModFun::Grow => (grow, pos_args, named_args),
                BuiltInGenModFun::Shrink => (shrink, pos_args, named_args),
                BuiltInGenModFun::Solidify => (solidify, pos_args, named_args),
                BuiltInGenModFun::Blur => (blur, pos_args, named_args),
                BuiltInGenModFun::Sharpen => (sharpen, pos_args, named_args),
                BuiltInGenModFun::Shake => (shake, pos_args, named_args),
                BuiltInGenModFun::Skip => (skip, pos_args, named_args),
                BuiltInGenModFun::Rewind => (rewind, pos_args, named_args),
                BuiltInGenModFun::Rnd => (rnd, pos_args, named_args),
                BuiltInGenModFun::Rep => (rep, pos_args, named_args),
                BuiltInGenModFun::Reverse => (rep, pos_args, named_args),
            });
            gpoml.push(gm);
            Atom::GeneratorProcessorOrModifierList(gpoml)
        }
        Some(Expr::Constant(Atom::GeneratorModifierList(mut gpoml))) => {
            let mut tail_drain = tail.drain(..);
            let mut pos_args = Vec::new();
            let mut named_args = HashMap::new();

            while let Some(Expr::Constant(c)) = tail_drain.next() {
                match c {
                    Atom::Float(f) => pos_args.push(ConfigParameter::Numeric(f)),
                    Atom::Keyword(k) => {
                        named_args.insert(
                            k,
                            match tail_drain.next() {
                                Some(Expr::Constant(Atom::Float(f))) => ConfigParameter::Numeric(f),
                                Some(Expr::Constant(Atom::Symbol(s))) => {
                                    ConfigParameter::Symbolic(s)
                                }
                                _ => ConfigParameter::Numeric(0.0), // dumb placeholder
                            },
                        );
                    }
                    _ => {}
                }
            }
            let gm = GeneratorProcessorOrModifier::GeneratorModifierFunction(match gen_mod {
                BuiltInGenModFun::Haste => (haste, pos_args, named_args),
                BuiltInGenModFun::Relax => (relax, pos_args, named_args),
                BuiltInGenModFun::Grow => (grow, pos_args, named_args),
                BuiltInGenModFun::Shrink => (shrink, pos_args, named_args),
                BuiltInGenModFun::Solidify => (solidify, pos_args, named_args),
                BuiltInGenModFun::Blur => (blur, pos_args, named_args),
                BuiltInGenModFun::Sharpen => (sharpen, pos_args, named_args),
                BuiltInGenModFun::Shake => (shake, pos_args, named_args),
                BuiltInGenModFun::Skip => (skip, pos_args, named_args),
                BuiltInGenModFun::Rewind => (rewind, pos_args, named_args),
                BuiltInGenModFun::Rnd => (rnd, pos_args, named_args),
                BuiltInGenModFun::Rep => (rep, pos_args, named_args),
                BuiltInGenModFun::Reverse => (rep, pos_args, named_args),
            });
            gpoml.push(gm);
            Atom::GeneratorModifierList(gpoml)
        }
        Some(l) => {
            tail.push(l);

            let mut pos_args = Vec::new();
            let mut named_args = HashMap::new();
            let mut tail_drain = tail.drain(..);

            while let Some(Expr::Constant(c)) = tail_drain.next() {
                match c {
                    Atom::Float(f) => pos_args.push(ConfigParameter::Numeric(f)),
                    Atom::Keyword(k) => {
                        named_args.insert(
                            k,
                            match tail_drain.next() {
                                Some(Expr::Constant(Atom::Float(f))) => ConfigParameter::Numeric(f),
                                Some(Expr::Constant(Atom::Symbol(s))) => {
                                    ConfigParameter::Symbolic(s)
                                }
                                _ => ConfigParameter::Numeric(0.0), // dumb placeholder
                            },
                        );
                    }
                    _ => {}
                }
            }

            Atom::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorModifierFunction(match gen_mod {
                    BuiltInGenModFun::Haste => (haste, pos_args, named_args),
                    BuiltInGenModFun::Relax => (relax, pos_args, named_args),
                    BuiltInGenModFun::Grow => (grow, pos_args, named_args),
                    BuiltInGenModFun::Shrink => (shrink, pos_args, named_args),
                    BuiltInGenModFun::Solidify => (solidify, pos_args, named_args),
                    BuiltInGenModFun::Blur => (blur, pos_args, named_args),
                    BuiltInGenModFun::Sharpen => (sharpen, pos_args, named_args),
                    BuiltInGenModFun::Shake => (shake, pos_args, named_args),
                    BuiltInGenModFun::Skip => (skip, pos_args, named_args),
                    BuiltInGenModFun::Rewind => (rewind, pos_args, named_args),
                    BuiltInGenModFun::Rnd => (rnd, pos_args, named_args),
                    BuiltInGenModFun::Rep => (rep, pos_args, named_args),
                    BuiltInGenModFun::Reverse => (reverse, pos_args, named_args),
                }),
            )
        }
        None => match gen_mod {
            BuiltInGenModFun::Shrink => Atom::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorModifierFunction((
                    shrink,
                    Vec::new(),
                    HashMap::new(),
                )),
            ),
            BuiltInGenModFun::Reverse => Atom::GeneratorProcessorOrModifier(
                GeneratorProcessorOrModifier::GeneratorModifierFunction((
                    reverse,
                    Vec::new(),
                    HashMap::new(),
                )),
            ),
            _ => {
                println!("genmodfun needs arguments!");
                Atom::Nothing
            }
        },
    }
}
