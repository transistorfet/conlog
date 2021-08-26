
use std::fmt::Debug;
use std::collections::HashMap;

use crate::tree::{ Term, TermKind, Expr, ExprKind, Clause };
use crate::builtins::lookup_builtin;
use crate::misc::UniqueID;


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
            TermKind::Integer(num) => TermKind::Integer(num),
            TermKind::String(string) => TermKind::String(string),
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
}

pub struct Database {
    clauses: Vec<Clause>,
}

impl Database {
    pub fn new(clauses: Vec<Clause>) -> Self {
        Database {
            clauses,
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
            result,
            bindings,
            rule,
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
                    let iteration = UniqueID::generate();
                    let t = rename_term(t.clone(), iteration);

                    println!("Unifying fact {} with {}", self.goal, t);
                    if let Some((n, bindings)) = unify_term(&self.goal, &t) {
                        println!("Returning {:?} {:?}", n, bindings);
                        return Some(Partial::new(bindings.substitute(n), bindings, i));
                    }
                },
                Clause::Rule(lhs, rhs) => {
                    let iteration = UniqueID::generate();
                    let lhs = rename_term(lhs.clone(), iteration);
                    let rhs = rename_expr(rhs.clone(), iteration);

                    println!("Unifying rule {} with {}", self.goal, lhs);
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
    false
}

pub fn unify_term(term1: &Term, term2: &Term) -> Option<(Term, Bindings)> {
    //println!("Unifying term {:?} and {:?}", term1, term2);
    match (&**term1, &**term2) {
        (TermKind::Atom(n), TermKind::Atom(m)) if n == m => Some((Box::new(TermKind::Atom(n.clone())), Bindings::empty())),

        (TermKind::Integer(n), TermKind::Integer(m)) if n == m => Some((Box::new(TermKind::Integer(*n)), Bindings::empty())),

        (TermKind::String(n), TermKind::String(m)) if n == m => Some((Box::new(TermKind::String(n.clone())), Bindings::empty())),

        (TermKind::Compound(n, args1), TermKind::Compound(m, args2)) if n == m && args1.len() == args2.len() => {
            let mut args = vec!();
            let mut bindings = Bindings::empty();

            for (a1, a2) in args1.iter().zip(args2.iter()) {
                if let Some((n, new_bindings)) = unify_term(a1, a2) {
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
            let (head, head_bindings) = unify_term(h1, h2)?;
            let (tail, tail_bindings) = unify_term(t1, t2)?;
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

pub fn simplify_term(term: &Term, bindings: &Bindings, at_rule: usize) -> Option<Partial> {
    if let Some(func) = lookup_builtin(term) {
        match func(term, bindings, at_rule) {
            Some(partial) => simplify_term(&partial.result, &partial.bindings, partial.rule),
            None => None,
        }
    } else {
        Some(Partial::new(term.clone(), bindings.clone(), at_rule))
    }
}

pub fn compare_term(term1: &Term, term2: &Term) -> bool {
    match (&**term1, &**term2) {
        (TermKind::Atom(n), TermKind::Atom(m)) if n == m => true,
        (TermKind::Integer(n), TermKind::Integer(m)) if n == m => true,
        (TermKind::String(n), TermKind::String(m)) if n == m => true,
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
            if !compare_term(h1, h2) {
                return false;
            }
            compare_term(t1, t2)
        },
        (TermKind::Var(n), TermKind::Var(m)) if n == m => true,
        _ => false,
    }
}

fn rename_term(term: Term, iteration: UniqueID) -> Term {
    Box::new(match *term {
        TermKind::EmptyList => TermKind::EmptyList,
        TermKind::Atom(n) => TermKind::Atom(n),
        TermKind::Integer(n) => TermKind::Integer(n),
        TermKind::String(n) => TermKind::String(n),
        TermKind::Compound(n, args) => {
            let args = args.iter().map(|t| rename_term(t.clone(), iteration)).collect();
            TermKind::Compound(n, args)
        },
        TermKind::List(head, tail) => {
            TermKind::List(rename_term(head, iteration), rename_term(tail, iteration))
        },
        TermKind::Var(n) => {
            TermKind::Var(format!("{}_{}", n, iteration))
        },
    })
}

fn rename_expr(expr: Expr, iteration: UniqueID) -> Expr {
    Box::new(match *expr {
        ExprKind::Term(term) => {
            ExprKind::Term(rename_term(term, iteration))
        },
        ExprKind::Conjunct(expr1, expr2) => {
            ExprKind::Conjunct(rename_expr(expr1, iteration), rename_expr(expr2, iteration))
        },
    })
}

