
use std::fmt::Debug;
use std::collections::HashMap;

use crate::tree::{ Term, TermKind, Expr, ExprKind, Clause, atom };


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

    pub fn merge(&self, bindings: &Bindings) -> Bindings {
        let mut new_bindings = Bindings::empty();
        new_bindings.0.extend(self.0.clone());
        new_bindings.0.extend(bindings.0.clone());
        new_bindings
    }

    pub fn substitute(&self, term: Term) -> Term {
        Box::new(match *term {
            TermKind::Atom(n) => TermKind::Atom(n),
            TermKind::Compound(n, args) => {
                let args = args.iter().map(|t| self.substitute(t.clone())).collect();
                TermKind::Compound(n, args)
            },
            TermKind::Var(n) => {
                if let Some(t) = self.0.get(&n) {
                    *t.clone()
                } else {
                    TermKind::Var(n)
                }
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

    pub fn solve(&self, db: &Database) -> Option<Partial> {
        self.solve_from(db, 0)
    }

    pub fn solve_from(&self, db: &Database, at: usize) -> Option<Partial> {
        for i in at..db.clauses.len() {
            match &db.clauses[i] {
                Clause::Fact(t) => {
                    if let Some((n, bindings)) = unify_term(&self.goal, &t) {
                        return Some(Partial::new(n, bindings, i));
                    }
                },
                Clause::Rule(lhs, rhs) => {
                    if let Some((_, mut bindings)) = unify_term(&self.goal, lhs) {
                        if let Some(partial) = self._solve_expression(db, &bindings, rhs, 0) {
                            bindings = bindings.merge(&partial.bindings);
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
                if let Some(func) = lookup_builtin(term) {
                    func(term, init_bindings, at_rule)
                } else {
                    let dependent = Query { goal: init_bindings.substitute(term.clone()) };
                    dependent.solve_from(db, at_rule)
                }
            },
            ExprKind::Conjunct(expr1, expr2) => {
                let mut rule = 0;
                loop {
                    match self._solve_expression(db, init_bindings, expr1, rule) {
                        Some(partial) => {
                            // The first expr has a result, and if the second expr also has a result, then return
                            // Otherwise backtrack and try to find another solution to the first expr
                            let bindings = init_bindings.merge(&partial.bindings);
                            if let Some(partial) = self._solve_expression(db, &bindings, expr2, 0) {
                                return Some(partial);
                            }

                            // If the previous expression was the cut operator, then don't backtrack
                            if is_atom_of(expr1, "!") {
                                println!("Cut");
                                return None;
                            }

                            println!("Backtracking");
                            rule = partial.rule + 1;
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
    println!("Check: {:?} =?= {:?}", term1, term2);
    match (&**term1, &**term2) {
        (TermKind::Var(n), TermKind::Var(m)) if n == m => {
            Some((Box::new(TermKind::Var(n.clone())), Bindings::empty()))
        },

        (TermKind::Var(n), m) | (m, TermKind::Var(n)) => {
            println!("Binding {:?} to {:?}", n, m);
            Some((Box::new(m.clone()), Bindings::with(n, Box::new(m.clone()))))
        },

        (TermKind::Atom(n), TermKind::Atom(m)) if n == m => Some((Box::new(TermKind::Atom(n.clone())), Bindings::empty())),

        (TermKind::Compound(n, args1), TermKind::Compound(m, args2)) if n == m && args1.len() == args2.len() => {
            let mut args = vec!();
            let mut bindings = Bindings::empty();

            for (a1, a2) in args1.iter().zip(args2.iter()) {
                if let Some((n, new_bindings)) = unify_term(&a1, &a2) {
                    args.push(n);
                    bindings = bindings.merge(&new_bindings);
                } else {
                    return None;
                }
            }
            Some((Box::new(TermKind::Compound(n.clone(), args)), bindings))
        },

        _ => None
    }
}

pub fn lookup_builtin(term: &Term) -> Option<BuiltinPredicate> {
    match &**term {
        TermKind::Atom(s) => {
            match s.as_str() {
                "!" => Some(builtin_cut),
                "fail" => Some(builtin_fail),
                _ => None
            }
        },
        _ => None,
    }
}

fn builtin_cut(_term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    Some(Partial::new(atom("true"), bindings.clone(), at_rule))
}

fn builtin_fail(_term: &Term, _bindings: &Bindings, _at_rule: usize) -> Option<Partial> {
    None
}

