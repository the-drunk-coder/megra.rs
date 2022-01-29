use crate::builtin_types::*;
use crate::generator::Generator;
use crate::parser::parser_helpers::*;
use crate::session::SyncContext;
use std::collections::BTreeSet;

pub fn handle(tail: &mut Vec<Expr>) -> Atom {
    let mut tail_drain = tail.drain(..);

    // name is the first symbol
    let name = if let Some(thing) = tail_drain.next() {
        if let Some(namestring) = get_string_from_expr(&thing) {
            namestring
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    let active = if let Some(thing) = tail_drain.next() {
        if let Some(act) = get_bool_from_expr(&thing) {
            act
        } else {
            false
        }
    } else {
        false
    };

    if !active {
        return Atom::SyncContext(SyncContext {
            name,
            generators: Vec::new(),
            part_proxies: Vec::new(),
            sync_to: None,
            active: false,
            shift: 0,
            block_tags: BTreeSet::new(),
            solo_tags: BTreeSet::new(),
        });
    }

    let mut gens: Vec<Generator> = Vec::new();
    let mut proxies: Vec<PartProxy> = Vec::new();
    let mut sync_to = None;
    let mut shift: i32 = 0;
    let mut collect_block_tags: bool = false;
    let mut collect_solo_tags: bool = false;
    let mut block_tags: BTreeSet<String> = BTreeSet::new();
    let mut solo_tags: BTreeSet<String> = BTreeSet::new();

    while let Some(Expr::Constant(c)) = tail_drain.next() {
        match c {
            Atom::Keyword(k) => {
                match k.as_str() {
                    "sync" => {
                        collect_solo_tags = false;
                        collect_block_tags = false;
                        if let Expr::Constant(Atom::Symbol(sync)) = tail_drain.next().unwrap() {
                            sync_to = Some(sync);
                        }
                    }
                    "shift" => {
                        collect_solo_tags = false;
                        collect_block_tags = false;
                        if let Expr::Constant(Atom::Float(f)) = tail_drain.next().unwrap() {
                            shift = f as i32;
                        }
                    }
                    "solo" => {
                        collect_block_tags = false;
                        collect_solo_tags = true;
                    }
                    "block" => {
                        collect_solo_tags = false;
                        collect_block_tags = true;
                    }
                    _ => {} // ignore
                }
            }
            Atom::Symbol(s) => {
                if collect_solo_tags {
                    solo_tags.insert(s);
                } else if collect_block_tags {
                    block_tags.insert(s);
                } else {
                    // assume it's a part proxy
                    // part proxy without additional modifiers
                    proxies.push(PartProxy::Proxy(s, Vec::new()));
                }
            }
            Atom::PartProxy(p) => {
                collect_solo_tags = false;
                collect_block_tags = false;
                // part proxy without additional modifiers
                proxies.push(p);
            }
            Atom::ProxyList(mut l) => {
                collect_solo_tags = false;
                collect_block_tags = false;
                // part proxy without additional modifiers
                proxies.append(&mut l);
            }
            Atom::Generator(mut k) => {
                collect_solo_tags = false;
                collect_block_tags = false;
                k.id_tags.insert(name.clone());
                gens.push(k);
            }
            Atom::GeneratorList(mut kl) => {
                collect_solo_tags = false;
                collect_block_tags = false;
                for k in kl.iter_mut() {
                    k.id_tags.insert(name.clone());
                }
                gens.append(&mut kl);
            }
            _ => println! {"ignored"},
        }
    }

    Atom::SyncContext(SyncContext {
        name,
        generators: gens,
        part_proxies: proxies,
        sync_to,
        active: true,
        shift,
        block_tags,
        solo_tags,
    })
}
