
use crate::tree::{ Term, TermKind, atom, integer };
use crate::solver::{ Database, Query, Partial, Bindings, simplify_term, unify_term, compare_term };

pub type BuiltinPredicate = fn(&Database, &Term, &Bindings, usize) -> Option<Partial>;

pub fn lookup_builtin(term: &Term) -> Option<BuiltinPredicate> {
    let name = match &**term {
        TermKind::Atom(s) => s,
        TermKind::Compound(s, _) => s,
        _ => return None,
    };

    let builtin: Option<BuiltinPredicate> = match name.as_str() {
        "call"      => Some(builtin_call),
        _           => None,
    };

    if builtin.is_some() {
        return builtin;
    }

    let name_with_arity = match &**term {
        TermKind::Atom(s) => format!("{}/0", s),
        TermKind::Compound(s, args) => format!("{}/{}", s, args.len()),
        _ => return None,
    };

    match name_with_arity.as_str() {
        "!/0"       => Some(builtin_cut_0),
        "fail/0"    => Some(builtin_fail_0),
        "nl/0"      => Some(builtin_nl_0),
        "write/1"   => Some(builtin_write_1),
        "is/2"      => Some(builtin_is_2),
        "=/2"       => Some(builtin_equal_2),
        "\\=/2"     => Some(builtin_not_equal_2),
        "</2"       => Some(builtin_less_than_2),
        ">/2"       => Some(builtin_greater_than_2),
        "<=/2"      => Some(builtin_less_than_or_equal_2),
        ">=/2"      => Some(builtin_greater_than_or_equal_2),
        "+/2"       => Some(builtin_add_2),
        "-/2"       => Some(builtin_subtract_2),
        "*/2"       => Some(builtin_multiply_2),
        "//2"       => Some(builtin_divide_2),
        _ => None,
    }
}

fn builtin_cut_0(_db: &Database, _term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    Some(Partial::new(atom("true"), bindings.clone(), at_rule))
}

fn builtin_fail_0(_db: &Database, _term: &Term, _bindings: &Bindings, _at_rule: usize) -> Option<Partial> {
    None
}

fn builtin_nl_0(_db: &Database, _term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    println!("");
    Some(Partial::new(atom("true"), bindings.clone(), at_rule))
}

fn builtin_write_1(_db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    print!("{}", args[0]);
    Some(Partial::new(atom("true"), bindings.clone(), at_rule))
}

fn builtin_is_2(db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    let rhs = simplify_term(db, &args[1], bindings, at_rule).unwrap();
    println!("{:?} {:?}", args[0], rhs.result);
    match unify_term(&args[0], &rhs.result) {
        Some((result, newbindings)) => {
            let bindings = newbindings.merge(bindings)?;
            Some(Partial::new(result, bindings, at_rule))
        },
        None => None,
    }
}

fn builtin_equal_2(_db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    if compare_term(&args[0], &args[1]) {
        Some(Partial::new(atom("true"), bindings.clone(), at_rule))
    } else {
        None
    }
}

fn builtin_not_equal_2(_db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    if !compare_term(&args[0], &args[1]) {
        Some(Partial::new(atom("true"), bindings.clone(), at_rule))
    } else {
        None
    }
}

fn builtin_less_than_2(_db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) if n < m => Some(Partial::new(atom("true"), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_greater_than_2(_db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) if n > m => Some(Partial::new(atom("true"), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_less_than_or_equal_2(_db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) if n <= m => Some(Partial::new(atom("true"), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_greater_than_or_equal_2(_db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) if n >= m => Some(Partial::new(atom("true"), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_add_2(_db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Adding {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) => Some(Partial::new(integer(n + m), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_subtract_2(_db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Subtracting {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) => Some(Partial::new(integer(n - m), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_multiply_2(_db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Adding {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) => Some(Partial::new(integer(n * m), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_divide_2(_db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Adding {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) => Some(Partial::new(integer(n / m), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_call(db: &Database, term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let (first, args) = term.get_args()?.split_at(1);

    let result = match &*first[0] {
        TermKind::Atom(x) if args.len() == 0 => TermKind::Atom(x.to_string()),
        TermKind::Atom(x) => TermKind::Compound(x.to_string(), args.to_vec()),
        TermKind::Compound(x, first_args) => TermKind::Compound(x.to_string(), [first_args, args].concat()),
        _ => return None,
    };

    Query::new(Box::new(result)).solve(db)
}

