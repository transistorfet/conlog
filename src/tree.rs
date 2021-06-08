
use std::fmt::Debug;
use std::convert::From;


#[derive(Clone, Debug, PartialEq)]
pub enum TermKind {
    Var(String),
    Atom(String),
    Compound(String, Vec<Term>),
}

pub type Term = Box<TermKind>;


#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Term(Term),
    Conjunct(Expr, Expr),
}

pub type Expr = Box<ExprKind>;


#[derive(Clone, Debug, PartialEq)]
pub enum Clause {
    // you could eliminate the fact special case by having a termkind True, which is the rhs of a rule (such that True is a special case rather than a normal Atom)
    Fact(Term),
    Rule(Term, Expr),
}


impl From<Term> for Expr {
    fn from(item: Term) -> Self {
        Box::new(ExprKind::Term(item))
    }
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

pub fn conjunct(expr1: impl Into<Expr>, expr2: impl Into<Expr>) -> Expr {
    Box::new(ExprKind::Conjunct(expr1.into(), expr2.into()))
}

pub fn fact(term: Term) -> Clause {
    Clause::Fact(term)
}

pub fn rule(lhs: Term, rhs: impl Into<Expr>) -> Clause {
    Clause::Rule(lhs, rhs.into())
}

