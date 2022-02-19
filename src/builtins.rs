
use crate::tree::{ Term, TermKind, atom, integer };
use crate::solver::{ Partial, Bindings, simplify_term, unify_term, compare_term };

pub type BuiltinPredicate = fn(&Term, &Bindings, usize) -> Option<Partial>;

pub fn lookup_builtin(term: &Term) -> Option<BuiltinPredicate> {
    let name = match &**term {
        TermKind::Atom(s) => format!("{}/0", s),
        TermKind::Compound(s, args) => format!("{}/{}", s, args.len()),
        _ => return None,
    };

    match name.as_str() {
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

fn builtin_cut_0(_term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    Some(Partial::new(atom("true"), bindings.clone(), at_rule))
}

fn builtin_fail_0(_term: &Term, _bindings: &Bindings, _at_rule: usize) -> Option<Partial> {
    None
}

fn builtin_nl_0(_term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    println!("");
    Some(Partial::new(atom("true"), bindings.clone(), at_rule))
}

fn builtin_write_1(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    print!("{}", args[0]);
    Some(Partial::new(atom("true"), bindings.clone(), at_rule))
}

fn builtin_is_2(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    let rhs = simplify_term(&args[1], bindings, at_rule).unwrap();
    println!("{:?} {:?}", args[0], rhs.result);
    match unify_term(&args[0], &rhs.result) {
        Some((result, newbindings)) => {
            let bindings = newbindings.merge(bindings)?;
            Some(Partial::new(result, bindings, at_rule))
        },
        None => None,
    }
}

fn builtin_equal_2(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    if compare_term(&args[0], &args[1]) {
        Some(Partial::new(atom("true"), bindings.clone(), at_rule))
    } else {
        None
    }
}

fn builtin_not_equal_2(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    if !compare_term(&args[0], &args[1]) {
        Some(Partial::new(atom("true"), bindings.clone(), at_rule))
    } else {
        None
    }
}

fn builtin_less_than_2(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) if n < m => Some(Partial::new(atom("true"), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_greater_than_2(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) if n > m => Some(Partial::new(atom("true"), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_less_than_or_equal_2(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) if n <= m => Some(Partial::new(atom("true"), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_greater_than_or_equal_2(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Comparing {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) if n >= m => Some(Partial::new(atom("true"), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_add_2(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Adding {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) => Some(Partial::new(integer(n + m), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_subtract_2(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Subtracting {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) => Some(Partial::new(integer(n - m), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_multiply_2(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Adding {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) => Some(Partial::new(integer(n * m), bindings.clone(), at_rule)),
        _ => None
    }
}

fn builtin_divide_2(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    let args = term.get_args()?;

    println!("Adding {} with {}", &args[0], &args[1]);
    match (&*args[0], &*args[1]) {
        (TermKind::Integer(n), TermKind::Integer(m)) => Some(Partial::new(integer(n / m), bindings.clone(), at_rule)),
        _ => None
    }
}

