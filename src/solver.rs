
use std::fmt::Debug;
use std::collections::HashMap;

use crate::tree::{ Term, TermKind, Expr, ExprKind, Clause, atom };



use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UniqueID(pub usize);

static mut _NEXT_ID: usize = 10;

impl UniqueID {
    pub fn generate() -> UniqueID {
        unsafe {
            _NEXT_ID += 1;
            //format!("anon{}", _next_id)
            UniqueID(_NEXT_ID)
        }
    }
}

impl fmt::Display for UniqueID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}




#[derive(Clone, Debug, PartialEq)]
pub struct Bindings(HashMap<String, Term>);

impl Bindings {
    pub fn empty() -> Self {
        Bindings(HashMap::new())
    }

    pub fn with(name: &str, term: Term) -> Self {
        let mut bindings = Bindings(HashMap::new());
        bindings.0.insert(name.to_string(), term);
        bindings
    }

    pub fn merge(&self, bindings: &Bindings) -> Option<Bindings> {
        let mut new_bindings = Bindings::empty();
        new_bindings.0.extend(self.0.clone());
        for (key, value) in bindings.0.iter() {
            match new_bindings.0.get(key) {
                Some(original) if !compare_term(original, value) => {
                    println!("Already defined {:?} as {:?} but trying to set to {:?}", key, original, value);
                    return None;
                },
                _ => {
                    new_bindings.0.insert(key.clone(), value.clone());
                },
            }
        }
        Some(new_bindings)
    }

    pub fn substitute(&self, term: Term) -> Term {
        Box::new(match *term {
            TermKind::EmptyList => TermKind::EmptyList,
            TermKind::Atom(n) => TermKind::Atom(n),
            TermKind::Compound(n, args) => {
                let args = args.iter().map(|t| self.substitute(t.clone())).collect();
                TermKind::Compound(n, args)
            },
            TermKind::List(head, tail) => {
                TermKind::List(self.substitute(head), self.substitute(tail))
            },
            TermKind::Var(n) => {
                if let Some(t) = self.0.get(&n) {
                    println!("Substituting {:?} for {:?}", n, t);
                    *self.substitute(t.clone())
                } else {
                    TermKind::Var(n)
                }
            },
        })
    }

    pub fn rename_term(&mut self, term: Term) -> Term {
        Box::new(match *term {
            TermKind::EmptyList => TermKind::EmptyList,
            TermKind::Atom(n) => TermKind::Atom(n),
            TermKind::Compound(n, args) => {
                let args = args.iter().map(|t| self.rename_term(t.clone())).collect();
                TermKind::Compound(n, args)
            },
            TermKind::List(head, tail) => {
                TermKind::List(self.rename_term(head), self.rename_term(tail))
            },
            TermKind::Var(n) => {
                if let Some(t) = self.0.get(&n) {
                    *t.clone()
                } else {
                    let var = TermKind::Var(format!("{}_{}", n, UniqueID::generate()));
                    self.0.insert(n.to_string(), Box::new(var.clone()));
                    var
                }
            },
        })
    }

    pub fn rename_expr(&mut self, expr: Expr) -> Expr {
        Box::new(match *expr {
            ExprKind::Term(term) => {
                ExprKind::Term(self.rename_term(term))
            },
            ExprKind::Conjunct(expr1, expr2) => {
                ExprKind::Conjunct(self.rename_expr(expr1), self.rename_expr(expr2))
            },
        })
    }
}

pub type BuiltinPredicate = fn(&Term, &Bindings, usize) -> Option<Partial>;

pub struct Database {
    clauses: Vec<Clause>,
}

impl Database {
    pub fn new(clauses: Vec<Clause>) -> Self {
        Database {
            clauses: clauses,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Partial {
    pub result: Term,
    pub bindings: Bindings,
    pub rule: usize,
}

impl Partial {
    pub fn new(result: Term, bindings: Bindings, rule: usize) -> Partial {
        Partial {
            result: result,
            bindings: bindings,
            rule: rule,
        }
    }
}

pub struct Query {
    goal: Term,
}

impl Query {
    pub fn new(term: Term) -> Self {
        Query {
            goal: term,
        }
    }

    #[allow(dead_code)]
    pub fn solve(&self, db: &Database) -> Option<Partial> {
        self.solve_from(db, 0)
    }

    pub fn solve_from(&self, db: &Database, at: usize) -> Option<Partial> {
        if at >= db.clauses.len() {
            return None;
        }

        println!("Solving {:?} at {}", self.goal, at);
        if let Some(func) = lookup_builtin(&self.goal) {
            return func(&self.goal, &Bindings::empty(), db.clauses.len());
        }

        for i in at..db.clauses.len() {
            match &db.clauses[i] {
                Clause::Fact(t) => {
                    println!("Unifying {} with {}", self.goal, t);
                    if let Some((n, bindings)) = unify_term(&self.goal, &t) {
                        println!("Returning {:?} {:?}", n, bindings);
                        return Some(Partial::new(n, bindings, i));
                    }
                },
                Clause::Rule(lhs, rhs) => {
                    let mut renaming = Bindings::empty();
                    let lhs = renaming.rename_term(lhs.clone());
                    let rhs = renaming.rename_expr(rhs.clone());

                    println!("Unifying {} with {}", self.goal, lhs);
                    if let Some((_, mut bindings)) = unify_term(&self.goal, &lhs) {
                        if let Some(partial) = self._solve_expression(db, &bindings, &rhs, 0) {
                            bindings = bindings.merge(&partial.bindings)?;
                            println!("Returning {:?} {:?}", bindings.substitute(lhs.clone()), bindings);
                            return Some(Partial::new(bindings.substitute(lhs.clone()), bindings, i));
                        }
                    }
                },
            }
        }

        None
    }

    fn _solve_expression(&self, db: &Database, init_bindings: &Bindings, expr: &Expr, at_rule: usize) -> Option<Partial> {
        match &**expr {
            ExprKind::Term(term) => {
                let dependent = Query { goal: init_bindings.substitute(term.clone()) };
                match dependent.solve_from(db, at_rule) {
                    Some(mut partial) => {
                        partial.bindings = init_bindings.merge(&partial.bindings)?;
                        Some(partial)
                    },
                    None => None,
                }
            },
            ExprKind::Conjunct(expr1, expr2) => {
                let mut rule = 0;
                loop {
                    match self._solve_expression(db, init_bindings, expr1, rule) {
                        Some(partial) => {
                            // The first expr has a result, and if the second expr also has a result, then return
                            // Otherwise backtrack and try to find another solution to the first expr
                            let bindings = init_bindings.merge(&partial.bindings)?;
                            if let Some(partial) = self._solve_expression(db, &bindings, expr2, 0) {
                                return Some(partial);
                            }

                            // If the previous expression was the cut operator, then don't backtrack
                            if is_atom_of(expr1, "!") {
                                println!("Cut");
                                return None;
                            }

                            rule = partial.rule + 1;
                            println!("Backtracking");
                        },
                        None => {
                            println!("Out of backtrack options");
                            return None;
                       }
                    }
                }
            },
        }
    }
}

pub fn is_atom_of(expr: &Expr, expected: &str) -> bool {
    if let ExprKind::Term(term) = &**expr {
        match &**term {
            TermKind::Atom(string) | TermKind::Compound(string, _) => {
                if string.as_str() == expected {
                    return true;
                }
            },
            _ => { },
        }
    }
    return false;
}

pub fn unify_term(term1: &Term, term2: &Term) -> Option<(Term, Bindings)> {
    match (&**term1, &**term2) {
        (TermKind::Atom(n), TermKind::Atom(m)) if n == m => Some((Box::new(TermKind::Atom(n.clone())), Bindings::empty())),

        (TermKind::Compound(n, args1), TermKind::Compound(m, args2)) if n == m && args1.len() == args2.len() => {
            let mut args = vec!();
            let mut bindings = Bindings::empty();

            for (a1, a2) in args1.iter().zip(args2.iter()) {
                if let Some((n, new_bindings)) = unify_term(&a1, &a2) {
                    args.push(n);
                    bindings = bindings.merge(&new_bindings)?;
                } else {
                    return None;
                }
            }
            Some((Box::new(TermKind::Compound(n.clone(), args)), bindings))
        },

        (TermKind::EmptyList, TermKind::EmptyList) => Some((Box::new(TermKind::EmptyList), Bindings::empty())),

        (TermKind::List(h1, t1), TermKind::List(h2, t2)) => {
            let (head, head_bindings) = unify_term(&h1, &h2)?;
            let (tail, tail_bindings) = unify_term(&t1, &t2)?;
            Some((Box::new(TermKind::List(head, tail)), head_bindings.merge(&tail_bindings)?))
        },

        (TermKind::Var(n), TermKind::Var(m)) if n == m => {
            Some((Box::new(TermKind::Var(n.clone())), Bindings::empty()))
        },

        (TermKind::Var(n), m) | (m, TermKind::Var(n)) => {
            println!("Binding {} to {}", n, m);
            Some((Box::new(m.clone()), Bindings::with(n, Box::new(m.clone()))))
        },

        _ => None
    }
}

fn compare_term(term1: &Term, term2: &Term) -> bool {
    match (&**term1, &**term2) {
        (TermKind::Atom(n), TermKind::Atom(m)) if n == m => true,
        (TermKind::Compound(n, args1), TermKind::Compound(m, args2)) if n == m && args1.len() == args2.len() => {
            for (a1, a2) in args1.iter().zip(args2.iter()) {
                if !compare_term(a1, a2) {
                    return false;
                }
            }
            true
        },
        (TermKind::EmptyList, TermKind::EmptyList) => true,
        (TermKind::List(h1, t1), TermKind::List(h2, t2)) => {
            if !compare_term(&h1, &h2) {
                return false;
            }
            compare_term(&t1, &t2)
        },
        (TermKind::Var(n), TermKind::Var(m)) if n == m => true,
        _ => false,
    }
}


pub fn lookup_builtin(term: &Term) -> Option<BuiltinPredicate> {
    match &**term {
        TermKind::Atom(s) => {
            match s.as_str() {
                "!" => Some(builtin_cut_0),
                "fail" => Some(builtin_fail_0),
                _ => None
            }
        },
        TermKind::Compound(s, args) => {
            match s.as_str() {
                "equal" if args.len() == 2 => Some(builtin_equal_2),
                "not_equal" if args.len() == 2 => Some(builtin_not_equal_2),
                _ => None
            }
        }
        _ => None,
    }
}

fn builtin_cut_0(_term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    Some(Partial::new(atom("true"), bindings.clone(), at_rule))
}

fn builtin_fail_0(_term: &Term, _bindings: &Bindings, _at_rule: usize) -> Option<Partial> {
    None
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

