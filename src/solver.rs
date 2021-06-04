
use std::fmt::Debug;
use std::collections::HashMap;


#[derive(Clone, Debug, PartialEq)]
pub enum TermKind {
    Var(String),
    Atom(String),
    Compound(String, Vec<Term>),
    Conjunct(Term, Term),
}

pub type Term = Box<TermKind>;

/*
pub enum ExprKind {
    Term(Term),
    Conjunct(Expression, Expression),
}

pub type Expression = Box<ExprKind>;
*/

pub enum Clause {
    // you could eliminate the fact special case by having a termkind True, which is the rhs of a rule (such that True is a special case rather than a normal Atom)
    Fact(Term),
    Rule(Term, Term),
}

pub fn variable(name: &str) -> Term {
    Box::new(TermKind::Var(name.to_string()))
}

pub fn atom(name: &str) -> Term {
    Box::new(TermKind::Atom(name.to_string()))
}

pub fn compound(name: &str, args: Vec<Term>) -> Term {
    Box::new(TermKind::Compound(name.to_string(), args))
}

pub fn conjunct(term1: Term, term2: Term) -> Term {
    Box::new(TermKind::Conjunct(term1, term2))
}

pub fn fact(term: Term) -> Clause {
    Clause::Fact(term)
}

pub fn rule(lhs: Term, rhs: Term) -> Clause {
    Clause::Rule(lhs, rhs)
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
            TermKind::Conjunct(n, m) => {
                TermKind::Conjunct(self.substitute(n.clone()), self.substitute(m.clone()))
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
        let mut i = at;

        while i < db.clauses.len() {
            match &db.clauses[i] {
                Clause::Fact(t) => {
                    if let Some((n, bindings)) = unify_term(&self.goal, &t) {
                        return Some(Partial::new(n, bindings, i));
                    }
                },
                Clause::Rule(lhs, rhs) => {
                    if let Some((n, mut bindings)) = unify_term(&self.goal, lhs) {
                        let mut queries = vec!();

                        _collect_subqueries(&mut queries, rhs.clone());

                        if let Some(partial) = self._solve_dependents(db, &bindings, queries) {
                            bindings = bindings.merge(&partial.bindings);
                            return Some(Partial::new(bindings.substitute(lhs.clone()), bindings, i));
                        }
                    }
                },
            }
            i += 1;
        }

        None
    }

    fn _solve_dependents(&self, db: &Database, init_bindings: &Bindings, queries: Vec<Term>) -> Option<Partial> {
        let mut i = 0;
        let mut at_rule = 0;
        let mut bindings = init_bindings.clone();
        let mut partials: Vec<Partial> = vec!();

        while i < queries.len() {
            let mut dependent = Query { goal: bindings.substitute(queries[i].clone()) };
            println!("Solving {:?} at {:?}", dependent.goal, at_rule);
            match dependent.solve_from(db, at_rule) {
                Some(partial) => {
                    if partials.len() <= i {
                        partials.push(partial);
                    } else {
                        partials[i] = partial;
                    }

                    bindings = bindings.merge(&partials[i].bindings.clone());
                    at_rule = 0;
                    i += 1;
                },
                None => {
                    println!("Backtracking");
                    if i == 0 {
                        println!("Out of backtrack options");
                        return None;
                    }

                    bindings = if i == 1 {
                        init_bindings.clone()
                    } else {
                        partials[i - 2].bindings.clone()
                    };
                    at_rule = partials[i - 1].rule + 1;
                    i -= 1;
                },
            }
        }
        return Some(Partial::new(atom("true"), bindings, 0));
    }
}


fn _collect_subqueries(list: &mut Vec<Term>, expr: Term) {
    match *expr {
        TermKind::Conjunct(t1, t2) => {
            _collect_subqueries(list, t1);
            _collect_subqueries(list, t2);
        },
        t @ _ => list.push(Box::new(t)),
    }
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

