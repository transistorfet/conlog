
use std::fmt::Debug;


#[derive(Clone, Debug, PartialEq)]
pub enum TermKind {
    Var(String),
    Atom(String),
    Compound(String, Vec<Term>),
    Conjunct(Term, Term),
}

pub type Term = Box<TermKind>;

/*
#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Term(Term),
    Conjunct(Expression, Expression),
}

pub type Expression = Box<ExprKind>;
*/

#[derive(Clone, Debug, PartialEq)]
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

